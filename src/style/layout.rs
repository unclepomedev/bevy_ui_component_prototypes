use bevy::prelude::*;

/// Configures a flex container with a row direction and centers its items cross-axis.
/// Conceptually similar to Tailwind's `flex-row items-center`.
pub(crate) fn row_center() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        ..default()
    }
}
