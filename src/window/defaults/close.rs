use crate::window::{components::UiWindow, requests::CloseWindow};
use bevy::prelude::*;

pub(super) fn on_close_window(
    close: On<CloseWindow>,
    windows: Query<(), With<UiWindow>>,
    mut commands: Commands,
) {
    if windows.contains(close.window) {
        commands.entity(close.window).despawn();
    }
}
