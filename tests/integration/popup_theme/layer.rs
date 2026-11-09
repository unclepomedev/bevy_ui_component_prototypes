//! Tests of the `GlobalZIndex` of a popup.

use super::spawn_popup;
use crate::support::test_app;
use bevy::prelude::*;
use bevy_ui_component_prototypes::popup::popup;

fn layer(app: &App, root: Entity) -> Option<i32> {
    app.world().get::<GlobalZIndex>(root).map(|z| z.0)
}

#[test]
fn a_popup_without_a_layer_is_at_the_default_one_when_spawned() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");

    assert_eq!(layer(&app, root), Some(300));
}

#[test]
fn a_layer_given_when_spawning_is_kept() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn((popup().message("Quit?").build(), GlobalZIndex(5)))
        .id();

    app.update();

    assert_eq!(layer(&app, root), Some(5));
}
