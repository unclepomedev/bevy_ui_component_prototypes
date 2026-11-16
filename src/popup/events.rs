use super::components::PopupChoice;
use bevy::prelude::*;

/// A button of `popup` was activated. The popup stays until an observer of this despawns it.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct PopupChosen {
    #[event_target]
    pub popup: Entity,
    pub choice: PopupChoice,
}
