use super::theme::Theme;
use bevy::{ecs::system::SystemParam, prelude::*};

/// The place where "which theme applies to this entity" is resolved.
#[derive(SystemParam)]
pub(crate) struct ThemeLookup<'w> {
    global: Res<'w, Theme>,
}

impl ThemeLookup<'_> {
    pub(crate) fn for_entity(&self, _entity: Entity) -> &Theme {
        &self.global
    }

    /// Determines whether all widgets need to be resynchronized.
    /// This is called when, for example, the theme is changed.
    pub(crate) fn needs_resync(&self) -> bool {
        self.global.is_changed()
    }
}
