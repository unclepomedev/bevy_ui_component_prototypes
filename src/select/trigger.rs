use super::components::{
    Select, SelectArrow, SelectContent, SelectLabels, SelectTrigger, SelectedIndex, trigger_node,
};
use crate::{
    button::{ButtonIcon, ButtonLabel, ButtonParts, write_button_colors, write_button_layout},
    common::{
        focus::FocusRingTracker,
        picking::pass_through,
        state::VisualState,
        style_targets::{drain_removed, pick_targets},
        theme_lookup::ThemeLookup,
    },
};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{InteractionDisabled, UiSystems},
};

pub(super) struct SelectTriggerStylePlugin;
impl Plugin for SelectTriggerStylePlugin {
    fn build(&self, app: &mut App) {
        // The label is replaced first, so that the new one is styled in the same frame.
        app.add_systems(
            PostUpdate,
            (refresh_select_label, sync_select_style)
                .chain()
                .before(UiSystems::Prepare),
        );
    }
}

#[derive(QueryData)]
pub(super) struct LabelRow {
    entity: Entity,
    chosen: &'static SelectedIndex,
    labels: &'static SelectLabels,
}

/// Shows the label of the chosen option, or the placeholder, in the trigger.
pub(super) fn refresh_select_label(
    selects: Query<LabelRow, (With<Select>, Changed<SelectedIndex>)>,
    children: Query<&Children>,
    slots: Query<(), With<SelectContent>>,
    mut commands: Commands,
) {
    for item in &selects {
        let Some(slot) = children
            .iter_descendants(item.entity)
            .find(|entity| slots.contains(*entity))
        else {
            continue;
        };
        commands.entity(slot).despawn_related::<Children>();

        let wanted = item
            .chosen
            .and_then(|index| item.labels.options.get(index))
            .or(item.labels.placeholder.as_ref());
        if let Some(spawn_label) = wanted {
            let spawn_label = spawn_label.clone();
            commands
                .spawn(ChildOf(slot))
                .queue(move |mut label: EntityWorldMut| {
                    spawn_label(&mut label);
                    // Unseen until the style sync colors it; a default `TextColor` is white.
                    label.insert((ButtonLabel, TextColor(Color::NONE), pass_through()));
                });
        }
    }
}

/// Resolves each trigger's look from its theme and state and writes it to the trigger and its content.
pub(super) fn sync_select_style(
    mut lookup: ThemeLookup,
    mut ring: FocusRingTracker,
    mut targets: SelectTargets,
    mut triggers: Query<TriggerRow, (With<SelectTrigger>, Without<ButtonIcon>)>,
    mut arrows: Query<&mut BorderColor, (With<SelectArrow>, Without<SelectTrigger>)>,
    children: Query<&Children>,
    mut parts: ButtonParts,
) {
    let (focused, focus_moved) = ring.update();
    let targets = targets.collect(lookup.needs_resync(), focus_moved);

    for entity in targets {
        let Ok(mut trigger) = triggers.get_mut(entity) else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let style = &theme.select.trigger;
        let metrics = &theme.select_metrics.trigger;
        let is_focused = focused == Some(entity);
        let state = VisualState::resolve(trigger.interaction, is_focused, trigger.disabled);
        let look = style.look(state, is_focused);

        write_button_colors(
            &look,
            &mut trigger.bg,
            &mut trigger.border,
            &mut trigger.outline,
        );
        write_button_layout(
            &mut trigger.node,
            &trigger_node(&theme.select_metrics, &style.shape),
        );
        parts.paint(entity, style, &look, Some(metrics), theme);
        for descendant in children.iter_descendants(entity) {
            if let Ok(mut arrow) = arrows.get_mut(descendant) {
                arrow.set_if_neq(BorderColor::all(look.fg));
            }
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct TriggerRow {
    interaction: &'static PickingInteraction,
    disabled: Has<InteractionDisabled>,
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    outline: &'static mut Outline,
    node: &'static mut Node,
}

#[derive(SystemParam)]
pub(super) struct SelectTargets<'w, 's> {
    changed: ChangedTriggers<'w, 's>,
    all: Query<'w, 's, Entity, With<SelectTrigger>>,
    triggers: Query<'w, 's, (), With<SelectTrigger>>,
    chosen: Query<'w, 's, &'static Children, (With<Select>, Changed<SelectedIndex>)>,
    enabled: RemovedComponents<'w, 's, InteractionDisabled>,
}

impl SelectTargets<'_, '_> {
    /// A trigger is also restyled when the choice changes: the label is replaced, and the new one
    /// has to be styled.
    fn collect(&mut self, resync: bool, focus_moved: Vec<Entity>) -> Vec<Entity> {
        let mut removed = drain_removed(&mut self.enabled);
        removed.extend(
            self.chosen
                .iter()
                .flat_map(|children| children.iter())
                .filter(|child| self.triggers.contains(*child)),
        );
        pick_targets(
            resync,
            self.all.iter(),
            self.changed.iter(),
            removed,
            focus_moved,
        )
    }
}

type ChangedTriggers<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<SelectTrigger>,
        Or<(
            Added<SelectTrigger>,
            Changed<PickingInteraction>,
            Added<InteractionDisabled>,
        )>,
    ),
>;
