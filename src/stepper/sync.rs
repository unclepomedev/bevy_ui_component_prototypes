use super::{
    components::{
        Stepper, StepperButton, StepperCell, StepperMarks, StepperOwners, StepperValue, StepperView,
    },
    disabled::sync_part_disabled,
    nodes::lay_out,
    paint::{Look, StepperPaint},
    width::warn_narrow_cells,
};
use crate::common::{
    focus::FocusRingTracker,
    state::VisualState,
    style_targets::{drain_removed, pick_targets},
    theme_lookup::ThemeLookup,
};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{InteractionDisabled, UiSystems},
};

pub(crate) struct StepperStylePlugin;
impl Plugin for StepperStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                // The style reads what the first one writes.
                (sync_part_disabled, sync_stepper_style)
                    .chain()
                    .before(UiSystems::Prepare),
                // The width is known only once the layout has run.
                warn_narrow_cells.in_set(UiSystems::PostLayout),
            ),
        );
    }
}

/// Rewrites the look of each stepper that needs it: the focus ring on the stepper, and the colors
/// and the layout of its parts.
fn sync_stepper_style(
    mut lookup: ThemeLookup,
    mut ring: FocusRingTracker,
    mut targets: StepperTargets,
    mut steppers: Query<StepperRow>,
    mut paint: StepperPaint,
) {
    let (focused, focus_moved) = ring.update();
    let targets = targets.collect(lookup.needs_resync(), focus_moved);

    for entity in targets {
        let Ok(mut stepper) = steppers.get_mut(entity) else {
            continue;
        };
        let theme = lookup.for_entity(entity);
        let style = &theme.stepper;
        let is_focused = focused == Some(entity);
        let state = VisualState::from_flags(false, false, is_focused, stepper.view.is_disabled());

        let ring = style.ring(is_focused, state);
        stepper
            .outline
            .set_if_neq(Outline::new(ring.width, ring.offset, ring.color));
        let inline = stepper.view.is_inline();
        lay_out(&mut stepper.node, &style.shape, inline);

        let look = Look {
            theme,
            current: stepper.view.current(),
            fill: stepper.view.fill(),
            marks: stepper.marks,
            inline,
            state,
        };
        paint.apply(entity, &look);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct StepperRow {
    view: StepperView,
    marks: &'static StepperMarks,
    outline: &'static mut Outline,
    node: &'static mut Node,
}

#[derive(SystemParam)]
struct StepperTargets<'w, 's> {
    changed: ChangedSteppers<'w, 's>,
    changed_parts: ChangedParts<'w, 's>,
    all: Query<'w, 's, Entity, With<Stepper>>,
    owners: StepperOwners<'w, 's>,
    enabled: RemovedComponents<'w, 's, InteractionDisabled>,
}

impl StepperTargets<'_, '_> {
    /// Every stepper after a theme change, otherwise only the ones that changed: the stepper
    /// itself, one of its cells or buttons, or either end of a focus move.
    fn collect(&mut self, resync: bool, focus_moved: Vec<Entity>) -> Vec<Entity> {
        let enabled = drain_removed(&mut self.enabled);
        let enabled = enabled.into_iter().filter_map(|part| self.owners.of(part));
        let parts = self
            .changed_parts
            .iter()
            .filter_map(|part| self.owners.of(part));
        pick_targets(
            resync,
            self.all.iter(),
            self.changed.iter().chain(parts),
            enabled,
            focus_moved,
        )
    }
}

type ChangedSteppers<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<Stepper>,
        Or<(
            Changed<Stepper>,
            Changed<StepperValue>,
            Changed<StepperMarks>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

type ChangedParts<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        Or<(With<StepperCell>, With<StepperButton>)>,
        Or<(Changed<PickingInteraction>, Added<InteractionDisabled>)>,
    ),
>;
