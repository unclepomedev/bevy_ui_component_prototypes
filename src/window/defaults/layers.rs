use crate::window::{
    components::{ActiveWindow, Maximized, UiWindow},
    stack::WindowStack,
};
use bevy::{ecs::query::QueryData, prelude::*};

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

/// What `apply_layers` reads of a window.
#[derive(QueryData)]
pub(super) struct Layered {
    layer: Option<&'static GlobalZIndex>,
    active: Has<ActiveWindow>,
    maximized: Has<Maximized>,
}

/// Writes the layer of each window, and moves [`ActiveWindow`] to the top one.
/// A window that is maximized has a layer of its own, if the plugin was given one.
pub(super) fn apply_layers(
    stack: Res<WindowStack>,
    windows: Query<Layered, With<UiWindow>>,
    maximized: Query<(), (With<UiWindow>, Added<Maximized>)>,
    mut restored: RemovedComponents<Maximized>,
    mut commands: Commands,
) {
    // Read the removals in any case: an unread one comes again in the next frame.
    let restored = restored.read().count() > 0;
    if !stack.is_changed() && maximized.is_empty() && !restored {
        return;
    }
    let top = stack.top();
    let is_maximized = |window| windows.get(window).is_ok_and(|w| w.maximized);
    for (window, layer) in stack.layers_with(is_maximized) {
        let Ok(LayeredItem {
            layer: current,
            active,
            ..
        }) = windows.get(window)
        else {
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
