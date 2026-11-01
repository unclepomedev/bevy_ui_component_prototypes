use bevy::prelude::*;
use std::{collections::HashMap, fmt::Debug, hash::Hash};

/// Which entry of a theme a widget takes: the base one, or the one defined under a name.
/// Each widget names its own `Id` type, so an id of one widget cannot be given to another.
#[derive(Component, Clone, Copy, Debug, PartialEq, Default)]
pub enum Choice<Id: Send + Sync + 'static> {
    #[default]
    Default,
    /// The entry the theme defines under this name. A name the theme lacks falls back to the base entry.
    Named(Id),
}

impl<Id: Send + Sync + 'static> From<Id> for Choice<Id> {
    fn from(id: Id) -> Self {
        Self::Named(id)
    }
}

impl<Id: Copy + Eq + Hash + Send + Sync + 'static> Choice<Id> {
    /// `on_missing` runs when a name has no entry in `named`; the base entry is used then.
    pub(crate) fn select<'a, T>(
        &self,
        base: &'a T,
        named: &'a HashMap<Id, T>,
        on_missing: impl FnOnce(&Id),
    ) -> &'a T {
        match self {
            Self::Default => base,
            Self::Named(id) => named.get(id).unwrap_or_else(|| {
                on_missing(id);
                base
            }),
        }
    }
}
