use super::style::{ButtonMetrics, ButtonShape};
use crate::common::layout;
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
    if node.height != want.height {
        node.height = want.height;
    }
    if node.padding != want.padding {
        node.padding = want.padding;
    }
    if node.column_gap != want.column_gap {
        node.column_gap = want.column_gap;
    }
    if node.border != want.border {
        node.border = want.border;
    }
    if node.border_radius != want.border_radius {
        node.border_radius = want.border_radius;
    }
}
