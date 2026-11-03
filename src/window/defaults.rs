mod close;
mod layers;
mod raise;
mod rect;

use self::{
    close::on_close_window,
    layers::{apply_layers, track_windows},
    raise::{on_raise_window, raise_on_focus, raise_on_press},
    rect::{on_move_window, on_resize_window},
};
use super::stack::WindowStack;
use bevy::{input_focus::InputFocus, prelude::*, ui::UiSystems};

/// Applies the requests made to windows.
///
/// - Keeps the [`WindowStack`]: a new window goes on the top, a despawned one leaves.
/// - Gives each window the `GlobalZIndex` of its place in the stack, and [`ActiveWindow`](super::ActiveWindow) to the top one.
/// - Moves a window by the delta of [`MoveWindow`](super::MoveWindow), within its [`WindowLimits`](super::WindowLimits).
/// - Moves an edge of a window by the delta of [`ResizeWindow`](super::ResizeWindow), within its [`WindowLimits`](super::WindowLimits).
/// - Despawns a window on [`CloseWindow`](super::CloseWindow).
/// - Brings a window to the front on [`RaiseWindow`](super::RaiseWindow), and asks for it when the pointer presses on the window
///   or on anything in it, and when the input focus moves to the window or to anything in it.
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
            .add_observer(on_resize_window)
            .add_observer(on_close_window)
            .add_systems(
                Update,
                raise_on_focus.run_if(resource_exists_and_changed::<InputFocus>),
            )
            .add_systems(
                PostUpdate,
                (track_windows, apply_layers)
                    .chain()
                    .before(UiSystems::Prepare),
            );
    }
}
