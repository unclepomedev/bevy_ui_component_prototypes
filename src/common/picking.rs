use bevy::picking::Pickable;

/// Lets pointer events pass through a decorative child (a label, an icon, a thumb) to the parent widget.
pub(crate) fn pass_through() -> Pickable {
    Pickable {
        should_block_lower: false,
        is_hoverable: false,
    }
}
