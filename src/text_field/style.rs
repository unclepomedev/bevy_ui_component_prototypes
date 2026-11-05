use crate::common::{
    color::{BG_NORMAL, BG_PRESSED, BORDER, DISABLED, FOCUS_RING, TEXT_MAIN, TEXT_MUTED},
    focus::FocusRing,
    state::{StateColors, VisualState},
};
use bevy::prelude::*;

/// The border and the corners of a text field, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextFieldShape {
    pub border_width: f32,
    pub radius: f32,
}

impl Default for TextFieldShape {
    fn default() -> Self {
        Self {
            border_width: 1.0,
            radius: 4.0,
        }
    }
}

/// Colors, focus ring and shape of a text field. The state colors have no pressed look: `pressed` is unused.
#[derive(Clone, Copy, Debug)]
pub struct TextFieldStyle {
    pub bg: StateColors,
    pub border: StateColors,
    /// The color of the text.
    pub text: StateColors,
    pub placeholder: Color,
    pub cursor: Color,
    /// The background of the selected text while the field has the focus.
    pub selection: Color,
    /// The background of the selected text while it does not.
    pub unfocused_selection: Color,
    pub focus_ring: FocusRing,
    pub shape: TextFieldShape,
}

/// The footprint of a text field. In logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextFieldMetrics {
    /// The space between the border and the text.
    pub padding: UiRect,
    pub font_size: FontSize,
}

impl Default for TextFieldMetrics {
    fn default() -> Self {
        Self {
            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
            font_size: FontSize::Px(16.0),
        }
    }
}

impl Default for TextFieldStyle {
    fn default() -> Self {
        Self {
            bg: StateColors {
                normal: BG_PRESSED,
                hovered: BG_NORMAL,
                pressed: BG_NORMAL,
                disabled: DISABLED,
                focused: BG_PRESSED,
            },
            border: StateColors {
                normal: BORDER,
                hovered: BORDER,
                pressed: BORDER,
                disabled: BORDER,
                focused: FOCUS_RING,
            },
            text: StateColors {
                normal: TEXT_MAIN,
                hovered: TEXT_MAIN,
                pressed: TEXT_MAIN,
                disabled: TEXT_MUTED,
                focused: TEXT_MAIN,
            },
            placeholder: TEXT_MUTED,
            cursor: TEXT_MAIN,
            selection: Color::srgb(0.25, 0.40, 0.70),
            unfocused_selection: Color::srgb(0.30, 0.30, 0.40),
            focus_ring: FocusRing {
                color: FOCUS_RING,
                width: Val::Px(2.0),
                offset: Val::Px(2.0),
            },
            shape: TextFieldShape::default(),
        }
    }
}

/// What a text field looks like in one state, resolved from a [`TextFieldStyle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextFieldLook {
    pub(crate) bg: Color,
    pub(crate) border: Color,
    pub(crate) text: Color,
    pub(crate) ring: FocusRing,
}

impl TextFieldStyle {
    /// `ring_visible`: the focus indicator is drawn on the field.
    pub(crate) fn look(&self, state: VisualState, ring_visible: bool) -> TextFieldLook {
        TextFieldLook {
            bg: self.bg.pick(state),
            border: self.border.pick(state),
            text: self.text.pick(state),
            ring: if ring_visible && state != VisualState::Disabled {
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

    #[test]
    fn the_ring_is_drawn_only_while_the_indicator_is_visible_and_the_field_is_enabled() {
        let style = TextFieldStyle::default();
        assert_eq!(
            style.look(VisualState::Focused, true).ring,
            style.focus_ring
        );
        assert_eq!(
            style.look(VisualState::Focused, false).ring,
            FocusRing::NONE
        );
        assert_eq!(
            style.look(VisualState::Disabled, true).ring,
            FocusRing::NONE
        );
    }

    #[test]
    fn the_default_focused_border_differs_from_the_normal_one() {
        let style = TextFieldStyle::default();
        assert_ne!(style.border.focused, style.border.normal);
    }
}
