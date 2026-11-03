//! Behavior tests for `WindowDefaultsPlugin`: the stack of windows, their layers, and `RaiseWindow`.

use crate::support::test_app;
use bevy::prelude::*;
use bevy_ui_component_prototypes::{
    ActiveWindow, RaiseWindow, WindowDefaultsPlugin, WindowStack, window,
};

fn app_with_defaults(plugin: WindowDefaultsPlugin) -> App {
    let mut app = test_app();
    app.add_plugins(plugin);
    app
}

fn spawn_window(app: &mut App) -> Entity {
    app.world_mut().spawn(window().build()).id()
}

fn layer(app: &App, window: Entity) -> Option<i32> {
    app.world().get::<GlobalZIndex>(window).map(|z| z.0)
}

fn is_active(app: &App, window: Entity) -> bool {
    app.world().get::<ActiveWindow>(window).is_some()
}

#[test]
fn windows_get_layers_in_the_order_they_were_spawned() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b, c] = [(); 3].map(|_| spawn_window(&mut app));
    app.update();
    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
    assert_eq!(layer(&app, c), Some(30));
}

#[test]
fn the_base_and_the_step_come_from_the_plugin() {
    let mut app = app_with_defaults(WindowDefaultsPlugin {
        z_base: 100,
        z_step: 5,
    });
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    assert_eq!(layer(&app, a), Some(100));
    assert_eq!(layer(&app, b), Some(105));
}

#[test]
fn only_the_top_window_is_active() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    assert!(!is_active(&app, a));
    assert!(is_active(&app, b));
}

#[test]
fn raising_a_window_brings_it_to_the_top() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b, c] = [(); 3].map(|_| spawn_window(&mut app));
    app.update();

    app.world_mut().trigger(RaiseWindow { window: a });
    app.update();

    assert_eq!(layer(&app, b), Some(10));
    assert_eq!(layer(&app, c), Some(20));
    assert_eq!(layer(&app, a), Some(30));
    assert!(is_active(&app, a));
    assert!(!is_active(&app, c));
}

#[test]
fn raising_the_top_window_changes_nothing() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    let before = app.world().resource::<WindowStack>().clone();

    app.world_mut().trigger(RaiseWindow { window: b });
    app.update();

    assert_eq!(app.world().resource::<WindowStack>(), &before);
    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
}

#[test]
fn raising_an_entity_that_is_not_a_window_is_ignored() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let a = spawn_window(&mut app);
    let stranger = app.world_mut().spawn_empty().id();
    app.update();

    app.world_mut().trigger(RaiseWindow { window: stranger });
    app.update();

    assert_eq!(layer(&app, stranger), None);
    assert_eq!(layer(&app, a), Some(10));
}

#[test]
fn a_despawned_window_leaves_no_gap() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b, c] = [(); 3].map(|_| spawn_window(&mut app));
    app.update();

    app.world_mut().despawn(b);
    app.update();

    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, c), Some(20));
    assert!(!app.world().resource::<WindowStack>().contains(b));
}

#[test]
fn the_active_window_moves_down_when_the_top_one_is_despawned() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();

    app.world_mut().despawn(b);
    app.update();

    assert!(is_active(&app, a));
}

#[test]
fn a_window_spawned_later_goes_on_the_top() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let a = spawn_window(&mut app);
    app.update();

    let b = spawn_window(&mut app);
    app.update();

    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
    assert!(is_active(&app, b));
    assert!(!is_active(&app, a));
}

#[test]
fn without_the_plugin_nothing_is_applied() {
    let mut app = test_app();
    let a = spawn_window(&mut app);
    app.update();

    app.world_mut().trigger(RaiseWindow { window: a });
    app.update();

    assert_eq!(layer(&app, a), None);
    assert!(!is_active(&app, a));
}
