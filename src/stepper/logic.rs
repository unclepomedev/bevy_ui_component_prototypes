use std::cmp::Ordering;

/// Which cells of a stepper are lit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepperFill {
    /// Every cell up to and including the current one.
    #[default]
    UpTo,
    /// Only the current cell.
    Current,
}

/// Where a cell is relative to the current one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellProgress {
    Passed,
    Current,
    Upcoming,
}

impl CellProgress {
    /// `current` must already be in range (see [`clamp_index`]).
    pub(crate) fn of(index: usize, current: usize) -> Self {
        match index.cmp(&current) {
            Ordering::Less => Self::Passed,
            Ordering::Equal => Self::Current,
            Ordering::Greater => Self::Upcoming,
        }
    }

    pub(crate) fn is_lit(self, fill: StepperFill) -> bool {
        match self {
            Self::Current => true,
            Self::Passed => fill == StepperFill::UpTo,
            Self::Upcoming => false,
        }
    }
}

/// Brings a stored value into `0..steps`. `steps` must be at least 1.
pub(crate) fn clamp_index(value: usize, steps: usize) -> usize {
    value.min(steps - 1)
}

/// The value to request when moving `current` by `delta`, or `None` when that would not change it.
/// `current` must already be in range. Without `wrap` the result stops at the ends; with it, it continues from the opposite end.
fn next_index(current: usize, steps: usize, delta: isize, wrap: bool) -> Option<usize> {
    let count = steps as isize;
    let moved = current as isize + delta;
    let moved = if wrap {
        moved.rem_euclid(count)
    } else {
        moved.clamp(0, count - 1)
    };
    changed(current, moved as usize)
}

/// The value to request when jumping from `current` to `index`, or `None` when that would not
/// change it. An `index` out of range counts as the nearest end.
fn jump_index(current: usize, steps: usize, index: usize) -> Option<usize> {
    changed(current, clamp_index(index, steps))
}

/// What a press asks of a stepper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Move {
    By(isize),
    To(usize),
    First,
    Last,
}

/// The value to request for `request`, or `None` when that would not change `current`.
pub(crate) fn target(current: usize, steps: usize, wrap: bool, request: Move) -> Option<usize> {
    match request {
        Move::By(delta) => next_index(current, steps, delta, wrap),
        Move::To(index) => jump_index(current, steps, index),
        Move::First => jump_index(current, steps, 0),
        Move::Last => jump_index(current, steps, steps - 1),
    }
}

// A request equal to the current value is not a change, so nothing is reported for it.
fn changed(current: usize, requested: usize) -> Option<usize> {
    (requested != current).then_some(requested)
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_past_the_end_is_shown_at_the_last_cell() {
        assert_eq!(clamp_index(99, 10), 9);
        assert_eq!(clamp_index(3, 10), 3);
        assert_eq!(clamp_index(5, 1), 0);
    }

    #[test]
    fn cells_are_passed_current_or_upcoming() {
        assert_eq!(CellProgress::of(1, 3), CellProgress::Passed);
        assert_eq!(CellProgress::of(3, 3), CellProgress::Current);
        assert_eq!(CellProgress::of(4, 3), CellProgress::Upcoming);
    }

    #[test]
    fn up_to_lights_the_passed_cells_and_current_lights_only_the_current_one() {
        let lit = |fill| {
            [0, 1, 2, 3, 4]
                .map(|index| CellProgress::of(index, 2).is_lit(fill))
                .iter()
                .filter(|lit| **lit)
                .count()
        };
        assert_eq!(lit(StepperFill::UpTo), 3);
        assert_eq!(lit(StepperFill::Current), 1);
    }

    #[test]
    fn stepping_moves_by_the_delta() {
        assert_eq!(next_index(3, 10, 1, false), Some(4));
        assert_eq!(next_index(3, 10, -1, false), Some(2));
    }

    #[test]
    fn stepping_stops_at_the_ends_without_wrap() {
        assert_eq!(next_index(9, 10, 1, false), None);
        assert_eq!(next_index(0, 10, -1, false), None);
    }

    #[test]
    fn stepping_continues_from_the_other_end_with_wrap() {
        assert_eq!(next_index(9, 10, 1, true), Some(0));
        assert_eq!(next_index(0, 10, -1, true), Some(9));
    }

    #[test]
    fn a_single_step_never_changes() {
        for wrap in [false, true] {
            assert_eq!(next_index(0, 1, 1, wrap), None);
            assert_eq!(next_index(0, 1, -1, wrap), None);
        }
    }

    #[test]
    fn a_jump_to_the_current_value_is_not_a_change() {
        assert_eq!(jump_index(4, 10, 4), None);
        assert_eq!(jump_index(4, 10, 7), Some(7));
    }

    #[test]
    fn a_jump_past_the_end_lands_on_the_last_cell() {
        assert_eq!(jump_index(4, 10, 99), Some(9));
        assert_eq!(jump_index(9, 10, 99), None);
    }

    #[test]
    fn each_request_resolves_to_a_value_or_to_nothing() {
        assert_eq!(target(3, 10, false, Move::By(1)), Some(4));
        assert_eq!(target(9, 10, true, Move::By(1)), Some(0));
        assert_eq!(target(9, 10, false, Move::By(1)), None);
        assert_eq!(target(3, 10, false, Move::To(6)), Some(6));
        assert_eq!(target(3, 10, false, Move::To(3)), None);
        assert_eq!(target(3, 10, false, Move::First), Some(0));
        assert_eq!(target(3, 10, false, Move::Last), Some(9));
        assert_eq!(target(0, 10, true, Move::First), None);
        assert_eq!(target(9, 10, true, Move::Last), None);
    }
}
