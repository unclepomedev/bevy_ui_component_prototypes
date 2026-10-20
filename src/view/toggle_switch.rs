mod builder;
mod components;

pub use self::builder::{ToggleSwitchBuilder, toggle_switch};
pub use self::components::ToggleSwitch;

use self::components::{SwitchThumb, thumb_side};
use super::{focus::FocusRingTracker, theme_lookup::ThemeLookup};
use crate::style::state::VisualState;
use bevy::{
    ecs::system::SystemParam,
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
        let Ok((
            interaction,
            checked,
            disabled,
            mut bg,
            mut border,
            mut outline,
            mut node,
            children,
        )) = switches.get_mut(entity)
        else {
            continue;
        };

        let style = lookup.for_entity(entity).switch;
        let is_focused = focused == Some(entity);
        let state = VisualState::resolve(interaction, is_focused, disabled);
        let look = style.look(state, checked, is_focused);

        bg.set_if_neq(BackgroundColor(look.track));
        border.set_if_neq(BorderColor::all(look.border));
        outline.set_if_neq(Outline::new(
            look.ring.width,
            look.ring.offset,
            look.ring.color,
        ));
        let side = thumb_side(checked);
        if node.justify_content != side {
            node.justify_content = side;
        }
        for child in children.into_iter().flat_map(|children| children.iter()) {
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
        // Read every time, so that a removal is never seen twice.
        let unchecked: Vec<Entity> = self.unchecked.read().collect();
        let enabled: Vec<Entity> = self.enabled.read().collect();
        if resync {
            self.all.iter().collect()
        } else {
            self.changed
                .iter()
                .chain(unchecked)
                .chain(enabled)
                .chain(focus_moved)
                .collect()
        }
    }
}

type ChangedSwitches<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<ToggleSwitch>,
        Or<(
            Changed<PickingInteraction>,
            Changed<Checked>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

type SwitchQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static PickingInteraction,
        Has<Checked>,
        Has<InteractionDisabled>,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
        &'static mut Outline,
        &'static mut Node,
        Option<&'static Children>,
    ),
    With<ToggleSwitch>,
>;
