mod builder;
mod components;

pub use self::builder::{ToggleSwitchBuilder, toggle_switch};
pub use self::components::ToggleSwitch;

use self::components::{SwitchThumb, thumb_side};
use super::{
    focus::FocusRingTracker,
    style_targets::{drain_removed, pick_targets},
    theme_lookup::ThemeLookup,
};
use crate::style::state::VisualState;
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{Checked, InteractionDisabled, UiSystems},
};

pub(crate) struct SwitchStylePlugin;
impl Plugin for SwitchStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_switch_style.before(UiSystems::Prepare));
    }
}

/// Resolves each switch's look from its theme and state and writes it to the track and the thumb.
fn sync_switch_style(
    mut lookup: ThemeLookup,
    mut ring: FocusRingTracker,
    mut targets: SwitchTargets,
    mut switches: SwitchQuery,
    mut thumbs: Query<&mut BackgroundColor, (With<SwitchThumb>, Without<ToggleSwitch>)>,
) {
    let (focused, focus_moved) = ring.update();
    let targets = targets.collect(lookup.needs_resync(), focus_moved);

    for entity in targets {
        let Ok(mut switch) = switches.get_mut(entity) else {
            continue;
        };

        let style = lookup.for_entity(entity).switch;
        let is_focused = focused == Some(entity);
        let state = VisualState::resolve(switch.interaction, is_focused, switch.disabled);
        let look = style.look(state, switch.checked, is_focused);

        switch.bg.set_if_neq(BackgroundColor(look.track));
        switch.border.set_if_neq(BorderColor::all(look.border));
        switch.outline.set_if_neq(Outline::new(
            look.ring.width,
            look.ring.offset,
            look.ring.color,
        ));
        let side = thumb_side(switch.checked);
        if switch.node.justify_content != side {
            switch.node.justify_content = side;
        }
        for child in switch
            .children
            .into_iter()
            .flat_map(|children| children.iter())
        {
            if let Ok(mut thumb) = thumbs.get_mut(child) {
                thumb.set_if_neq(BackgroundColor(look.thumb));
            }
        }
    }
}

/// Finds the switches whose look may have changed from the last run.
#[derive(SystemParam)]
struct SwitchTargets<'w, 's> {
    changed: ChangedSwitches<'w, 's>,
    all: Query<'w, 's, Entity, With<ToggleSwitch>>,
    unchecked: RemovedComponents<'w, 's, Checked>,
    enabled: RemovedComponents<'w, 's, InteractionDisabled>,
}

impl SwitchTargets<'_, '_> {
    /// Every switch after a theme change, otherwise only the ones that changed
    /// (including both ends of a focus move).
    fn collect(&mut self, resync: bool, focus_moved: Vec<Entity>) -> Vec<Entity> {
        let mut removed = drain_removed(&mut self.unchecked);
        removed.extend(drain_removed(&mut self.enabled));
        pick_targets(
            resync,
            self.all.iter(),
            self.changed.iter(),
            removed,
            focus_moved,
        )
    }
}

type ChangedSwitches<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<ToggleSwitch>,
        Or<(
            Added<ToggleSwitch>,
            Changed<PickingInteraction>,
            Changed<Checked>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

type SwitchQuery<'w, 's> = Query<'w, 's, SwitchRow, With<ToggleSwitch>>;

/// What the sync reads from a switch and writes to it.
#[derive(QueryData)]
#[query_data(mutable)]
struct SwitchRow {
    interaction: &'static PickingInteraction,
    checked: Has<Checked>,
    disabled: Has<InteractionDisabled>,
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    outline: &'static mut Outline,
    node: &'static mut Node,
    children: Option<&'static Children>,
}
