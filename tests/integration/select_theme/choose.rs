//! Tests of Choosing.

use super::{
    Fruit, Requests, app_that_logs_requests, focus_state, index, list, menu_event, popup,
    report_row, rows, selected_rows, spawn_fruit, spawn_fruit_under, trigger, visibility,
};
use crate::support::test_app;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::prelude::*;
use bevy::ui_widgets::{MenuAction, MenuFocusState, ReselectListRow};

#[test]
fn a_row_the_list_reports_becomes_one_request_for_its_value() {
    let mut app = app_that_logs_requests();
    // Inside another entity, so that an event that travels up would be seen more than once.
    let outer = app.world_mut().spawn_empty().id();
    let root = spawn_fruit_under(&mut app, outer);
    let list = list(&app, root);
    let rows = rows(&app, root);

    report_row(&mut app, list, rows[1]);
    report_row(&mut app, list, rows[2]);

    assert_eq!(
        app.world().resource::<Requests>().0,
        [(root, Fruit::Banana), (root, Fruit::Cherry)]
    );
}

#[test]
fn a_request_never_changes_the_choice_by_itself() {
    let mut app = app_that_logs_requests();
    let root = spawn_fruit(&mut app);
    let list = list(&app, root);
    let row = rows(&app, root)[2];

    report_row(&mut app, list, row);

    assert_eq!(index(&app, root), None);
    assert_eq!(selected_rows(&app, root), [false, false, false]);
}

#[test]
fn a_change_that_does_not_come_from_a_list_of_a_select_is_ignored() {
    let mut app = app_that_logs_requests();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let row = rows(&app, root)[0];

    report_row(&mut app, trigger, row);

    assert!(app.world().resource::<Requests>().0.is_empty());
}

#[test]
fn an_entity_that_is_not_a_row_of_the_list_is_ignored() {
    let mut app = app_that_logs_requests();
    let root = spawn_fruit(&mut app);
    let list = list(&app, root);
    let trigger = trigger(&app, root);

    report_row(&mut app, list, trigger);

    assert!(app.world().resource::<Requests>().0.is_empty());
}

#[test]
fn choosing_an_option_closes_the_list_and_gives_the_focus_back_quietly() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let list = list(&app, root);
    let popup = popup(&app, root);
    let row = rows(&app, root)[1];
    menu_event(&mut app, trigger, MenuAction::Toggle);
    app.insert_resource(InputFocus::from_entity(list));
    // A click hid it.
    app.insert_resource(InputFocusVisible(false));

    report_row(&mut app, list, row);

    assert_eq!(visibility(&app, popup), Visibility::Hidden);
    assert_eq!(focus_state(&app, popup), MenuFocusState::Closed);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(trigger));
    assert!(!app.world().resource::<InputFocusVisible>().0);
}

#[test]
fn choosing_the_chosen_option_again_closes_the_list_too() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let list = list(&app, root);
    let popup = popup(&app, root);
    let row = rows(&app, root)[0];
    menu_event(&mut app, trigger, MenuAction::Toggle);
    app.insert_resource(InputFocus::from_entity(list));

    app.world_mut()
        .trigger(ReselectListRow { source: list, row });
    app.update();

    assert_eq!(visibility(&app, popup), Visibility::Hidden);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(trigger));
}

#[test]
fn a_choice_made_while_the_list_is_closed_leaves_the_focus_alone() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let list = list(&app, root);
    let row = rows(&app, root)[0];
    app.insert_resource(InputFocus::from_entity(list));

    report_row(&mut app, list, row);

    assert_eq!(app.world().resource::<InputFocus>().get(), Some(list));
}
