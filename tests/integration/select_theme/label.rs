//! Tests of The label that is shown.

use super::{
    Fruit, Marker, Requests, app_that_logs_requests, content, list, menu_event, popup, report_row,
    rows, shown, spawn_fruit, spawn_preselected, text, trigger, visibility,
};
use crate::support::test_app;
use bevy::{
    prelude::*,
    ui_widgets::{MenuAction, ReselectListRow},
};
use bevy_ui_component_prototypes::select::{SelectedOption, select};

#[test]
fn nothing_chosen_shows_the_placeholder() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

#[test]
fn without_a_placeholder_nothing_is_shown() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .option(Fruit::Apple, text("Apple"))
                .build(),
        )
        .id();

    app.update();

    assert_eq!(shown_text(&app, root), None);
}

#[test]
fn the_initial_choice_shows_its_label_after_one_update() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .placeholder(text("Pick one"))
                .options([
                    (Fruit::Apple, text("Apple")),
                    (Fruit::Banana, text("Banana")),
                ])
                .selected(Fruit::Banana)
                .build(),
        )
        .id();

    app.update();

    assert_eq!(shown_text(&app, root), Some("Banana".to_string()));
}

#[test]
fn changing_the_choice_replaces_the_label() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    let placeholder = shown(&app, root).unwrap();

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Cherry,
    });
    app.update();

    assert_eq!(shown_text(&app, root), Some("Cherry".to_string()));
    assert!(app.world().get_entity(placeholder).is_err());
    assert_eq!(children_of(&app, content(&app, root)), 1);
}

#[test]
fn labels_of_different_types_can_be_mixed_and_land_on_the_label_entity() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .option(Fruit::Apple, text("Apple"))
                .option(Fruit::Banana, (Text::new("Banana"), Marker(7)))
                .selected(Fruit::Banana)
                .build(),
        )
        .id();

    app.update();

    let label = shown(&app, root).unwrap();
    assert_eq!(app.world().get::<Marker>(label).map(|m| m.0), Some(7));
    // Neither the trigger nor the root gets it.
    assert!(app.world().get::<Marker>(trigger(&app, root)).is_none());
    assert!(app.world().get::<Marker>(root).is_none());
}

#[test]
fn a_value_chooses_its_option_and_removing_it_shows_the_placeholder_again() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Banana,
    });
    app.update();
    assert_eq!(shown_text(&app, root), Some("Banana".to_string()));

    app.world_mut()
        .entity_mut(root)
        .remove::<SelectedOption<Fruit>>();
    app.update();
    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

#[test]
fn a_value_that_is_not_an_option_shows_the_placeholder() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .placeholder(text("Pick one"))
                .option(Fruit::Apple, text("Apple"))
                .build(),
        )
        .id();
    app.update();

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Cherry,
    });
    app.update();

    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

#[test]
fn a_row_of_another_select_is_not_a_choice_of_this_one() {
    let mut app = app_that_logs_requests();
    let first = spawn_fruit(&mut app);
    let second = spawn_fruit(&mut app);
    let first_trigger = trigger(&app, first);
    let first_list = list(&app, first);
    let foreign_row = rows(&app, second)[1];
    menu_event(&mut app, first_trigger, MenuAction::Toggle);

    report_row(&mut app, first_list, foreign_row);

    assert!(app.world().resource::<Requests>().0.is_empty());
    assert_eq!(visibility(&app, popup(&app, first)), Visibility::Inherited);
}

#[test]
fn a_reselect_of_a_row_of_another_select_does_not_close_this_one() {
    let mut app = test_app();
    let first = spawn_fruit(&mut app);
    let second = spawn_fruit(&mut app);
    let first_trigger = trigger(&app, first);
    let first_list = list(&app, first);
    let foreign_row = rows(&app, second)[0];
    menu_event(&mut app, first_trigger, MenuAction::Toggle);

    app.world_mut().trigger(ReselectListRow {
        source: first_list,
        row: foreign_row,
    });
    app.update();

    assert_eq!(visibility(&app, popup(&app, first)), Visibility::Inherited);
}

#[test]
fn a_preselected_option_is_a_selected_option_like_any_other() {
    let mut app = test_app();
    let root = spawn_preselected(&mut app, Fruit::Banana);

    assert_eq!(
        app.world().get::<SelectedOption<Fruit>>(root),
        Some(&SelectedOption {
            value: Fruit::Banana
        })
    );
}

#[test]
fn a_preselected_select_can_be_cleared_by_removing_the_value() {
    let mut app = test_app();
    let root = spawn_preselected(&mut app, Fruit::Banana);
    app.update();

    app.world_mut()
        .entity_mut(root)
        .remove::<SelectedOption<Fruit>>();
    app.update();

    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

#[test]
fn a_preselected_value_that_is_not_an_option_chooses_nothing() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .placeholder(text("Pick one"))
                .option(Fruit::Apple, text("Apple"))
                .selected(Fruit::Cherry)
                .build(),
        )
        .id();
    app.update();

    assert!(app.world().get::<SelectedOption<Fruit>>(root).is_none());
    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

fn children_of(app: &App, entity: Entity) -> usize {
    app.world().get::<Children>(entity).map_or(0, |c| c.len())
}

fn shown_text(app: &App, root: Entity) -> Option<String> {
    shown(app, root).map(|label| app.world().get::<Text>(label).unwrap().0.clone())
}
