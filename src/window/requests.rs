use bevy::prelude::*;

/// Asks for `window` to be brought to the front.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct RaiseWindow {
    #[event_target]
    pub window: Entity,
}
