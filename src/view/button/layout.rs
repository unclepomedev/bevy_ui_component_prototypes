use crate::style::{
    button::{ButtonMetrics, ButtonShape},
    layout,
};
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
