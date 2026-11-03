use super::style::{WindowMetrics, WindowShape, WindowSide, WindowStyle};
use bevy::prelude::*;

/// A themed window: a frame with a title bar, a close button, and an area for content.
///
/// Put the content in the entity marked [`WindowContent`]: `window().content_bundle(...)` adds a bundle to it.
/// The position and the size are those of [`WindowRect`].
///
/// A window is placed in the node it is a child of, with absolute positioning: make that node as large as
/// the area the windows may be in.
#[derive(Component)]
#[require(
    WindowRect,
    BackgroundColor = BackgroundColor(Color::NONE),
    BorderColor = BorderColor::all(Color::NONE),
    Node = window_node(&WindowShape::default())
)]
pub struct UiWindow;

/// Where a [`UiWindow`] is in its parent, and how large. In logical pixels, from the top left of the parent.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct WindowRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WindowRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl Default for WindowRect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 320.0, 240.0)
    }
}

/// The area of a [`UiWindow`] below its title bar. It clips what is in it.
#[derive(Component, Clone, Copy, Default)]
pub struct WindowContent;

/// The text of the title. The bundle given to `title` is added to this entity.
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct WindowTitle;

/// The parts of a window that a style system writes to.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowPart {
    TitleBar,
    Close,
    /// The square that the cross is drawn in; the cross is two bars across it.
    GlyphBox,
    GlyphBar,
}

/// The frame. The position and the size are written from [`WindowRect`].
pub(crate) fn window_node(shape: &WindowShape) -> Node {
    Node {
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Column,
        border: UiRect::all(px(shape.border_width)),
        border_radius: BorderRadius::all(px(shape.radius)),
        ..default()
    }
}

/// The close button comes first among the children, and the title second. Reversing the row puts the
/// close button at the right.
pub(crate) fn title_bar_node(metrics: &WindowMetrics, side: WindowSide) -> Node {
    Node {
        height: px(metrics.title_height),
        padding: metrics.title_padding,
        column_gap: px(metrics.title_gap),
        align_items: AlignItems::Center,
        flex_direction: match side {
            WindowSide::Left => FlexDirection::Row,
            WindowSide::Right => FlexDirection::RowReverse,
        },
        flex_shrink: 0.0,
        ..default()
    }
}

pub(crate) fn close_node(metrics: &WindowMetrics, style: &WindowStyle) -> Node {
    Node {
        width: px(metrics.close_width),
        height: px(metrics.close_height),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(style.close.radius(metrics))),
        flex_shrink: 0.0,
        ..default()
    }
}

pub(crate) fn glyph_box_node(metrics: &WindowMetrics) -> Node {
    Node {
        width: px(metrics.glyph_size),
        height: px(metrics.glyph_size),
        ..default()
    }
}

/// A bar across the whole glyph box, in its middle; turned by the transform into one stroke of the cross.
pub(crate) fn glyph_bar_node(metrics: &WindowMetrics) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px((metrics.glyph_size - metrics.glyph_stroke) / 2.0),
        width: px(metrics.glyph_size),
        height: px(metrics.glyph_stroke),
        ..default()
    }
}

pub(crate) fn glyph_bar_transform(degrees: f32) -> UiTransform {
    UiTransform {
        rotation: Rot2::degrees(degrees),
        ..default()
    }
}

/// The title takes the space the close button leaves, and is cut off at its end.
pub(crate) fn title_node() -> Node {
    Node {
        flex_grow: 1.0,
        min_width: px(0),
        overflow: Overflow::clip_x(),
        ..default()
    }
}

pub(crate) fn content_node() -> Node {
    Node {
        flex_grow: 1.0,
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(),
        ..default()
    }
}
