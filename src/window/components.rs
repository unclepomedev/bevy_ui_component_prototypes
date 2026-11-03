use super::{
    limits::WindowLimits,
    requests::WindowEdge,
    style::{WindowMetrics, WindowShape, WindowSide, WindowStyle},
};
use bevy::prelude::*;

/// A themed window: a frame with a title bar, a close button, and an area for content.
///
/// Put the content in the entity marked [`WindowContent`]: `window().content_bundle(...)` adds a bundle to it.
/// The position and the size are those of [`WindowRect`]. Dragging the title bar asks for the window to be
/// moved ([`MoveWindow`](super::MoveWindow)), within [`WindowLimits`]. Activating the close button asks for it
/// to be closed ([`CloseWindow`](super::CloseWindow)), and dragging the frame asks for it to be resized
/// ([`ResizeWindow`](super::ResizeWindow)).
///
/// A window is placed in the node it is a child of, with absolute positioning: make that node as large as
/// the area the windows may be in.
#[derive(Component)]
#[require(
    WindowRect,
    WindowLimits,
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

/// Marks the window in front of the others. At most one window has it.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActiveWindow;

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
    /// A strip along the frame that is dragged to resize the window; it has a [`WindowGrip`].
    Grip,
}

/// Marks an invisible strip along the frame of a [`UiWindow`] that resizes the window at the edge or the
/// corner it holds when it is dragged.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowGrip {
    pub edge: WindowEdge,
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
/// close button at the right. The top corners follow the inside of the frame, so that the bar does not stick
/// out of them.
pub(crate) fn title_bar_node(
    metrics: &WindowMetrics,
    shape: &WindowShape,
    side: WindowSide,
) -> Node {
    Node {
        border_radius: BorderRadius::top(px(shape.inner_radius())),
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

/// An invisible strip centered on the outer edge of the frame, or a square centered on a corner.
/// The positions are from the padding box of the window, so the border is counted in.
pub(crate) fn grip_node(edge: WindowEdge, metrics: &WindowMetrics, shape: &WindowShape) -> Node {
    let grip = metrics.resize_grip;
    // Where the near side of a strip is, and where the strips of the sides start: past the corners.
    let near = px(-(shape.border_width + grip / 2.0));
    let span = px(grip / 2.0 - shape.border_width);
    let mut node = Node {
        position_type: PositionType::Absolute,
        ..default()
    };
    if edge.has_left() {
        node.left = near;
        node.width = px(grip);
    } else if edge.has_right() {
        node.right = near;
        node.width = px(grip);
    } else {
        node.left = span;
        node.right = span;
    }
    if edge.has_top() {
        node.top = near;
        node.height = px(grip);
    } else if edge.has_bottom() {
        node.bottom = near;
        node.height = px(grip);
    } else {
        node.top = span;
        node.bottom = span;
    }
    node
}

/// The box of the title. It takes the space the close button leaves, and clips the title at its end:
/// the clipping of a node is for what is in it, not for the node itself.
pub(crate) fn title_box_node() -> Node {
    Node {
        flex_grow: 1.0,
        min_width: px(0),
        overflow: Overflow::clip_x(),
        ..default()
    }
}

/// The title fills its box; a text that is longer goes on past the end of the box, where it is clipped.
pub(crate) fn title_node() -> Node {
    Node {
        width: percent(100),
        ..default()
    }
}

/// The area fills what the title bar leaves of the window. It can be smaller than its content: `min_width`
/// and `min_height` are zero, otherwise a flex item is as large as its content at least.
pub(crate) fn content_node() -> Node {
    Node {
        flex_grow: 1.0,
        min_width: px(0),
        min_height: px(0),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(),
        ..default()
    }
}
