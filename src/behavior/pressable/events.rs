use bevy::prelude::*;

#[derive(EntityEvent)]
/// Fired when the entity is activated/pressed.
pub struct Activated {
    pub entity: Entity,
}
