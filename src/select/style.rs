use crate::{
    button::{ButtonMetrics, ButtonStyle, ButtonStyles},
    common::color::{
        AFFIRM_HOVERED, AFFIRM_NORMAL, AFFIRM_PRESSED, BG_HOVERED, BG_NORMAL, BG_PRESSED, BORDER,
        DISABLED, TEXT_MAIN, TEXT_MUTED,
    },
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
    /// Background of an option row, by whether it is the chosen option and by its state.
    pub option_bg: SelectionColors,
    /// Text color of an option row, chosen the same way as `option_bg`.
    pub option_fg: SelectionColors,
}

/// The colors of an option row in each state it can be in. There is no hovered state: the row under the pointer
/// is the highlighted one.
#[derive(Clone, Copy, Debug)]
pub struct OptionColors {
    pub normal: Color,
    /// The row that holds the highlight: the one under the pointer, or the one the keys are on.
    pub highlighted: Color,
    pub pressed: Color,
    pub disabled: Color,
}

/// The colors of an option row, for the options that are not chosen and for the one that is.
#[derive(Clone, Copy, Debug)]
pub struct SelectionColors {
    pub unselected: OptionColors,
    pub selected: OptionColors,
}

/// Which colors an option row shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OptionState {
    Normal,
    Highlighted,
    Pressed,
    Disabled,
}

impl OptionState {
    /// Precedence: disabled > pressed > highlighted > normal.
    pub(crate) fn resolve(pressed: bool, highlighted: bool, disabled: bool) -> Self {
        if disabled {
            Self::Disabled
        } else if pressed {
            Self::Pressed
        } else if highlighted {
            Self::Highlighted
        } else {
            Self::Normal
        }
    }
}

impl OptionColors {
    pub(crate) fn pick(&self, state: OptionState) -> Color {
        match state {
            OptionState::Normal => self.normal,
            OptionState::Highlighted => self.highlighted,
            OptionState::Pressed => self.pressed,
            OptionState::Disabled => self.disabled,
        }
    }
}

impl Default for SelectStyle {
    fn default() -> Self {
        let bg = SelectionColors {
            unselected: OptionColors {
                normal: Color::NONE,
                highlighted: BG_HOVERED,
                pressed: BG_PRESSED,
                disabled: Color::NONE,
            },
            selected: OptionColors {
                normal: AFFIRM_NORMAL,
                highlighted: AFFIRM_HOVERED,
                pressed: AFFIRM_PRESSED,
                disabled: DISABLED,
            },
        };
        let fg = OptionColors {
            normal: TEXT_MAIN,
            highlighted: TEXT_MAIN,
            pressed: TEXT_MAIN,
            disabled: TEXT_MUTED,
        };
        Self {
            trigger: ButtonStyles::default().base,
            popup: SelectPopupStyle {
                bg: BG_NORMAL,
                border: BORDER,
                shape: SelectShape::default(),
            },
            option_bg: bg,
            option_fg: SelectionColors {
                unselected: fg,
                selected: fg,
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
    pub(crate) fn option_look(&self, state: OptionState, selected: bool) -> SelectOptionLook {
        let pick = |colors: &SelectionColors| {
            let colors = if selected {
                &colors.selected
            } else {
                &colors.unselected
            };
            colors.pick(state)
        };
        SelectOptionLook {
            bg: pick(&self.option_bg),
            fg: pick(&self.option_fg),
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
    /// How far above its nearest ancestor with a `GlobalZIndex` the popup sits.
    /// Counted from layer 0 when no ancestor has one.
    pub layer_gap: i32,
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
            layer_gap: 1,
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
        let selected = style.option_look(OptionState::Normal, true);
        let other = style.option_look(OptionState::Normal, false);
        assert_ne!(selected.bg, other.bg);
    }

    #[test]
    fn a_row_follows_its_state_and_the_selection() {
        let style = SelectStyle::default();
        let look = style.option_look(OptionState::Highlighted, true);
        assert_eq!(look.bg, style.option_bg.selected.highlighted);
        assert_eq!(look.fg, style.option_fg.selected.highlighted);
        let look = style.option_look(OptionState::Pressed, false);
        assert_eq!(look.bg, style.option_bg.unselected.pressed);
    }

    #[test]
    fn an_unselected_row_is_transparent_until_it_is_highlighted() {
        let style = SelectStyle::default();
        assert_eq!(
            style.option_look(OptionState::Normal, false).bg,
            Color::NONE
        );
        assert_ne!(
            style.option_look(OptionState::Highlighted, false).bg,
            Color::NONE
        );
    }

    #[test]
    fn a_row_state_resolves_by_precedence() {
        use OptionState::*;
        assert_eq!(OptionState::resolve(true, true, true), Disabled);
        assert_eq!(OptionState::resolve(true, true, false), Pressed);
        assert_eq!(OptionState::resolve(false, true, false), Highlighted);
        assert_eq!(OptionState::resolve(false, false, false), Normal);
    }

    #[test]
    fn the_default_metrics_leave_room_for_a_row_in_the_popup() {
        let metrics = SelectMetrics::default();
        assert_eq!(metrics.option_height, 40.0);
        assert!(metrics.popup_gap >= 0.0);
        assert!(metrics.max_visible_options >= 1);
    }
}
