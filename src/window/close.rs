use super::{components::UiWindow, requests::CloseWindow};
use bevy::{prelude::*, ui_widgets::Activate};

/// Observes the close button: activating it asks for the window it is in to be closed.
pub(super) fn close_on_activate(
    activate: On<Activate>,
    windows: Query<(), With<UiWindow>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let Some(window) = parents
        .iter_ancestors(activate.entity)
        .find(|entity| windows.contains(*entity))
    else {
        return;
    };
    commands.trigger(CloseWindow { window });
}
