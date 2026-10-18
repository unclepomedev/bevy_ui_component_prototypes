use crate::style::theme::{GlobalTheme, Theme};
use bevy::{ecs::system::SystemParam, prelude::*};

/// The place where "which theme applies to this entity" is resolved.
#[derive(SystemParam)]
pub(crate) struct ThemeLookup<'w, 's> {
    global: Res<'w, GlobalTheme>,
    themes: Query<'w, 's, &'static Theme>,
    parents: Query<'w, 's, &'static ChildOf>,
    changed_themes: Query<'w, 's, (), Changed<Theme>>,
    reparented: Query<'w, 's, (), Changed<ChildOf>>,
    removed_themes: RemovedComponents<'w, 's, Theme>,
    unparented: RemovedComponents<'w, 's, ChildOf>,
}

impl ThemeLookup<'_, '_> {
    /// Returns the theme applying to `entity`: the closest `Theme`
    /// on the entity or one of its ancestors, falling back to the global theme.
    pub(crate) fn for_entity(&self, entity: Entity) -> &Theme {
        std::iter::once(entity)
            .chain(self.parents.iter_ancestors(entity))
            .find_map(|e| self.themes.get(e).ok())
            .unwrap_or(&self.global)
    }

    /// Whether the resolved theme of any widget may have changed.
    /// Consumes pending removal messages so each removal triggers exactly one resyncing.
    pub(crate) fn needs_resync(&mut self) -> bool {
        let removed = !self.removed_themes.is_empty() || !self.unparented.is_empty();
        self.removed_themes.clear();
        self.unparented.clear();

        removed
            || self.global.is_changed()
            || !self.changed_themes.is_empty()
            || !self.reparented.is_empty()
    }
}
