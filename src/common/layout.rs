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

/// Copies the listed fields of `want` into `node` (a `Mut<Node>`), only those that differ.
///
/// A write through `Mut` marks the component changed even when the value is the same, so each
/// field is compared first. List the fields the caller owns; the others are left to the app.
/// `want` is evaluated once per field, so pass a local.
macro_rules! write_changed {
    ($node:expr, $want:expr, $($field:ident),+ $(,)?) => {
        $(
            if $node.$field != $want.$field {
                $node.$field = $want.$field;
            }
        )+
    };
}
pub(crate) use write_changed;
