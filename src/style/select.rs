use super::{
    button::{ButtonMetrics, ButtonStyle, ButtonStyles},
    color::{
        AFFIRM_HOVERED, AFFIRM_NORMAL, AFFIRM_PRESSED, BG_HOVERED, BG_NORMAL, BG_PRESSED, BORDER,
        DISABLED, TEXT_MAIN, TEXT_MUTED,
    },
    state::{CheckedColors, StateColors, VisualState},
};
use bevy::prelude::*;

/// Border width and corner radius of the popup, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectShape {
    pub border_width: f32,
    pub radius: f32,
}

impl Default for SelectShape {
    fn default() -> Self {
        Self {
            border_width: 2.0,
            radius: 8.0,
        }
    }
}

/// The frame of the list that opens under the select.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectPopupStyle {
    pub bg: Color,
    pub border: Color,
    pub shape: SelectShape,
}

/// Colors and shape of a select.
#[derive(Clone, Debug)]
pub struct SelectStyle {
    /// The closed select is drawn like a button, so a ghost or a sliced `surface` works as it does there.
    pub trigger: ButtonStyle,
    pub popup: SelectPopupStyle,
    /// Background of an option row: `checked` is the selected option. The row under the pointer is the
    /// highlighted one, and it takes `focused`; a row does not use `hovered`.
    pub option_bg: CheckedColors,
    /// Text color of an option row, chosen the same way as `option_bg`.
    pub option_fg: CheckedColors,
}

impl Default for SelectStyle {
    fn default() -> Self {
        let bg = CheckedColors {
            unchecked: StateColors {
                normal: Color::NONE,
                hovered: BG_HOVERED,
                pressed: BG_PRESSED,
                disabled: Color::NONE,
                focused: BG_HOVERED,
            },
            checked: StateColors {
                normal: AFFIRM_NORMAL,
                hovered: AFFIRM_HOVERED,
                pressed: AFFIRM_PRESSED,
                disabled: DISABLED,
                focused: AFFIRM_HOVERED,
            },
        };
        let fg = StateColors {
            normal: TEXT_MAIN,
            hovered: TEXT_MAIN,
            pressed: TEXT_MAIN,
            disabled: TEXT_MUTED,
            focused: TEXT_MAIN,
        };
        Self {
            trigger: ButtonStyles::default().base,
            popup: SelectPopupStyle {
                bg: BG_NORMAL,
                border: BORDER,
                shape: SelectShape::default(),
            },
            option_bg: bg,
            option_fg: CheckedColors {
                unchecked: fg,
                checked: fg,
            },
        }
    }
}

/// What an option row looks like in one state, resolved from a [`SelectStyle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SelectOptionLook {
    pub(crate) bg: Color,
    pub(crate) fg: Color,
}

impl SelectStyle {
    /// `selected` is whether the row is the selected option.
    pub(crate) fn option_look(&self, state: VisualState, selected: bool) -> SelectOptionLook {
        SelectOptionLook {
            bg: self.option_bg.pick(selected, state),
            fg: self.option_fg.pick(selected, state),
        }
    }
}

/// The footprint of a select: the trigger, and the rows and the frame of the popup.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectMetrics {
    /// The trigger. The labels of the options take its font size too.
    pub trigger: ButtonMetrics,
    /// The height of an option row, in pixels.
    pub option_height: f32,
    pub option_padding: UiRect,
    pub popup_padding: UiRect,
    /// Space between the trigger and the popup, in pixels.
    pub popup_gap: f32,
    /// The popup is drawn in front of everything that has a lower `GlobalZIndex`. It has to be in front
    /// of what comes after the select in the UI tree, or that would hide it.
    pub popup_z_index: i32,
    /// How many options the list shows at most; with more, it scrolls. At least one.
    pub max_visible_options: usize,
}

impl Default for SelectMetrics {
    fn default() -> Self {
        Self {
            trigger: ButtonMetrics {
                height: Val::Px(56.0),
                ..ButtonMetrics::default()
            },
            option_height: 40.0,
            option_padding: UiRect::horizontal(Val::Px(16.0)),
            popup_padding: UiRect::all(Val::Px(4.0)),
            popup_gap: 4.0,
            popup_z_index: 50,
            max_visible_options: 8,
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
    fn the_trigger_starts_as_the_neutral_button() {
        let style = SelectStyle::default();
        let neutral = ButtonStyles::default().base;
        assert_eq!(style.trigger.bg.normal, neutral.bg.normal);
        assert_eq!(style.trigger.fg.normal, neutral.fg.normal);
    }

    #[test]
    fn the_selected_row_is_told_apart_from_the_others_by_default() {
        let style = SelectStyle::default();
        let selected = style.option_look(VisualState::Normal, true);
        let other = style.option_look(VisualState::Normal, false);
        assert_ne!(selected.bg, other.bg);
    }

    #[test]
    fn a_row_follows_the_visual_state_and_the_selection() {
        let style = SelectStyle::default();
        let look = style.option_look(VisualState::Hovered, true);
        assert_eq!(look.bg, style.option_bg.checked.hovered);
        assert_eq!(look.fg, style.option_fg.checked.hovered);
        let look = style.option_look(VisualState::Pressed, false);
        assert_eq!(look.bg, style.option_bg.unchecked.pressed);
    }

    #[test]
    fn an_unselected_row_is_transparent_until_it_is_hovered() {
        let style = SelectStyle::default();
        assert_eq!(
            style.option_look(VisualState::Normal, false).bg,
            Color::NONE
        );
        assert_ne!(
            style.option_look(VisualState::Hovered, false).bg,
            Color::NONE
        );
    }

    #[test]
    fn the_default_metrics_leave_room_for_a_row_in_the_popup() {
        let metrics = SelectMetrics::default();
        assert_eq!(metrics.option_height, 40.0);
        assert!(metrics.popup_gap >= 0.0);
        assert!(metrics.max_visible_options >= 1);
    }
}
