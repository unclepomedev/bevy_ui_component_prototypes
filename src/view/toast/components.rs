use super::variant::ToastVariant;
use crate::{
    style::toast::{ToastMetrics, ToastShape},
    view::pass_through,
};
use bevy::{picking::Pickable, prelude::*};

/// A transient notification. It lives for `fade_in + duration + fade_out` (the fades come from the theme), then despawns itself.
///
/// Setting `elapsed` ahead moves it along its life, which is how to dismiss one early.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
#[require(
    ToastVariant,
    UiTransform,
    Pickable = pass_through(),
    Node = toast_node(&ToastMetrics::default(), &ToastShape::default())
)]
pub struct Toast {
    /// Seconds since the toast appeared.
    pub elapsed: f32,
    /// Seconds the toast stays fully visible. `None` takes the theme's `default_duration`.
    pub duration: Option<f32>,
}

/// The text inside a [`Toast`].
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct ToastLabel;

/// The size and the shape of a toast. Text wraps at `max_width`.
pub(crate) fn toast_node(metrics: &ToastMetrics, shape: &ToastShape) -> Node {
    Node {
        min_width: metrics.min_width,
        max_width: metrics.max_width,
        padding: metrics.padding,
        border: UiRect::all(Val::Px(shape.border_width)),
        border_radius: BorderRadius::all(Val::Px(shape.radius)),
        align_items: AlignItems::Center,
        ..default()
    }
}
