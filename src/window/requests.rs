use bevy::prelude::*;

/// Asks for `window` to be brought to the front.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct RaiseWindow {
    #[event_target]
    pub window: Entity,
}

/// Asks for `window` to be moved by `delta`, in logical pixels.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MoveWindow {
    #[event_target]
    pub window: Entity,
    pub delta: Vec2,
}

/// Asks for `window` to be closed.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct CloseWindow {
    #[event_target]
    pub window: Entity,
}

/// An edge or a corner of a window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl WindowEdge {
    pub(crate) const ALL: [Self; 8] = [
        Self::Top,
        Self::Bottom,
        Self::Left,
        Self::Right,
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
    ];

    pub(crate) fn has_left(self) -> bool {
        matches!(self, Self::Left | Self::TopLeft | Self::BottomLeft)
    }

    pub(crate) fn has_right(self) -> bool {
        matches!(self, Self::Right | Self::TopRight | Self::BottomRight)
    }

    pub(crate) fn has_top(self) -> bool {
        matches!(self, Self::Top | Self::TopLeft | Self::TopRight)
    }

    pub(crate) fn has_bottom(self) -> bool {
        matches!(self, Self::Bottom | Self::BottomLeft | Self::BottomRight)
    }
}

/// Asks for `edge` of `window` to be moved by `delta`, in logical pixels.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ResizeWindow {
    #[event_target]
    pub window: Entity,
    pub edge: WindowEdge,
    pub delta: Vec2,
}
