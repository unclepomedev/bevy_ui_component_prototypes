/// The layer of a popup that opens inside the content of a widget: `gap` above the nearest ancestor
/// that has a `GlobalZIndex`, or above layer 0 when none has.
pub(crate) fn layer_above(ancestor: Option<i32>, gap: i32) -> i32 {
    ancestor.unwrap_or(0).saturating_add(gap)
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_popup_is_the_gap_above_its_ancestor() {
        assert_eq!(layer_above(Some(30), 5), 35);
        assert_eq!(layer_above(Some(-5), 1), -4);
    }

    #[test]
    fn without_an_ancestor_it_is_the_gap_above_layer_zero() {
        assert_eq!(layer_above(None, 5), 5);
    }

    #[test]
    fn it_stops_at_the_top() {
        assert_eq!(layer_above(Some(i32::MAX), 5), i32::MAX);
    }
}
