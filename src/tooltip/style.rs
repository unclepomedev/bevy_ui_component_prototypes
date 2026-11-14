use crate::common::{
    color::{BG_PRESSED, BORDER, TEXT_MAIN},
    spacing::SPACING_SM,
};
use bevy::prelude::*;

/// Border width and corner radius of a tooltip, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipShape {
    pub border_width: f32,
    pub radius: f32,
}

impl Default for TooltipShape {
    fn default() -> Self {
        Self {
            border_width: 1.0,
            radius: 6.0,
        }
    }
}

/// Colors and shape of the tooltips of a theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipStyle {
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub shape: TooltipShape,
}

impl Default for TooltipStyle {
    fn default() -> Self {
        Self {
            bg: BG_PRESSED,
            fg: TEXT_MAIN,
            border: BORDER,
            shape: TooltipShape::default(),
        }
    }
}

/// The footprint and the timing shared by every tooltip of a theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipMetrics {
    /// Text wraps at this width.
    pub max_width: Val,
    pub padding: UiRect,
    pub font_size: FontSize,
    /// The space between the widget and its tooltip, in pixels.
    pub gap: f32,
    /// Seconds the pointer or the focus rests on a widget before its tooltip shows.
    /// A [`Tooltip`](super::Tooltip) can set its own.
    pub delay: f32,
    /// Seconds after a tooltip has hidden during which the next one shows without waiting,
    /// so that moving along a row of widgets does not wait at every one. `0.0` turns it off.
    pub warmup: f32,
    /// How far above its nearest ancestor with a [`GlobalZIndex`] the tooltip sits.
    /// Counted from layer 0 when no ancestor has one.
    pub layer_gap: i32,
}

impl Default for TooltipMetrics {
    fn default() -> Self {
        Self {
            max_width: Val::Px(320.0),
            padding: UiRect::axes(SPACING_SM, Val::Px(6.0)),
            font_size: FontSize::Px(16.0),
            gap: 6.0,
            delay: 0.5,
            warmup: 0.3,
            layer_gap: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_timing_waits_before_it_shows() {
        let metrics = TooltipMetrics::default();
        assert!(metrics.delay > 0.0);
        assert!(metrics.warmup >= 0.0);
    }

    #[test]
    fn the_default_text_can_be_read_on_its_background() {
        let style = TooltipStyle::default();
        assert_ne!(style.bg, style.fg);
    }
}
