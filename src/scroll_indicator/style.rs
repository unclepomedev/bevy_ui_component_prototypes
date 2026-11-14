use crate::common::color::{BG_PRESSED, BORDER};
use bevy::prelude::*;

/// The colors of a scroll indicator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollIndicatorStyle {
    /// The track the thumb moves along.
    pub track: Color,
    pub thumb: Color,
}

impl Default for ScrollIndicatorStyle {
    fn default() -> Self {
        Self {
            track: BG_PRESSED,
            thumb: BORDER,
        }
    }
}

/// The size and the place of a scroll indicator, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollIndicatorMetrics {
    pub width: f32,
    /// The space between the track and the top, the bottom and the right edge of the node it is in.
    pub inset: f32,
}

impl Default for ScrollIndicatorMetrics {
    fn default() -> Self {
        Self {
            width: 4.0,
            inset: 4.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_can_be_seen() {
        let (style, metrics) = (
            ScrollIndicatorStyle::default(),
            ScrollIndicatorMetrics::default(),
        );
        assert_ne!(style.track, style.thumb);
        assert!(metrics.width > 0.0);
        assert!(metrics.inset >= 0.0);
    }
}
