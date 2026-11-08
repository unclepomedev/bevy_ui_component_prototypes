//! Tests of the limit of a host.

use super::{alive, elapsed, run, set_elapsed, timed_app};
use bevy::prelude::*;
use bevy_ui_component_prototypes::toast::{toast, toast_host};

#[test]
fn a_host_keeps_its_newest_toasts_and_fades_the_oldest_out_early() {
    let mut app = timed_app(0.05);
    let host = app.world_mut().spawn(toast_host().max(2).build()).id();
    let oldest = spawn_toast_under(&mut app, host, "1");
    let middle = spawn_toast_under(&mut app, host, "2");
    let newest = spawn_toast_under(&mut app, host, "3");

    run(&mut app, 1);

    // The fade-out of the probe theme starts at 2.5 seconds.
    assert!(elapsed(&app, oldest) >= 2.5);
    assert!(elapsed(&app, middle) < 1.0);
    assert!(elapsed(&app, newest) < 1.0);

    run(&mut app, 6);
    assert!(!alive(&app, oldest));
    assert!(alive(&app, middle));
    assert!(alive(&app, newest));
}

#[test]
fn a_host_below_its_limit_leaves_its_toasts_alone() {
    let mut app = timed_app(0.05);
    let host = app.world_mut().spawn(toast_host().max(3).build()).id();
    let first = spawn_toast_under(&mut app, host, "1");
    let second = spawn_toast_under(&mut app, host, "2");

    run(&mut app, 1);

    assert!(elapsed(&app, first) < 1.0);
    assert!(elapsed(&app, second) < 1.0);
}

#[test]
fn a_toast_that_is_already_fading_does_not_count() {
    let mut app = timed_app(0.05);
    let host = app.world_mut().spawn(toast_host().max(1).build()).id();
    let fading = spawn_toast_under(&mut app, host, "1");
    set_elapsed(&mut app, fading, 2.6);
    let new = spawn_toast_under(&mut app, host, "2");

    run(&mut app, 1);

    assert!(elapsed(&app, new) < 1.0);
    assert!(alive(&app, fading));
}

#[test]
fn the_limit_is_counted_for_each_host() {
    let mut app = timed_app(0.05);
    let first_host = app.world_mut().spawn(toast_host().max(1).build()).id();
    let second_host = app.world_mut().spawn(toast_host().max(1).build()).id();
    let first = spawn_toast_under(&mut app, first_host, "1");
    let second = spawn_toast_under(&mut app, second_host, "2");

    run(&mut app, 1);

    assert!(elapsed(&app, first) < 1.0);
    assert!(elapsed(&app, second) < 1.0);
}

#[test]
fn children_of_the_host_that_are_not_toasts_are_left_alone() {
    let mut app = timed_app(0.05);
    let host = app.world_mut().spawn(toast_host().max(1).build()).id();
    app.world_mut().spawn(ChildOf(host));
    let toast = spawn_toast_under(&mut app, host, "1");

    run(&mut app, 1);

    assert!(elapsed(&app, toast) < 1.0);
}

fn spawn_toast_under(app: &mut App, host: Entity, text: &str) -> Entity {
    app.world_mut()
        .spawn((toast().text(text).build(), ChildOf(host)))
        .id()
}
