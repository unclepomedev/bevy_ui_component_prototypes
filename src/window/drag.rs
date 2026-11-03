use super::requests::MoveWindow;
use bevy::{
    picking::{events::PointerDrag, pointer::PointerButton},
    prelude::*,
};

/// Observes the title bar: dragging it with the primary button asks for the window to be moved.
/// A drag that began on a part of the bar, such as the close button, does not.
pub(super) fn drag_title_bar(
    drag: On<PointerDrag>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let bar = drag.event_target();
    if drag.button != PointerButton::Primary || drag.original_event_target() != bar {
        return;
    }
    let Ok(parent) = parents.get(bar) else {
        return;
    };
    commands.trigger(MoveWindow {
        window: parent.parent(),
        delta: drag.delta,
    });
}
