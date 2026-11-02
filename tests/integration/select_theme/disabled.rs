//! Tests of Disabled.

use super::{
    Fruit, Requests, app_that_logs_requests, fruit_select, list, menu_event, popup, report_row,
    rows, spawn_fruit, text, trigger, trigger_node, visibility,
};
use crate::{
    probe::probe_theme,
    support::{bg, test_app},
};
use bevy::{
    a11y::AccessibilityNode,
    input_focus::{
        InputFocus,
        tab_navigation::{NavAction, TabIndex},
    },
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{ActiveDescendant, MenuAction},
};
use bevy_ui_component_prototypes::select;

#[test]
fn a_disabled_select_has_a_gray_trigger_that_is_out_of_the_tab_order() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn((fruit_select(), InteractionDisabled))
        .id();

    app.update();

    let trigger = trigger(&app, root);
    assert!(app.world().get::<InteractionDisabled>(trigger).is_some());
    assert_eq!(bg(&app, trigger), probe_theme().select.trigger.bg.disabled);
    assert_eq!(app.world().get::<TabIndex>(trigger).unwrap().0, -1);
}

#[test]
fn a_select_that_is_disabled_later_and_enabled_again_follows() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    let trigger = trigger(&app, root);

    app.world_mut().entity_mut(root).insert(InteractionDisabled);
    app.update();
    assert!(app.world().get::<InteractionDisabled>(trigger).is_some());
    assert_eq!(bg(&app, trigger), probe_theme().select.trigger.bg.disabled);

    app.world_mut()
        .entity_mut(root)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(app.world().get::<InteractionDisabled>(trigger).is_none());
    assert_eq!(bg(&app, trigger), probe_theme().select.trigger.bg.normal);
    assert_eq!(app.world().get::<TabIndex>(trigger).unwrap().0, 0);
}

#[test]
fn a_disabled_select_does_not_open() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn((fruit_select(), InteractionDisabled))
        .id();
    let trigger = trigger(&app, root);

    menu_event(&mut app, trigger, MenuAction::Toggle);
    menu_event(&mut app, trigger, MenuAction::Open(NavAction::First));

    assert_eq!(visibility(&app, popup(&app, root)), Visibility::Hidden);
}

#[test]
fn disabling_an_open_select_closes_its_list_and_gives_the_focus_back() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let list = list(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);
    app.insert_resource(InputFocus::from_entity(list));

    app.world_mut().entity_mut(root).insert(InteractionDisabled);
    app.update();

    assert_eq!(visibility(&app, popup(&app, root)), Visibility::Hidden);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(trigger));
}

#[test]
fn a_disabled_trigger_is_disabled_for_a_screen_reader() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn((fruit_select(), InteractionDisabled))
        .id();

    app.update();

    assert!(trigger_node(&app, root).is_disabled());
}

#[test]
fn an_option_that_is_disabled_has_a_disabled_row() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(fruit_select_with_banana_disabled())
        .id();

    assert_eq!(disabled_rows(&app, root), [false, true, false]);
}

#[test]
fn options_can_be_given_with_their_disabled_state_at_once() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .options_with_disabled([
                    (Fruit::Apple, text("Apple"), false),
                    (Fruit::Banana, text("Banana"), true),
                    (Fruit::Cherry, text("Cherry"), true),
                ])
                .build(),
        )
        .id();

    assert_eq!(disabled_rows(&app, root), [false, true, true]);
}

#[test]
fn a_disabled_row_stays_gray_where_the_highlight_is() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(fruit_select_with_banana_disabled())
        .id();
    app.update();
    let rows = rows(&app, root);
    let list = list(&app, root);

    app.world_mut()
        .entity_mut(list)
        .insert(ActiveDescendant(Some(rows[1])));
    app.update();

    let style = probe_theme().select;
    assert_eq!(bg(&app, rows[1]), style.option_bg.unselected.disabled);
    assert_eq!(bg(&app, rows[0]), style.option_bg.unselected.normal);
}

#[test]
fn a_disabled_row_is_disabled_for_a_screen_reader() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(fruit_select_with_banana_disabled())
        .id();

    app.update();

    let flags: Vec<bool> = rows(&app, root)
        .iter()
        .map(|row| {
            app.world()
                .get::<AccessibilityNode>(*row)
                .unwrap()
                .is_disabled()
        })
        .collect();
    assert_eq!(flags, [false, true, false]);
}

#[test]
fn a_disabled_row_is_not_a_choice() {
    let mut app = app_that_logs_requests();
    let root = app
        .world_mut()
        .spawn(fruit_select_with_banana_disabled())
        .id();
    let list = list(&app, root);
    let rows = rows(&app, root);

    report_row(&mut app, list, rows[1]);
    assert!(app.world().resource::<Requests>().0.is_empty());

    report_row(&mut app, list, rows[2]);
    assert_eq!(
        app.world().resource::<Requests>().0,
        [(root, Fruit::Cherry)]
    );
}

fn fruit_select_with_banana_disabled() -> impl Bundle {
    select::<Fruit>()
        .placeholder(text("Pick one"))
        .option(Fruit::Apple, text("Apple"))
        .option_disabled(Fruit::Banana, text("Banana"))
        .option(Fruit::Cherry, text("Cherry"))
        .build()
}

fn disabled_rows(app: &App, root: Entity) -> Vec<bool> {
    rows(app, root)
        .iter()
        .map(|row| app.world().get::<InteractionDisabled>(*row).is_some())
        .collect()
}
