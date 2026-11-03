use super::{components::WindowGrip, requests::ResizeWindow};
use bevy::{
    picking::{events::PointerDrag, pointer::PointerButton},
    prelude::*,
};

/// Observes a grip: dragging it with the primary button asks for its edge of the window to be moved.
pub(super) fn drag_grip(
    drag: On<PointerDrag>,
    grips: Query<&WindowGrip>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let grip = drag.event_target();
    if drag.button != PointerButton::Primary || drag.original_event_target() != grip {
        return;
    }
    let (Ok(WindowGrip(edge)), Ok(parent)) = (grips.get(grip), parents.get(grip)) else {
        return;
    };
    commands.trigger(ResizeWindow {
        window: parent.parent(),
        edge: *edge,
        delta: drag.delta,
    });
}
