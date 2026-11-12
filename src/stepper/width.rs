use super::components::StepperCell;
use crate::common::theme_lookup::ThemeLookup;
use bevy::{log::warn_once, prelude::*};

// Layout rounds to whole pixels, so a cell at exactly the minimum can come out a little under it.
const SLACK: f32 = 0.5;

/// Whether a cell `width` wide is narrower than the `min` the theme asks for.
fn is_too_narrow(width: f32, min: f32) -> bool {
    // A node is 0 wide until its first layout, and that is not a narrow cell.
    width > 0.0 && width + SLACK < min
}

type ChangedCells<'w, 's> =
    Query<'w, 's, (Entity, &'static ComputedNode), (With<StepperCell>, Changed<ComputedNode>)>;

/// Warns once when a stepper's cells are narrower than its theme's minimum. Nothing else changes.
pub(super) fn warn_narrow_cells(lookup: ThemeLookup, cells: ChangedCells) {
    for (entity, node) in &cells {
        let width = node.size.x * node.inverse_scale_factor;
        let min = lookup.for_entity(entity).stepper.shape.min_cell_width;
        if is_too_narrow(width, min) {
            warn_once!(
                "A stepper cell is {width:.1}px wide, under the {min:.1}px its theme asks for. \
                 Use fewer steps or a wider parent; it keeps working."
            );
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
    fn a_cell_under_the_minimum_is_too_narrow() {
        assert!(is_too_narrow(10.0, 16.0));
    }

    #[test]
    fn a_cell_at_the_minimum_is_not_too_narrow_even_after_rounding() {
        assert!(!is_too_narrow(16.0, 16.0));
        assert!(!is_too_narrow(15.7, 16.0));
    }

    #[test]
    fn a_node_that_was_not_laid_out_is_not_too_narrow() {
        assert!(!is_too_narrow(0.0, 16.0));
    }
}
