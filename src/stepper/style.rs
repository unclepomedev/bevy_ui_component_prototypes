use super::logic::{CellProgress, StepperFill};
use crate::common::{
    color::{
        AFFIRM_HOVERED, AFFIRM_NORMAL, AFFIRM_PRESSED, BG_HOVERED, BG_NORMAL, BG_PRESSED, DISABLED,
        FOCUS_RING, TEXT_MAIN, TEXT_MUTED,
    },
    focus::FocusRing,
    state::{StateColors, VisualState},
};
use bevy::prelude::*;

/// The size and the corners of a stepper, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepperShape {
    /// Height of a cell. The previous and next buttons are squares of this size.
    pub cell_height: f32,
    /// The width below which a cell is too narrow to press; a stepper whose cells get narrower warns.
    pub min_cell_width: f32,
    /// The empty space between two cells.
    pub gap: f32,
    /// Corner radius of the cells and the buttons.
    pub radius: f32,
    /// Width of the border of a cell; none when 0.
    pub border_width: f32,
    /// Width and height of the mark under a cell.
    pub mark_size: f32,
    /// Which cells are lit, unless the stepper picks its own.
    pub fill: StepperFill,
}

impl Default for StepperShape {
    fn default() -> Self {
        Self {
            cell_height: 28.0,
            min_cell_width: 16.0,
            gap: 4.0,
            radius: 4.0,
            border_width: 0.0,
            mark_size: 4.0,
            fill: StepperFill::default(),
        }
    }
}

/// Colors, focus ring and shape of a stepper.
#[derive(Clone, Copy, Debug)]
pub struct StepperStyle {
    pub shape: StepperShape,
    /// Cells before the current one. Drawn as `upcoming` when the fill does not light them.
    pub passed: StateColors,
    /// The current cell.
    pub current: StateColors,
    /// Cells after the current one.
    pub upcoming: StateColors,
    /// The border of every cell, which shows once [`StepperShape::border_width`] is above 0.
    pub cell_border: StateColors,
    /// Background of the previous and next buttons.
    pub button: StateColors,
    /// The arrows on the buttons.
    pub arrow: StateColors,
    /// The text between the buttons.
    pub label: StateColors,
    /// The marks under the cells, in the state of their cell.
    pub mark: StateColors,
    /// Around the stepper while it owns the visible focus indicator; never shown when disabled.
    pub focus_ring: FocusRing,
}

impl Default for StepperStyle {
    fn default() -> Self {
        // A cell has no focused look: the ring is on the stepper as a whole.
        let cell = |normal, hovered, pressed, disabled| StateColors {
            normal,
            hovered,
            pressed,
            disabled,
            focused: normal,
        };
        let text = StateColors {
            focused: TEXT_MAIN,
            disabled: TEXT_MUTED,
            ..StateColors::uniform(TEXT_MAIN)
        };
        Self {
            shape: StepperShape::default(),
            passed: cell(AFFIRM_PRESSED, AFFIRM_NORMAL, AFFIRM_PRESSED, BG_PRESSED),
            current: cell(AFFIRM_NORMAL, AFFIRM_HOVERED, AFFIRM_PRESSED, BG_PRESSED),
            upcoming: cell(BG_NORMAL, BG_HOVERED, BG_PRESSED, DISABLED),
            cell_border: cell(TEXT_MUTED, TEXT_MAIN, TEXT_MAIN, DISABLED),
            button: StateColors {
                focused: BG_HOVERED,
                ..cell(BG_NORMAL, BG_HOVERED, BG_PRESSED, DISABLED)
            },
            arrow: text,
            label: text,
            mark: text,
            focus_ring: FocusRing {
                color: FOCUS_RING,
                width: Val::Px(2.0),
                offset: Val::Px(2.0),
            },
        }
    }
}

/// A previous or next button in one state, resolved from a [`StepperStyle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ArrowLook {
    pub(crate) background: Color,
    pub(crate) arrow: Color,
}

impl StepperStyle {
    /// `fill` is the stepper's own choice; the theme's applies when it has none.
    pub(crate) fn cell(
        &self,
        progress: CellProgress,
        fill: Option<StepperFill>,
        state: VisualState,
    ) -> Color {
        let fill = fill.unwrap_or(self.shape.fill);
        let colors = match progress {
            CellProgress::Current => &self.current,
            CellProgress::Passed if progress.is_lit(fill) => &self.passed,
            CellProgress::Passed | CellProgress::Upcoming => &self.upcoming,
        };
        colors.pick(state)
    }

    pub(crate) fn cell_border(&self, state: VisualState) -> Color {
        self.cell_border.pick(state)
    }

    pub(crate) fn arrow(&self, state: VisualState) -> ArrowLook {
        ArrowLook {
            background: self.button.pick(state),
            arrow: self.arrow.pick(state),
        }
    }

    pub(crate) fn label(&self, state: VisualState) -> Color {
        self.label.pick(state)
    }

    pub(crate) fn mark(&self, state: VisualState) -> Color {
        self.mark.pick(state)
    }

    /// `focused` is whether this stepper owns the visible focus indicator.
    pub(crate) fn ring(&self, focused: bool, state: VisualState) -> FocusRing {
        if focused && state != VisualState::Disabled {
            self.focus_ring
        } else {
            FocusRing::NONE
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    const NORMAL: VisualState = VisualState::Normal;

    #[test]
    fn the_current_cell_is_lit_whatever_the_fill() {
        let style = StepperStyle::default();
        for fill in [StepperFill::UpTo, StepperFill::Current] {
            assert_eq!(
                style.cell(CellProgress::Current, Some(fill), NORMAL),
                style.current.normal
            );
        }
    }

    #[test]
    fn up_to_draws_passed_cells_as_passed() {
        let style = StepperStyle::default();
        let color = style.cell(CellProgress::Passed, Some(StepperFill::UpTo), NORMAL);
        assert_eq!(color, style.passed.normal);
    }

    #[test]
    fn current_fill_draws_passed_cells_as_upcoming() {
        let style = StepperStyle::default();
        let color = style.cell(CellProgress::Passed, Some(StepperFill::Current), NORMAL);
        assert_eq!(color, style.upcoming.normal);
    }

    #[test]
    fn a_stepper_without_its_own_fill_follows_the_theme() {
        let mut style = StepperStyle::default();
        style.shape.fill = StepperFill::Current;
        let color = style.cell(CellProgress::Passed, None, NORMAL);
        assert_eq!(color, style.upcoming.normal);
        let own = style.cell(CellProgress::Passed, Some(StepperFill::UpTo), NORMAL);
        assert_eq!(own, style.passed.normal);
    }

    #[test]
    fn a_cell_follows_the_visual_state() {
        let style = StepperStyle::default();
        let color = style.cell(CellProgress::Upcoming, None, VisualState::Hovered);
        assert_eq!(color, style.upcoming.hovered);
    }

    #[test]
    fn the_default_palette_tells_the_three_kinds_of_cell_apart() {
        let style = StepperStyle::default();
        assert_ne!(style.passed.normal, style.current.normal);
        assert_ne!(style.current.normal, style.upcoming.normal);
        assert_ne!(style.passed.normal, style.upcoming.normal);
    }

    #[test]
    fn an_arrow_button_follows_the_visual_state() {
        let style = StepperStyle::default();
        let look = style.arrow(VisualState::Disabled);
        assert_eq!(look.background, style.button.disabled);
        assert_eq!(look.arrow, style.arrow.disabled);
    }

    #[test]
    fn a_cell_border_follows_the_visual_state_and_is_off_by_default() {
        let style = StepperStyle::default();
        assert_eq!(style.shape.border_width, 0.0);
        assert_eq!(
            style.cell_border(VisualState::Hovered),
            style.cell_border.hovered
        );
    }

    #[test]
    fn label_and_mark_follow_the_visual_state() {
        let style = StepperStyle::default();
        assert_eq!(style.label(VisualState::Pressed), style.label.pressed);
        assert_eq!(style.mark(VisualState::Disabled), style.mark.disabled);
    }

    #[test]
    fn ring_follows_focus_but_never_shows_on_a_disabled_stepper() {
        let style = StepperStyle::default();
        assert_eq!(style.ring(true, NORMAL), style.focus_ring);
        assert_eq!(style.ring(false, NORMAL), FocusRing::NONE);
        assert_eq!(style.ring(true, VisualState::Disabled), FocusRing::NONE);
    }
}
