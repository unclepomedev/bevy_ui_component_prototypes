use super::{
    bar::{BarSegments, corner_radius},
    color::{AFFIRM_NORMAL, BG_NORMAL, DANGER_NORMAL, PRIMARY_NORMAL},
};
use bevy::prelude::*;
use std::collections::HashMap;

/// The size and the corners of a progress bar, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressShape {
    pub height: f32,
    /// Fully rounded ends when `true`, square corners when `false`.
    pub rounded: bool,
    /// Draws the rail and the fill as blocks. A block is lit once the value has reached its end.
    pub segments: Option<BarSegments>,
}

impl Default for ProgressShape {
    fn default() -> Self {
        Self {
            height: 12.0,
            rounded: true,
            segments: None,
        }
    }
}

/// Where the parts of a progress bar go, resolved from a [`ProgressShape`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProgressGeometry {
    pub(crate) height: f32,
    pub(crate) radius: f32,
    pub(crate) segments: Option<BarSegments>,
}

impl ProgressShape {
    pub(crate) fn geometry(&self) -> ProgressGeometry {
        ProgressGeometry {
            height: self.height,
            radius: corner_radius(self.height, self.rounded),
            segments: BarSegments::usable(self.segments),
        }
    }
}

/// Colors and shape of one kind of progress bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressStyle {
    pub rail: Color,
    pub fill: Color,
    pub shape: ProgressShape,
}

/// The name of a progress bar variant. Define one as a constant and use it for both the theme and the bars.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProgressVariantId(&'static str);

impl ProgressVariantId {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    // The names of `ProgressStyles::default()`.
    pub const PRIMARY: Self = Self::new("primary");
    pub const AFFIRMATIVE: Self = Self::new("affirmative");
    pub const DANGER: Self = Self::new("danger");
}

#[derive(Clone, Debug)]
pub struct ProgressStyles {
    /// Used by `ProgressVariant::Default`, and by a name that has no entry in `named`.
    pub base: ProgressStyle,
    pub named: HashMap<ProgressVariantId, ProgressStyle>,
}

impl Default for ProgressStyles {
    fn default() -> Self {
        let bar = |fill: Color| ProgressStyle {
            rail: BG_NORMAL,
            fill,
            shape: ProgressShape::default(),
        };
        let affirmative = bar(AFFIRM_NORMAL);
        Self {
            base: affirmative,
            named: HashMap::from([
                (ProgressVariantId::AFFIRMATIVE, affirmative),
                (ProgressVariantId::PRIMARY, bar(PRIMARY_NORMAL)),
                (ProgressVariantId::DANGER, bar(DANGER_NORMAL)),
            ]),
        }
    }
}
