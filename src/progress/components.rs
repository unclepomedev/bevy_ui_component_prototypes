use super::{
    style::{ProgressGeometry, ProgressShape},
    variant::ProgressVariant,
};
use crate::common::{bar::BarSegments, picking::pass_through};
use bevy::{picking::Pickable, prelude::*};

/// A themed horizontal bar: a rail and a filled part of it that follows a value.
///
/// It is controlled and has no input: change [`ProgressValue`] (and [`ProgressRange`]) and the look follows.
/// Colors and shape come from the theme; pick a look with [`ProgressVariant`].
#[derive(Component)]
#[require(
    ProgressValue,
    ProgressRange,
    ProgressVariant,
    Pickable = pass_through(),
    Node = progress_node(&ProgressShape::default().geometry())
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

/// Draws this bar as blocks of its own, whatever the theme says: the theme's
/// [`ProgressShape::segments`] is for every bar of a variant, while this is for one bar,
/// for example one whose number of blocks is only known at run time.
///
/// `None` leaves the choice to the theme, as does having no such component. Fewer than two blocks
/// draw a plain bar, also under a theme that draws blocks.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressSegments(pub Option<BarSegments>);

impl ProgressSegments {
    /// `count` blocks with `gap` between them, as a share of one pitch. See [`BarSegments::gap`].
    pub fn blocks(count: u32, gap: f32) -> Self {
        Self(Some(BarSegments { count, gap }))
    }

    /// Leaves the choice to the theme: the same as having no [`ProgressSegments`].
    ///
    /// A bar built with [`progress_bar`](super::progress_bar) always has the component, and this is
    /// its value unless `segments` was called. (A bundle cannot hold an optional component, so
    /// the choice is inside it.)
    pub const THEME: Self = Self(None);
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
