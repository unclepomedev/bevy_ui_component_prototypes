use bevy::picking::Pickable;

pub fn pass_through() -> Pickable {
    Pickable {
        should_block_lower: false,
        is_hoverable: false,
    }
}
