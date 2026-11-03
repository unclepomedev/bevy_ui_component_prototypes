//! Raising a window: by the pointer and by the focus.

use super::*;

#[test]
fn pressing_on_a_window_brings_it_to_the_front() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();

    press(&mut app, a);

    assert_eq!(layer(&app, a), Some(20));
    assert_eq!(layer(&app, b), Some(10));
    assert!(is_active(&app, a));
}

#[test]
fn pressing_on_something_inside_a_window_brings_the_window_to_the_front() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    let content = content_of(&app, a);
    let inside = app.world_mut().spawn(ChildOf(content)).id();

    press(&mut app, inside);

    assert_eq!(layer(&app, a), Some(20));
    assert_eq!(layer(&app, b), Some(10));
}

#[test]
fn pressing_outside_the_windows_changes_nothing() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    let outside = app.world_mut().spawn_empty().id();
    app.update();

    press(&mut app, outside);

    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
}

#[test]
fn moving_the_focus_into_a_window_brings_it_to_the_front() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    let content = content_of(&app, a);
    let inside = app.world_mut().spawn(ChildOf(content)).id();

    focus(&mut app, inside, true);
    app.update();

    assert_eq!(layer(&app, a), Some(20));
    assert_eq!(layer(&app, b), Some(10));
    assert!(is_active(&app, a));
}

#[test]
fn focus_outside_the_windows_changes_nothing() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    let outside = app.world_mut().spawn_empty().id();
    app.update();

    focus(&mut app, outside, true);
    app.update();

    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
}
