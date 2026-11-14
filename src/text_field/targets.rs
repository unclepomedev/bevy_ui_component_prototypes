use super::components::{TextField, TextFieldInput};
use crate::common::{
    focus::ShowsFocus,
    style_targets::{drain_removed, pick_targets},
};
use bevy::{
    ecs::system::SystemParam, picking::hover::Hovered, prelude::*, text::EditableText,
    ui::InteractionDisabled,
};

#[derive(SystemParam)]
pub(super) struct FieldTargets<'w, 's> {
    changed: ChangedFields<'w, 's>,
    changed_inputs: ChangedInputs<'w, 's>,
    all: Query<'w, 's, Entity, With<TextField>>,
    parents: Query<'w, 's, &'static ChildOf>,
    enabled: RemovedComponents<'w, 's, InteractionDisabled>,
    unfocused: RemovedComponents<'w, 's, ShowsFocus>,
}

impl FieldTargets<'_, '_> {
    /// The field an entity belongs to: itself, or the field around its input.
    fn field_of(&self, entity: Entity) -> Option<Entity> {
        if self.all.contains(entity) {
            return Some(entity);
        }
        let parent = self.parents.get(entity).ok()?.parent();
        self.all.contains(parent).then_some(parent)
    }

    /// Every field after a theme change, otherwise only the ones that changed or whose input changed
    /// (including both ends of a focus move).
    pub(super) fn collect(&mut self, resync: bool, focus_moved: Vec<Entity>) -> Vec<Entity> {
        let mut removed = drain_removed(&mut self.enabled);
        removed.extend(drain_removed(&mut self.unfocused));
        let changed: Vec<Entity> = self
            .changed
            .iter()
            .chain(self.changed_inputs.iter())
            .filter_map(|entity| self.field_of(entity))
            .collect();
        let removed: Vec<Entity> = removed
            .into_iter()
            .filter_map(|entity| self.field_of(entity))
            .collect();
        let focus_moved: Vec<Entity> = focus_moved
            .into_iter()
            .filter_map(|entity| self.field_of(entity))
            .collect();
        pick_targets(resync, self.all.iter(), changed, removed, focus_moved)
    }
}

type ChangedFields<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<TextField>,
        Or<(
            Added<TextField>,
            Changed<Hovered>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

type ChangedInputs<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<TextFieldInput>,
        Or<(
            Added<TextFieldInput>,
            Changed<EditableText>,
            Added<ShowsFocus>,
        )>,
    ),
>;
