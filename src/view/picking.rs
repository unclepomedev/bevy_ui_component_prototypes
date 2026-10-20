use bevy::picking::Pickable;

/// Lets pointer events fall through to the parent widget instead of being absorbed by a decorative child (a label, an icon, a thumb).
pub(crate) fn pass_through() -> Pickable {
    Pickable {
        should_block_lower: false,
        is_hoverable: false,
    }
}
