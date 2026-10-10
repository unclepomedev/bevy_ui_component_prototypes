use bevy::picking::Pickable;

pub(crate) fn pass_through() -> Pickable {
    Pickable {
        should_block_lower: false,
        is_hoverable: false,
    }
}
