use crate::window::{
    components::{ActiveWindow, UiWindow},
    stack::WindowStack,
};
use bevy::prelude::*;

/// Puts new windows on the top of the stack, and takes the despawned ones out.
pub(super) fn track_windows(
    added: Query<Entity, Added<UiWindow>>,
    mut removed: RemovedComponents<UiWindow>,
    mut stack: ResMut<WindowStack>,
) {
    for window in removed.read() {
        if stack.contains(window) {
            stack.remove(window);
        }
    }
    for window in &added {
        if !stack.contains(window) {
            stack.push(window);
        }
    }
}

/// Writes the layer of each window, and moves [`ActiveWindow`] to the top one.
pub(super) fn apply_layers(
    stack: Res<WindowStack>,
    windows: Query<(Option<&GlobalZIndex>, Has<ActiveWindow>), With<UiWindow>>,
    mut commands: Commands,
) {
    if !stack.is_changed() {
        return;
    }
    let top = stack.top();
    for (window, layer) in stack.layers() {
        let Ok((current, active)) = windows.get(window) else {
            continue;
        };
        if current.map(|z| z.0) != Some(layer) {
            commands.entity(window).insert(GlobalZIndex(layer));
        }
        let is_top = top == Some(window);
        if is_top && !active {
            commands.entity(window).insert(ActiveWindow);
        } else if !is_top && active {
            commands.entity(window).remove::<ActiveWindow>();
        }
    }
}
