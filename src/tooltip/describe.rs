use super::components::Tooltip;
use bevy::{a11y::AccessibilityNode, prelude::*};

/// Makes the text of a tooltip the description of its widget, where the widget has an accessibility node.
pub(super) fn describe_targets(mut targets: TooltipTargetChanged) {
    for (tooltip, mut node) in &mut targets {
        if node.0.description() != Some(tooltip.text.as_str()) {
            node.0.set_description(tooltip.text.as_str());
        }
    }
}

/// Takes the description back when the [`Tooltip`] goes, so that a screen reader stops reading it.
pub(super) fn clear_description(
    removed: On<Remove<Tooltip>>,
    tooltips: Query<&Tooltip>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    let (Ok(tooltip), Ok(mut node)) = (tooltips.get(removed.entity), nodes.get_mut(removed.entity))
    else {
        return;
    };
    if node.0.description() == Some(tooltip.text.as_str()) {
        node.0.clear_description();
    }
}

type TooltipTargetChanged<'w, 's> = Query<
    'w,
    's,
    (&'static Tooltip, &'static mut AccessibilityNode),
    Or<(Changed<Tooltip>, Changed<AccessibilityNode>)>,
>;
