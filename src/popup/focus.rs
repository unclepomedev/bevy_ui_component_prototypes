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

/// Keeps the input focus inside the front popup: the one with the highest `GlobalZIndex`.
/// Popups on the same layer have no defined order; the one with the greater entity counts as the front one.
pub(super) fn focus_front_popup(
    focus: Option<ResMut<InputFocus>>,
    roots: Query<(Entity, &Children, Option<&GlobalZIndex>), With<PopupRoot>>,
    panels: Query<(), With<PopupPanel>>,
    parents: Query<&ChildOf>,
) {
    let Some(mut focus) = focus else {
        return;
    };
    let Some((front, children, _)) = roots
        .iter()
        .max_by_key(|(entity, _, layer)| (layer.map_or(0, |layer| layer.0), *entity))
    else {
        return;
    };
    if let Some(focused) = focus.get()
        && parents
            .iter_ancestors(focused)
            .any(|ancestor| ancestor == front)
    {
        return;
    }
    if let Some(panel) = children.iter().find(|child| panels.contains(*child)) {
        focus.set(panel, FocusCause::Auto);
    }
}
