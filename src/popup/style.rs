use crate::{
    button::ButtonVariant,
    common::{
        color::{BORDER, TEXT_MAIN},
        font_size::FONT_SIZE_MD,
        spacing::SPACING_LG,
    },
};
use bevy::prelude::*;

const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const PANEL_BG: Color = Color::srgb(0.1, 0.1, 0.15);

/// Border width and corner radius of a popup panel, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupShape {
    pub border_width: f32,
    pub radius: f32,
}

impl Default for PopupShape {
    fn default() -> Self {
        Self {
            border_width: 2.0,
            radius: 8.0,
        }
    }
}

/// Colors and shape of a popup, and the look of its buttons.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupStyle {
    /// Covers the screen behind the panel.
    pub scrim: Color,
    /// The color of the panel.
    pub bg: Color,
    pub border: Color,
    /// The color of the message.
    pub fg: Color,
    pub shape: PopupShape,
    /// The variant of the buttons added with [`PopupBuilder::button`](super::PopupBuilder::button).
    pub button: ButtonVariant,
}

impl Default for PopupStyle {
    fn default() -> Self {
        Self {
            scrim: SCRIM,
            bg: PANEL_BG,
            border: BORDER,
            fg: TEXT_MAIN,
            shape: PopupShape::default(),
            button: ButtonVariant::Default,
        }
    }
}

/// The footprint shared by every popup of a theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupMetrics {
    pub min_width: Val,
    /// The message wraps at this width.
    pub max_width: Val,
    pub padding: UiRect,
    /// The space between the message and the row of buttons.
    pub row_gap: Val,
    /// The space between two buttons.
    pub button_gap: Val,
    /// The width of each button.
    pub button_width: Val,
    pub font_size: FontSize,
}

impl Default for PopupMetrics {
    fn default() -> Self {
        Self {
            min_width: Val::Px(360.0),
            max_width: Val::Px(520.0),
            padding: UiRect::all(SPACING_LG),
            row_gap: SPACING_LG,
            button_gap: SPACING_LG,
            button_width: Val::Px(180.0),
            font_size: FONT_SIZE_MD,
        }
    }
}
