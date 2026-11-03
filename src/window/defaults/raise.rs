use crate::window::{components::UiWindow, requests::RaiseWindow, stack::WindowStack};
use bevy::{input_focus::InputFocus, picking::events::PointerPress, prelude::*};

pub(super) fn on_raise_window(raise: On<RaiseWindow>, mut stack: ResMut<WindowStack>) {
    let window = raise.window;
    if stack.top() == Some(window) || !stack.contains(window) {
        return;
    }
    stack.raise(window);
}

/// Asks for the window to be raised when the focus moves to it or to a descendant.
pub(super) fn raise_on_focus(
    focus: Res<InputFocus>,
    windows: Query<(), With<UiWindow>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let Some(focused) = focus.get() else {
        return;
    };
    if let Some(window) = std::iter::once(focused)
        .chain(parents.iter_ancestors(focused))
        .find(|entity| windows.contains(*entity))
    {
        commands.trigger(RaiseWindow { window });
    }
}

/// Asks for the window to be raised when the press lands on it or on a descendant. Widgets that stop the
/// propagation of the press still reach this observer: it runs for the entity the press landed on.
pub(super) fn raise_on_press(
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
