use super::variant::ProgressVariant;
use crate::{style::progress::ProgressGeometry, view::pass_through};
use accesskit::{Node as AccessKitNode, Role};
use bevy::{a11y::AccessibilityNode, picking::Pickable, prelude::*};

/// A themed horizontal bar: a rail and a filled part of it that follows a value.
///
/// It is controlled and has no input: change [`ProgressValue`] (and [`ProgressRange`]) and the look follows.
/// Colors and shape come from the theme; pick a look with [`ProgressVariant`].
/// Text placed next to the bar is not associated with it: give the bar a name with `AccessibleLabel`.
#[derive(Component)]
#[require(
    ProgressValue,
    ProgressRange,
    ProgressVariant,
    AccessibilityNode(AccessKitNode::new(Role::ProgressIndicator)),
    Pickable = pass_through(),
    Node = progress_node(&crate::style::progress::ProgressShape::default().geometry())
)]
pub struct ProgressBar;

/// The current value.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Deref, DerefMut)]
pub struct ProgressValue(pub f32);

/// The values at the empty and the full end.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ProgressRange {
    pub start: f32,
    pub end: f32,
}

impl Default for ProgressRange {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 1.0,
        }
    }
}

impl ProgressRange {
    /// Where `value` lies between the ends, from 0 to 1. A value outside the range is clamped to it.
    pub fn fraction(&self, value: f32) -> f32 {
        let span = self.end - self.start;
        let share = (value - self.start) / span;
        if share.is_finite() {
            share.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// The filled part of a [`ProgressBar`].
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct ProgressFill;

pub(crate) fn progress_node(geometry: &ProgressGeometry) -> Node {
    Node {
        width: percent(100),
        height: px(geometry.height),
        border_radius: BorderRadius::all(px(geometry.radius)),
        ..default()
    }
}

pub(crate) fn fill_node(geometry: &ProgressGeometry) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(0),
        height: percent(100),
        border_radius: BorderRadius::all(px(geometry.radius)),
        ..default()
    }
}
