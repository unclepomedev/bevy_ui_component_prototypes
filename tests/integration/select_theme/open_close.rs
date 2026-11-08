//! Tests of Opening and closing.

use super::{
    Fruit, focus_state, list, menu_event, popup, rows, spawn_fruit, spawn_fruit_under,
    spawn_preselected, trigger, visibility,
};
use crate::support::test_app;
use bevy::{
    input_focus::{InputFocus, InputFocusVisible, tab_navigation::NavAction},
    prelude::*,
    ui::{ComputedNode, ScrollPosition, UiGlobalTransform},
    ui_widgets::{MenuAction, MenuEvent, MenuFocusState, ScrollAreaPlugin, ScrollIntoView},
};
use bevy_ui_component_prototypes::select::{SelectedIndex, SelectedOption};

#[test]
fn toggling_opens_the_popup_and_toggling_again_closes_it() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let popup = popup(&app, root);

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(visibility(&app, popup), Visibility::Inherited);
    assert_eq!(
        focus_state(&app, popup),
        MenuFocusState::Opening(NavAction::First)
    );

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(visibility(&app, popup), Visibility::Hidden);
    assert_eq!(focus_state(&app, popup), MenuFocusState::Closed);
}

#[test]
fn opening_asks_for_the_end_of_the_list_the_key_names() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);

    menu_event(&mut app, trigger, MenuAction::Open(NavAction::Last));

    assert_eq!(
        focus_state(&app, popup(&app, root)),
        MenuFocusState::Opening(NavAction::Last)
    );
}

#[test]
fn opening_an_open_select_leaves_it_as_it_is() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    menu_event(&mut app, trigger, MenuAction::Open(NavAction::Last));

    menu_event(&mut app, trigger, MenuAction::Open(NavAction::First));

    assert_eq!(
        focus_state(&app, popup(&app, root)),
        MenuFocusState::Opening(NavAction::Last)
    );
}

#[test]
fn close_all_from_the_popup_closes_an_open_select() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let popup = popup(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(visibility(&app, popup), Visibility::Inherited);

    menu_event(&mut app, popup, MenuAction::CloseAll);

    assert_eq!(visibility(&app, popup), Visibility::Hidden);
    assert_eq!(focus_state(&app, popup), MenuFocusState::Closed);
}

#[test]
fn close_all_on_a_closed_select_changes_nothing() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let popup = popup(&app, root);

    menu_event(&mut app, popup, MenuAction::CloseAll);

    assert_eq!(visibility(&app, popup), Visibility::Hidden);
}

#[test]
fn focus_root_puts_the_focus_back_on_the_trigger_and_shows_the_indicator() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let list = list(&app, root);
    app.insert_resource(InputFocus::from_entity(list));
    // A click on the trigger hid it.
    app.insert_resource(InputFocusVisible(false));

    menu_event(&mut app, list, MenuAction::FocusRoot);

    assert_eq!(app.world().resource::<InputFocus>().get(), Some(trigger));
    assert!(app.world().resource::<InputFocusVisible>().0);
}

#[test]
fn without_the_input_focus_plugins_focus_root_does_nothing() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);

    menu_event(&mut app, trigger, MenuAction::FocusRoot);

    assert!(app.world().get_resource::<InputFocus>().is_none());
}

#[derive(Resource, Default)]
struct Seen(Vec<Entity>);

#[test]
fn the_event_stops_at_the_select() {
    let mut app = test_app();
    app.init_resource::<Seen>()
        .add_observer(|event: On<MenuEvent>, mut seen: ResMut<Seen>| {
            seen.0.push(event.event_target());
        });
    let outer = app.world_mut().spawn_empty().id();
    let root = spawn_fruit_under(&mut app, outer);
    let trigger = trigger(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);

    let seen = &app.world().resource::<Seen>().0;
    assert!(seen.contains(&root));
    assert!(!seen.contains(&outer));
}

// ---------------------------------------------------------------------------
// Scrolling to the chosen option
// ---------------------------------------------------------------------------

/// The rows `ScrollIntoView` was sent for. Nothing handles the event in these tests, so only this record remains.
#[derive(Resource, Default)]
struct Scrolled(Vec<Entity>);

fn app_that_records_scrolling() -> App {
    let mut app = test_app();
    app.add_plugins(ScrollAreaPlugin);
    app.init_resource::<Scrolled>();
    app.add_observer(
        |event: On<ScrollIntoView>, mut scrolled: ResMut<Scrolled>| {
            scrolled.0.push(event.entity);
        },
    );
    app
}

fn scrolled(app: &App) -> Vec<Entity> {
    app.world().resource::<Scrolled>().0.clone()
}

fn toggle(app: &mut App, root: Entity) {
    let trigger = trigger(app, root);
    menu_event(app, trigger, MenuAction::Toggle);
}

#[test]
fn opening_a_list_with_a_chosen_option_scrolls_to_its_row() {
    let mut app = app_that_records_scrolling();
    let root = spawn_preselected(&mut app, Fruit::Cherry);
    app.update();
    // There is no layout in the tests, so the sizes are given by hand: the list shows 100 of its 300, and the
    // chosen row lies below the visible part.
    let list_entity = list(&app, root);
    let row = rows(&app, root)[2];
    app.world_mut().entity_mut(list_entity).insert((
        ComputedNode {
            size: Vec2::new(100.0, 100.0),
            content_size: Vec2::new(100.0, 300.0),
            inverse_scale_factor: 1.0,
            ..default()
        },
        UiGlobalTransform::from_translation(Vec2::new(50.0, 50.0)),
    ));
    app.world_mut().entity_mut(row).insert((
        ComputedNode {
            size: Vec2::new(100.0, 30.0),
            inverse_scale_factor: 1.0,
            ..default()
        },
        UiGlobalTransform::from_translation(Vec2::new(50.0, 250.0)),
    ));

    toggle(&mut app, root);

    assert_eq!(scrolled(&app), vec![row]);
    let position = app.world().get::<ScrollPosition>(list_entity).unwrap();
    assert_eq!(position.y, 165.0);
}

#[test]
fn the_list_scrolls_to_the_option_that_is_chosen_by_the_time_it_opens() {
    let mut app = app_that_records_scrolling();
    let root = spawn_fruit(&mut app);
    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Banana,
    });
    app.update();

    toggle(&mut app, root);

    assert_eq!(scrolled(&app), vec![rows(&app, root)[1]]);
}

#[test]
fn opening_a_list_with_nothing_chosen_brings_it_back_to_the_top() {
    let mut app = app_that_records_scrolling();
    let root = spawn_fruit(&mut app);
    let list = list(&app, root);
    app.world_mut()
        .entity_mut(list)
        .insert(ScrollPosition(Vec2::new(0.0, 100.0)));

    toggle(&mut app, root);

    assert!(scrolled(&app).is_empty());
    assert_eq!(
        app.world().get::<ScrollPosition>(list).unwrap().0,
        Vec2::ZERO
    );
}

#[test]
fn a_position_past_the_last_option_is_treated_as_nothing_chosen() {
    let mut app = app_that_records_scrolling();
    let root = spawn_fruit(&mut app);
    let list = list(&app, root);
    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(9) });
    app.world_mut()
        .entity_mut(list)
        .insert(ScrollPosition(Vec2::new(0.0, 100.0)));
    app.update();

    toggle(&mut app, root);

    assert!(scrolled(&app).is_empty());
    assert_eq!(
        app.world().get::<ScrollPosition>(list).unwrap().0,
        Vec2::ZERO
    );
}

#[test]
fn the_list_scrolls_only_when_it_opens() {
    let mut app = app_that_records_scrolling();
    let root = spawn_preselected(&mut app, Fruit::Apple);
    app.update();
    let rows = rows(&app, root);

    toggle(&mut app, root);
    assert_eq!(scrolled(&app), vec![rows[0]]);

    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(1) });
    app.update();
    assert_eq!(scrolled(&app).len(), 1);

    toggle(&mut app, root);
    assert_eq!(scrolled(&app).len(), 1);

    toggle(&mut app, root);
    assert_eq!(scrolled(&app), vec![rows[0], rows[1]]);
}
