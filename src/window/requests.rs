use bevy::prelude::*;

/// Asks for `window` to be brought to the front.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct RaiseWindow {
    #[event_target]
    pub window: Entity,
}

/// Asks for `window` to be moved by `delta`, in logical pixels.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MoveWindow {
    #[event_target]
    pub window: Entity,
    pub delta: Vec2,
}
