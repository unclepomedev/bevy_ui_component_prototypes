use bevy::prelude::*;

/// Fired when the entity is activated (pressed).
#[derive(EntityEvent)]
pub struct Activated {
    pub entity: Entity,
}
