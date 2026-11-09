use super::{
    components::{Select, SelectTrigger},
    open::CloseList,
    trigger::sync_select_style,
};
use crate::common::style_targets::drain_removed;
use bevy::{prelude::*, ui::InteractionDisabled};

pub(super) struct SelectDisabledPlugin;
impl Plugin for SelectDisabledPlugin {
    fn build(&self, app: &mut App) {
        // Before the trigger is styled, so that it is gray in the same frame.
        app.add_systems(PostUpdate, forward_disabled.before(sync_select_style));
    }
}

type DisabledSelects<'w, 's> =
    Query<'w, 's, (Entity, &'static Children), (With<Select>, Added<InteractionDisabled>)>;

/// A select that is disabled is disabled where the user touches it: the trigger, which takes the focus and
/// the pointer. The list is closed when it is open.
fn forward_disabled(
    disabled: DisabledSelects,
    selects: Query<(&Children, Has<InteractionDisabled>), With<Select>>,
    triggers: Query<(), With<SelectTrigger>>,
    mut removed: RemovedComponents<InteractionDisabled>,
    mut commands: Commands,
) {
    let trigger_of = |children: &Children| children.iter().find(|child| triggers.contains(*child));

    for entity in drain_removed(&mut removed) {
        // A select that was disabled again in the same frame stays disabled.
        if let Ok((children, false)) = selects.get(entity)
            && let Some(trigger) = trigger_of(children)
        {
            commands.entity(trigger).try_remove::<InteractionDisabled>();
        }
    }
    for (select, children) in &disabled {
        if let Some(trigger) = trigger_of(children) {
            commands.entity(trigger).try_insert(InteractionDisabled);
        }
        commands.trigger(CloseList { select });
    }
}
