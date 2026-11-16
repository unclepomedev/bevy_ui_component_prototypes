use super::{components::UiWindow, requests::ToggleMaximizeWindow};
use bevy::{prelude::*, ui_widgets::Activate};

/// Activating the maximize button asks for its window to be maximized, or to go back.
pub(super) fn maximize_on_activate(
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
    commands.trigger(ToggleMaximizeWindow { window });
}
