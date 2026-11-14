use super::{
    components::{Stepper, StepperButton, StepperCell, StepperValue, StepperView},
    logic::Move,
};
use crate::common::style_targets::drain_removed;
use bevy::{prelude::*, ui::InteractionDisabled};
use std::collections::BTreeSet;

type ChangedSteppers<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<Stepper>,
        Or<(
            Changed<Stepper>,
            Changed<StepperValue>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

/// Keeps `InteractionDisabled` on the cells and the buttons of a stepper as the stepper needs it.
/// A cell is disabled with the stepper, and a button also when pressing it could not change the value.
pub(super) fn sync_part_disabled(
    changed: ChangedSteppers,
    mut enabled: RemovedComponents<InteractionDisabled>,
    steppers: Query<StepperView>,
    children: Query<&Children>,
    parts: Query<(
        Has<InteractionDisabled>,
        Option<&StepperButton>,
        Option<&StepperCell>,
    )>,
    mut commands: Commands,
) {
    // Drain every removal, those on the parts too, which this system ignores.
    let enabled = drain_removed(&mut enabled)
        .into_iter()
        .filter(|entity| steppers.contains(*entity));
    let targets: BTreeSet<Entity> = changed.iter().chain(enabled).collect();

    for entity in targets {
        let Ok(stepper) = steppers.get(entity) else {
            continue;
        };
        for part in children.iter_descendants(entity) {
            let Ok((has, button, cell)) = parts.get(part) else {
                continue;
            };
            let want = match (button, cell) {
                (Some(button), _) => {
                    let step = Move::By(button.delta());
                    stepper.is_disabled() || stepper.change_for(step).is_none()
                }
                (None, Some(_)) => stepper.is_disabled(),
                (None, None) => continue,
            };
            if want == has {
                continue;
            }
            if want {
                commands.entity(part).try_insert(InteractionDisabled);
            } else {
                commands.entity(part).try_remove::<InteractionDisabled>();
            }
        }
    }
}
