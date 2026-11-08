//! Tests of the `GlobalZIndex` of a popup.

use super::spawn_popup;
use crate::{probe::probe_theme, support::test_app};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{GlobalTheme, popup};

fn layer(app: &App, root: Entity) -> Option<i32> {
    app.world().get::<GlobalZIndex>(root).map(|z| z.0)
}

#[test]
fn a_popup_without_a_layer_takes_the_one_of_its_theme_after_one_update() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");

    app.update();

    assert_eq!(layer(&app, root), Some(probe_theme().popup_metrics.z_index));
}

#[test]
fn a_layer_given_when_spawning_is_kept() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn((popup().message("Quit?").build(), GlobalZIndex(5)))
        .id();

    app.update();
    let mut theme = probe_theme();
    theme.popup_metrics.z_index = 999;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    assert_eq!(layer(&app, root), Some(5));
}

#[test]
fn the_layer_of_the_theme_follows_a_change_of_the_theme() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");
    app.update();

    let mut theme = probe_theme();
    theme.popup_metrics.z_index = 999;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    assert_eq!(layer(&app, root), Some(999));
}
