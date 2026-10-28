use super::variant::ToastVariant;
use crate::{
    style::toast::{ToastMetrics, ToastShape},
    view::pass_through,
};
use bevy::{picking::Pickable, prelude::*};

const DEFAULT_MAX: usize = 5;

/// Which end of a [`ToastHost`] the newest toast takes, and so which way the older ones move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastStack {
    /// The newest toast is at the bottom; older ones are pushed up. For a host anchored at the bottom.
    #[default]
    NewestAtBottom,
    /// The newest toast is at the top; older ones are pushed down. For a host anchored at the top.
    NewestAtTop,
}

impl ToastStack {
    /// The last child is the newest one, so it has to be at the end of the stack that gets the newest.
    pub(crate) fn direction(self) -> FlexDirection {
        match self {
            Self::NewestAtBottom => FlexDirection::Column,
            Self::NewestAtTop => FlexDirection::ColumnReverse,
        }
    }
}

/// Where toasts stack up. Toasts are its children: spawn them with `ChildOf(host)`.
///
/// Give the host a `Node` that places it. `align_items` of that `Node` decides how the toasts line up across the stack.
/// The `flex_direction` of the `Node` is written by the library, so do not set it yourself.
///
/// The host and its toasts let the pointer through.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
#[require(Node, Pickable = pass_through())]
pub struct ToastHost {
    /// How many toasts the host shows at once.
    pub max: usize,
    pub stack: ToastStack,
    /// Where a new toast slides in from, relative to its resting place.
    pub slide: Val2,
}

impl Default for ToastHost {
    fn default() -> Self {
        Self {
            max: DEFAULT_MAX,
            stack: ToastStack::default(),
            slide: Val2::ZERO,
        }
    }
}

/// A transient notification. Spawn it as a child of a [`ToastHost`].
/// It lives for `fade_in + duration + fade_out` (the fades come from the theme), then despawns itself.
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
