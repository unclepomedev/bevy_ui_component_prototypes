use super::components::{PopupPanel, PopupRoot};
use bevy::{
    input_focus::{AcquireFocus, FocusCause, InputFocus},
    prelude::*,
};

/// A press that reaches the root of a popup found nothing to focus in it. It would go on to clear the
/// focus, and Tab would then leave the modal group. The panel takes the focus instead.
pub(super) fn keep_focus_in_popup(
    mut acquire: On<AcquireFocus>,
    focus: Option<ResMut<InputFocus>>,
    roots: Query<&Children, With<PopupRoot>>,
    panels: Query<(), With<PopupPanel>>,
) {
    let (Some(mut focus), Ok(children)) = (focus, roots.get(acquire.focused_entity)) else {
        return;
    };
    let Some(panel) = children.iter().find(|child| panels.contains(*child)) else {
        return;
    };
    acquire.propagate(false);
    if focus.get() != Some(panel) {
        focus.set(panel, FocusCause::Pressed);
    }
}
