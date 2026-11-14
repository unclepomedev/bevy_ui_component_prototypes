use crate::common::{
    color::{
        AFFIRM_HOVERED, AFFIRM_NORMAL, AFFIRM_PRESSED, BG_HOVERED, BG_NORMAL, BG_PRESSED, BORDER,
        DANGER_HOVERED, DANGER_NORMAL, DANGER_PRESSED, DISABLED, FOCUS_RING, PRIMARY_HOVERED,
        PRIMARY_NORMAL, PRIMARY_PRESSED, TEXT_MAIN, TEXT_MUTED,
    },
    focus::FocusRing,
    font_size::FONT_SIZE_LG,
    spacing::{SPACING_LG, SPACING_SM},
    state::{StateColors, VisualState},
};
use bevy::{asset::Handle, color::Color, image::Image, prelude::*};
use std::collections::HashMap;

/// A 9-sliced image drawn as the button's surface.
#[derive(Clone, Debug)]
pub struct SlicedImage {
    pub image: Handle<Image>,
    pub slicer: TextureSlicer,
}

impl SlicedImage {
    /// A frame image whose corners are `corner` pixels square and whose edges repeat.
    ///
    /// The edge pieces are tiled at their own size, not stretched, so a pattern drawn along an edge
    /// (dashes, say) keeps its period. The middle is stretched. See [`DashedFrame`](super::DashedFrame)
    /// for an image made for this.
    pub fn tiled(image: Handle<Image>, corner: f32) -> Self {
        Self {
            image,
            slicer: TextureSlicer {
                border: BorderRect::all(corner),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                max_corner_scale: 1.0,
            },
        }
    }
}

/// Border width and corner radius of a button, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonShape {
    pub border_width: f32,
    pub radius: f32,
}

impl Default for ButtonShape {
    fn default() -> Self {
        Self {
            border_width: 2.0,
            radius: 8.0,
        }
    }
}

/// The footprint of a button: its height, inner padding and gap, label font size and icon box.
/// They are meant to scale together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonMetrics {
    pub height: Val,
    pub padding: UiRect,
    pub column_gap: Val,
    pub font_size: FontSize,
    pub icon_size: f32,
}

impl Default for ButtonMetrics {
    fn default() -> Self {
        Self {
            height: Val::Px(80.0),
            padding: UiRect::axes(SPACING_LG, SPACING_SM),
            column_gap: SPACING_SM,
            font_size: FONT_SIZE_LG,
            icon_size: 40.0,
        }
    }
}

/// The name of a button size. Define one as a constant and use it for both the theme and the buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ButtonSizeId(&'static str);

impl ButtonSizeId {
    /// Creates an ID. The name is the key into [`ButtonSizes::named`].
    /// A name with no entry there falls back to [`ButtonSizes::base`].
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

/// The sizes a theme defines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ButtonSizes {
    /// Used by `ButtonSize::Default`, and by a name that has no entry in `named`.
    pub base: ButtonMetrics,
    pub named: HashMap<ButtonSizeId, ButtonMetrics>,
}

#[derive(Clone, Debug)]
pub struct ButtonStyle {
    /// Background color. With a `surface`, it tints the image instead, and the node's own background stays transparent.
    pub bg: StateColors,
    pub fg: StateColors,
    /// The color of the content marked [`ButtonAccent`](super::ButtonAccent). `None` leaves it to `fg`.
    pub accent: Option<StateColors>,
    pub border: StateColors,
    pub focus_ring: FocusRing,
    pub shape: ButtonShape,
    /// `None` draws `bg` as a plain background.
    pub surface: Option<SlicedImage>,
}

impl ButtonStyle {
    /// A button drawn as its label alone.
    pub fn ghost(fg: StateColors) -> Self {
        Self {
            bg: StateColors::uniform(Color::NONE),
            fg,
            accent: None,
            border: StateColors::uniform(Color::NONE),
            focus_ring: FocusRing::NONE,
            shape: ButtonShape {
                border_width: 0.0,
                ..ButtonShape::default()
            },
            surface: None,
        }
    }
}

/// The name of a button variant. Define one as a constant and use it for both the theme and the buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ButtonVariantId(&'static str);

impl ButtonVariantId {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    // The names of `ButtonStyles::default()`.
    pub const NEUTRAL: Self = Self::new("neutral");
    pub const PRIMARY: Self = Self::new("primary");
    pub const AFFIRMATIVE: Self = Self::new("affirmative");
    pub const DANGER: Self = Self::new("danger");
}

#[derive(Clone, Debug)]
pub struct ButtonStyles {
    /// Used by `ButtonVariant::Default`, and by a name that has no entry in `named`.
    pub base: ButtonStyle,
    pub named: HashMap<ButtonVariantId, ButtonStyle>,
}

impl Default for ButtonStyles {
    fn default() -> Self {
        let solid = |n: Color, h: Color, p: Color| ButtonStyle {
            bg: StateColors {
                normal: n,
                hovered: h,
                pressed: p,
                disabled: DISABLED,
                focused: h,
            },
            accent: None,
            fg: StateColors {
                normal: TEXT_MAIN,
                hovered: TEXT_MAIN,
                pressed: TEXT_MAIN,
                disabled: TEXT_MUTED,
                focused: TEXT_MAIN,
            },
            border: StateColors::uniform(BORDER),
            focus_ring: FocusRing {
                color: FOCUS_RING,
                width: Val::Px(2.0),
                offset: Val::Px(2.0),
            },
            shape: ButtonShape::default(),
            surface: None,
        };
        let neutral = solid(BG_NORMAL, BG_HOVERED, BG_PRESSED);
        Self {
            base: neutral.clone(),
            named: HashMap::from([
                (ButtonVariantId::NEUTRAL, neutral),
                (
                    ButtonVariantId::PRIMARY,
                    solid(PRIMARY_NORMAL, PRIMARY_HOVERED, PRIMARY_PRESSED),
                ),
                (
                    ButtonVariantId::AFFIRMATIVE,
                    solid(AFFIRM_NORMAL, AFFIRM_HOVERED, AFFIRM_PRESSED),
                ),
                (
                    ButtonVariantId::DANGER,
                    solid(DANGER_NORMAL, DANGER_HOVERED, DANGER_PRESSED),
                ),
            ]),
        }
    }
}

/// What a button looks like in one state, resolved from a [`ButtonStyle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ButtonLook {
    /// Color of the node's own background (transparent when an image surface is drawn).
    pub(crate) background: Color,
    /// Color multiplied into the image surface.
    pub(crate) tint: Color,
    pub(crate) border: Color,
    pub(crate) fg: Color,
    /// Color of the content marked `ButtonAccent`; `fg` when the style has no accent.
    pub(crate) accent: Color,
    pub(crate) ring: FocusRing,
}

impl ButtonStyle {
    /// `focused` is whether this button owns the visible focus indicator.
    pub(crate) fn look(&self, state: VisualState, focused: bool) -> ButtonLook {
        let tint = self.bg.pick(state);
        ButtonLook {
            background: if self.surface.is_some() {
                Color::NONE
            } else {
                tint
            },
            tint,
            border: self.border.pick(state),
            fg: self.fg.pick(state),
            accent: self
                .accent
                .map_or_else(|| self.fg.pick(state), |accent| accent.pick(state)),
            ring: if focused && state != VisualState::Disabled {
                self.focus_ring
            } else {
                FocusRing::NONE
            },
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    fn style(surface: Option<SlicedImage>) -> ButtonStyle {
        let mut style = Theme::default().buttons.base;
        style.surface = surface;
        style
    }

    fn surface() -> Option<SlicedImage> {
        Some(SlicedImage {
            image: Handle::default(),
            slicer: TextureSlicer::default(),
        })
    }

    #[test]
    fn plain_style_paints_the_state_color_as_background() {
        let style = style(None);
        let look = style.look(VisualState::Hovered, false);
        assert_eq!(look.background, style.bg.hovered);
        assert_eq!(look.border, style.border.hovered);
        assert_eq!(look.fg, style.fg.hovered);
    }

    #[test]
    fn image_style_keeps_the_background_transparent_and_tints_the_image() {
        let style = style(surface());
        let look = style.look(VisualState::Pressed, false);
        assert_eq!(look.background, Color::NONE);
        assert_eq!(look.tint, style.bg.pressed);
    }

    #[test]
    fn ring_follows_focus_but_never_shows_on_a_disabled_button() {
        let style = style(None);
        assert_eq!(
            style.look(VisualState::Normal, true).ring.color,
            style.focus_ring.color
        );
        assert_eq!(
            style.look(VisualState::Hovered, true).ring.color,
            style.focus_ring.color
        );
        assert_eq!(
            style.look(VisualState::Normal, false).ring.color,
            FocusRing::NONE.color
        );
        assert_eq!(
            style.look(VisualState::Disabled, true).ring.color,
            FocusRing::NONE.color
        );
    }

    #[test]
    fn a_ghost_style_draws_only_the_label() {
        let fg = StateColors {
            normal: Color::srgb(0.1, 0.2, 0.3),
            hovered: Color::srgb(0.2, 0.3, 0.4),
            pressed: Color::srgb(0.3, 0.4, 0.5),
            disabled: Color::srgb(0.4, 0.5, 0.6),
            focused: Color::srgb(0.5, 0.6, 0.7),
        };
        let style = ButtonStyle::ghost(fg);

        for state in [
            VisualState::Normal,
            VisualState::Hovered,
            VisualState::Pressed,
            VisualState::Focused,
            VisualState::Disabled,
        ] {
            let look = style.look(state, true);
            assert_eq!(look.background, Color::NONE);
            assert_eq!(look.border, Color::NONE);
            assert_eq!(look.ring, FocusRing::NONE);
            assert_eq!(look.fg, fg.pick(state));
            assert_eq!(look.accent, fg.pick(state)); // no accent: it follows `fg`
        }
    }

    #[test]
    fn the_accent_has_its_own_colors_when_the_style_gives_them() {
        let mut style = style(None);
        let accent = StateColors {
            normal: Color::srgb(0.0, 0.9, 0.9),
            hovered: Color::srgb(0.1, 0.9, 0.9),
            pressed: Color::srgb(0.2, 0.9, 0.9),
            disabled: Color::srgb(0.3, 0.9, 0.9),
            focused: Color::srgb(0.4, 0.9, 0.9),
        };
        style.accent = Some(accent);

        let look = style.look(VisualState::Pressed, false);
        assert_eq!(look.accent, accent.pressed);
        assert_eq!(look.fg, style.fg.pressed);
    }
}
