mod builder;
mod components;

pub use self::{
    builder::{SliderBuilder, slider},
    components::{SliderEditing, ValueSlider},
};

use self::components::SliderPart;
use super::{focus::FocusRingTracker, theme_lookup::ThemeLookup};
use crate::style::state::VisualState;
use bevy::{
    ecs::system::SystemParam,
    picking::hover::Hovered,
    prelude::*,
    ui::{InteractionDisabled, Pressed, UiSystems},
    ui_widgets::{SliderDragState, SliderRange, SliderValue},
};

pub(crate) struct SliderStylePlugin;
impl Plugin for SliderStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_slider_style.before(UiSystems::Prepare));
    }
}

/// Resolves each slider's look from its theme and state and writes it, together with the position of
/// the fill and the thumb, to the parts.
fn sync_slider_style(
    mut lookup: ThemeLookup,
    mut ring: FocusRingTracker,
    mut targets: SliderTargets,
    mut sliders: SliderQuery,
    mut parts: Query<(&SliderPart, &mut BackgroundColor, &mut Node), Without<ValueSlider>>,
    children: Query<&Children>,
) {
    let (focused, focus_moved) = ring.update();
    let targets = targets.collect(lookup.needs_resync(), focus_moved);

    for entity in targets {
        let Ok((value, range, drag, pressed, hovered, disabled, mut outline)) =
            sliders.get_mut(entity)
        else {
            continue;
        };

        let style = lookup.for_entity(entity).slider;
        let is_focused = focused == Some(entity);
        // Pressing the thumb does not insert `Pressed`; only the drag state tells.
        let state = VisualState::from_flags(
            pressed || drag.dragging,
            hovered.get(),
            is_focused,
            disabled,
        );
        let look = style.look(state, is_focused);

        outline.set_if_neq(Outline::new(
            look.ring.width,
            look.ring.offset,
            look.ring.color,
        ));

        let position = Val::Percent(range.thumb_position(value.0).clamp(0.0, 1.0) * 100.0);
        for part in children.iter_descendants(entity) {
            let Ok((kind, mut bg, mut node)) = parts.get_mut(part) else {
                continue;
            };
            match kind {
                SliderPart::Rail => {
                    bg.set_if_neq(BackgroundColor(look.rail));
                }
                SliderPart::Fill => {
                    bg.set_if_neq(BackgroundColor(look.fill));
                    if node.width != position {
                        node.width = position;
                    }
                }
                SliderPart::Thumb => {
                    bg.set_if_neq(BackgroundColor(look.thumb));
                    if node.left != position {
                        node.left = position;
                    }
                }
            }
        }
    }
}

/// Finds the sliders whose look may have changed from the last run.
#[derive(SystemParam)]
struct SliderTargets<'w, 's> {
    changed: ChangedSliders<'w, 's>,
    all: Query<'w, 's, Entity, With<ValueSlider>>,
    released: RemovedComponents<'w, 's, Pressed>,
    enabled: RemovedComponents<'w, 's, InteractionDisabled>,
}

impl SliderTargets<'_, '_> {
    /// Every slider after a theme change, otherwise only the ones that changed
    /// (including both ends of a focus move).
    fn collect(&mut self, resync: bool, focus_moved: Vec<Entity>) -> Vec<Entity> {
        // Read every time, so that a removal is never seen twice.
        let released: Vec<Entity> = self.released.read().collect();
        let enabled: Vec<Entity> = self.enabled.read().collect();
        if resync {
            self.all.iter().collect()
        } else {
            self.changed
                .iter()
                .chain(released)
                .chain(enabled)
                .chain(focus_moved)
                .collect()
        }
    }
}

type ChangedSliders<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<ValueSlider>,
        Or<(
            Added<ValueSlider>,
            Changed<SliderValue>,
            Changed<SliderRange>,
            Changed<SliderDragState>,
            Changed<Hovered>,
            Added<Pressed>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

type SliderQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static SliderValue,
        &'static SliderRange,
        &'static SliderDragState,
        Has<Pressed>,
        &'static Hovered,
        Has<InteractionDisabled>,
        &'static mut Outline,
    ),
    With<ValueSlider>,
>;
