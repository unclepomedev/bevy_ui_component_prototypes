use super::{
    components::{ActiveWindow, UiWindow},
    requests::RaiseWindow,
    stack::WindowStack,
};
use bevy::{prelude::*, ui::UiSystems};

/// Applies the requests made to windows.
///
/// - Keeps the [`WindowStack`]: a new window goes on the top, a despawned one leaves.
/// - Gives each window the `GlobalZIndex` of its place in the stack, and [`ActiveWindow`] to the top one.
/// - Brings a window to the front on [`RaiseWindow`].
pub struct WindowDefaultsPlugin {
    /// The `GlobalZIndex` of the bottom window.
    pub z_base: i32,
    /// The difference in `GlobalZIndex` between a window and the one above it.
    pub z_step: i32,
}

impl Default for WindowDefaultsPlugin {
    fn default() -> Self {
        Self {
            z_base: 10,
            z_step: 10,
        }
    }
}

impl Plugin for WindowDefaultsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowStack::new(self.z_base, self.z_step))
            .add_observer(on_raise_window)
            .add_systems(
                PostUpdate,
                (track_windows, apply_layers)
                    .chain()
                    .before(UiSystems::Prepare),
            );
    }
}

fn on_raise_window(raise: On<RaiseWindow>, mut stack: ResMut<WindowStack>) {
    let window = raise.window;
    if stack.top() == Some(window) || !stack.contains(window) {
        return;
    }
    stack.raise(window);
}

/// Puts new windows on the top of the stack, and takes the despawned ones out.
fn track_windows(
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
fn apply_layers(
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
