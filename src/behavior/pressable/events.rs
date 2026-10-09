use bevy::prelude::*;

#[derive(EntityEvent)]
pub struct Pressed {
    pub entity: Entity,
}
