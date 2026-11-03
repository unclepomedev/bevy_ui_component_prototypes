use super::{
    components::{ActiveWindow, UiWindow, WindowRect},
    limits::WindowLimits,
    requests::{MoveWindow, RaiseWindow},
    stack::WindowStack,
};
use bevy::{picking::events::PointerPress, prelude::*, ui::ComputedNode, ui::UiSystems};

/// Applies the requests made to windows.
///
/// - Keeps the [`WindowStack`]: a new window goes on the top, a despawned one leaves.
/// - Gives each window the `GlobalZIndex` of its place in the stack, and [`ActiveWindow`] to the top one.
/// - Moves a window by the delta of [`MoveWindow`], within its [`WindowLimits`].
/// - Brings a window to the front on [`RaiseWindow`], and asks for it when the pointer presses on the window
///   or on anything in it.
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
            .add_observer(raise_on_press)
            .add_observer(on_move_window)
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

fn on_move_window(
    mv: On<MoveWindow>,
    mut windows: Query<(&mut WindowRect, &WindowLimits, Option<&ChildOf>), With<UiWindow>>,
    areas: Query<&ComputedNode>,
) {
    let Ok((mut rect, limits, parent)) = windows.get_mut(mv.window) else {
        return;
    };
    let area = parent
        .and_then(|parent| areas.get(parent.parent()).ok())
        .map_or(Vec2::ZERO, |node| node.size() * node.inverse_scale_factor());
    let target = Vec2::new(rect.x, rect.y) + mv.delta;
    let position = limits.clamp(target, Vec2::new(rect.width, rect.height), area);
    let moved = WindowRect {
        x: position.x,
        y: position.y,
        ..*rect
    };
    rect.set_if_neq(moved);
}

/// Asks for the window to be raised when the press lands on it or on a descendant. Widgets that stop the
/// propagation of the press still reach this observer: it runs for the entity the press landed on.
fn raise_on_press(
    press: On<PointerPress>,
    windows: Query<(), With<UiWindow>>,
    parents: Query<&ChildOf>,
    stack: Res<WindowStack>,
    mut commands: Commands,
) {
    let target = press.event_target();
    let Some(window) = std::iter::once(target)
        .chain(parents.iter_ancestors(target))
        .find(|entity| windows.contains(*entity))
    else {
        return;
    };
    if stack.top() != Some(window) {
        commands.trigger(RaiseWindow { window });
    }
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
