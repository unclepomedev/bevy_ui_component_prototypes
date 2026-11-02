//! Tests of Opening and closing.

use super::{
    focus_state, list, menu_event, popup, spawn_fruit, spawn_fruit_under, trigger, visibility,
};
use crate::support::test_app;
use bevy::input_focus::tab_navigation::NavAction;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::prelude::*;
use bevy::ui_widgets::{MenuAction, MenuEvent, MenuFocusState};

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
