use super::{
    color::{AFFIRM_NORMAL, BG_NORMAL, BORDER, DANGER_NORMAL, TEXT_MAIN},
    font_size::FONT_SIZE_MD,
    spacing::{SPACING_LG, SPACING_MD, SPACING_SM},
};
use bevy::prelude::*;
use std::collections::HashMap;

/// Border width and corner radius of a toast, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToastShape {
    pub border_width: f32,
    pub radius: f32,
}

impl Default for ToastShape {
    fn default() -> Self {
        Self {
            border_width: 0.0,
            radius: 8.0,
        }
    }
}

/// Colors and shape of one kind of toast. A toast is not interactive, so each color is a single value.
/// The alpha of a color is kept: the fade multiplies it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToastStyle {
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub shape: ToastShape,
}

/// The name of a toast variant. Define one as a constant and use it for both the theme and the toasts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastVariantId(&'static str);

impl ToastVariantId {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    // The names of `ToastStyles::default()`.
    pub const INFO: Self = Self::new("info");
    pub const SUCCESS: Self = Self::new("success");
    pub const DANGER: Self = Self::new("danger");
}

#[derive(Clone, Debug)]
pub struct ToastStyles {
    /// Used by `ToastVariant::Default`, and by a name that has no entry in `named`.
    pub base: ToastStyle,
    pub named: HashMap<ToastVariantId, ToastStyle>,
}

impl Default for ToastStyles {
    fn default() -> Self {
        let solid = |bg: Color| ToastStyle {
            bg,
            fg: TEXT_MAIN,
            border: BORDER,
            shape: ToastShape::default(),
        };
        let info = solid(BG_NORMAL);
        Self {
            base: info,
            named: HashMap::from([
                (ToastVariantId::INFO, info),
                (ToastVariantId::SUCCESS, solid(AFFIRM_NORMAL)),
                (ToastVariantId::DANGER, solid(DANGER_NORMAL)),
            ]),
        }
    }
}

/// The footprint and the timing shared by every toast of a theme.
///
/// A toast is fully visible for its `duration` (the hold); the fades come on top of it, so it lives for
/// `fade_in + duration + fade_out` in total.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToastMetrics {
    pub min_width: Val,
    /// Text wraps at this width.
    pub max_width: Val,
    pub padding: UiRect,
    pub font_size: FontSize,
    /// The space between two stacked toasts.
    pub row_gap: Val,
    /// The space between the host's corner and the screen edge.
    pub edge_margin: Val,
    /// Seconds.
    pub fade_in: f32,
    /// Seconds.
    pub fade_out: f32,
    /// How far (px) a toast starts from its resting place and slides in from.
    pub slide: f32,
    /// Seconds a toast stays fully visible when its builder gives no `duration`.
    pub default_duration: f32,
}

impl Default for ToastMetrics {
    fn default() -> Self {
        Self {
            min_width: Val::Px(220.0),
            max_width: Val::Px(360.0),
            padding: UiRect::axes(SPACING_LG, SPACING_MD),
            font_size: FONT_SIZE_MD,
            row_gap: SPACING_SM,
            edge_margin: SPACING_LG,
            fade_in: 0.25,
            fade_out: 0.4,
            slide: 40.0,
            default_duration: 3.0,
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_define_the_three_kinds_with_their_own_backgrounds() {
        let styles = ToastStyles::default();
        let bg = |id| styles.named[&id].bg;
        assert_ne!(bg(ToastVariantId::INFO), bg(ToastVariantId::SUCCESS));
        assert_ne!(bg(ToastVariantId::SUCCESS), bg(ToastVariantId::DANGER));
        assert_ne!(bg(ToastVariantId::INFO), bg(ToastVariantId::DANGER));
    }

    #[test]
    fn the_base_style_is_the_info_style() {
        let styles = ToastStyles::default();
        assert_eq!(styles.base, styles.named[&ToastVariantId::INFO]);
    }

    #[test]
    fn a_name_is_a_key_and_a_user_name_sits_next_to_the_defaults() {
        const WARNING: ToastVariantId = ToastVariantId::new("warning");
        assert_eq!(WARNING, ToastVariantId::new("warning"));
        assert_ne!(WARNING, ToastVariantId::INFO);

        let mut styles = ToastStyles::default();
        let mut warning = styles.base;
        warning.bg = Color::srgb(0.9, 0.7, 0.1);
        styles.named.insert(WARNING, warning);
        assert_eq!(styles.named.len(), 4);
        assert_eq!(styles.named[&WARNING].bg, warning.bg);
    }

    #[test]
    fn the_default_timing_can_play() {
        let metrics = ToastMetrics::default();
        assert!(metrics.fade_in > 0.0);
        assert!(metrics.fade_out > 0.0);
        assert!(metrics.default_duration > 0.0);
        assert!(metrics.slide >= 0.0);
    }
}
