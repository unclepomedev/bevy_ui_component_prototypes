use super::style::{ButtonMetrics, ButtonShape};
use crate::common::layout::{self, write_changed};
use bevy::prelude::*;

/// The layout of a button: what the size and the shape decide.
pub(crate) fn button_node(metrics: &ButtonMetrics, shape: &ButtonShape) -> Node {
    Node {
        height: metrics.height,
        padding: metrics.padding,
        column_gap: metrics.column_gap,
        border: UiRect::all(Val::Px(shape.border_width)),
        border_radius: BorderRadius::all(Val::Px(shape.radius)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..layout::row_center()
    }
}

/// Writes what `button_node` decides, only where it changed.
pub(crate) fn write_button_layout(node: &mut Mut<Node>, want: &Node) {
    write_changed!(
        node,
        want,
        height,
        padding,
        column_gap,
        border,
        border_radius
    );
}
