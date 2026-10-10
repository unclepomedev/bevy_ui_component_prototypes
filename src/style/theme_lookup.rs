use super::theme::{GlobalTheme, Theme};
use bevy::{ecs::system::SystemParam, prelude::*};

/// The place where "which theme applies to this entity" is resolved.
#[derive(SystemParam)]
pub(crate) struct ThemeLookup<'w, 's> {
    global: Res<'w, GlobalTheme>,
    themes: Query<'w, 's, &'static Theme>,
    parents: Query<'w, 's, &'static ChildOf>,
    changed_themes: Query<'w, 's, (), Changed<Theme>>,
    reparented: Query<'w, 's, (), Changed<ChildOf>>,
}

impl ThemeLookup<'_, '_> {
    /// Starting from the current entity, trace back through the ancestors and return the first Theme found; if none is found, return Global.
    pub(crate) fn for_entity(&self, entity: Entity) -> &Theme {
        std::iter::once(entity)
            .chain(self.parents.iter_ancestors(entity))
            .find_map(|e| self.themes.get(e).ok())
            .unwrap_or(self.global.as_ref())
    }

    /// Determines whether all widgets need to be resynchronized.
    /// This is called when, for example, the theme is changed.
    pub(crate) fn needs_resync(&self) -> bool {
        self.global.is_changed() || !self.changed_themes.is_empty() || !self.reparented.is_empty()
    }
}
