use super::state::{StateColors, VisualState};
use bevy::{asset::Handle, color::Color, image::Image, prelude::*};

/// Outline drawn around a button while its focus indicator is visible.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusRing {
    pub color: Color,
    pub width: Val,
    pub offset: Val,
}

impl FocusRing {
    /// No ring; focus is conveyed by the `focused` colors alone.
    pub const NONE: Self = Self {
        color: Color::NONE,
        width: Val::Px(0.0),
        offset: Val::Px(0.0),
    };
}

/// A 9-sliced image drawn as the button's surface.
#[derive(Clone, Debug)]
pub struct SlicedImage {
    pub image: Handle<Image>,
    pub slicer: TextureSlicer,
}

#[derive(Clone, Debug)]
pub struct ButtonStyle {
    /// Background color. With a `surface`, it tints the image instead, and the node's own background stays transparent.
    pub bg: StateColors,
    pub fg: StateColors,
    pub border: StateColors,
    pub focus_ring: FocusRing,
    /// `None` draws `bg` as a plain background.
    pub surface: Option<SlicedImage>,
}

#[derive(Clone, Debug)]
pub struct ButtonStyles {
    pub neutral: ButtonStyle,
    pub primary: ButtonStyle,
    pub affirmative: ButtonStyle,
    pub danger: ButtonStyle,
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
            ring: if focused && state != VisualState::Disabled {
                self.focus_ring
            } else {
                FocusRing::NONE
            },
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
    pub(crate) ring: FocusRing,
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::theme::Theme;

    fn style(surface: Option<SlicedImage>) -> ButtonStyle {
        let mut style = Theme::default().buttons.primary;
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
}
