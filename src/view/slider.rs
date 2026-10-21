mod builder;
mod components;

pub use self::{
    builder::{SliderBuilder, slider},
    components::{SliderEditing, ValueSlider},
};

use self::components::SliderPart;
use super::{
    focus::FocusRingTracker,
    style_targets::{drain_removed, pick_targets},
    theme_lookup::ThemeLookup,
};
use crate::style::{slider::SliderLook, state::VisualState};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
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

/// Rewrites the look of each slider that needs it: which state it is in, what that looks like in its
/// theme, and where that goes (the focus ring on the slider, the colors, and positions on the parts).
fn sync_slider_style(
    mut lookup: ThemeLookup,
    mut ring: FocusRingTracker,
    mut targets: SliderTargets,
    mut sliders: Query<SliderRow, With<ValueSlider>>,
    mut parts: Query<(&SliderPart, &mut BackgroundColor, &mut Node), Without<ValueSlider>>,
    children: Query<&Children>,
) {
    let (focused, focus_moved) = ring.update();
    let targets = targets.collect(lookup.needs_resync(), focus_moved);

    for entity in targets {
        let Ok(mut slider) = sliders.get_mut(entity) else {
            continue;
        };

        let is_focused = focused == Some(entity);
        let state = visual_state(&slider, is_focused);
        let look = lookup
            .for_entity(entity)
            .slider
            .look(state, is_focused, slider.editing);

        write_ring(&mut slider.outline, &look);
        let position = thumb_position(&slider);
        write_parts(&mut parts, &children, entity, &look, position);
    }
}

/// What the sync reads from a slider.
#[derive(QueryData)]
#[query_data(mutable)]
struct SliderRow {
    value: &'static SliderValue,
    range: &'static SliderRange,
    drag: &'static SliderDragState,
    pressed: Has<Pressed>,
    hovered: &'static Hovered,
    disabled: Has<InteractionDisabled>,
    editing: Has<SliderEditing>,
    outline: &'static mut Outline,
}

fn visual_state(slider: &SliderRowItem, is_focused: bool) -> VisualState {
    // Pressing the thumb does not insert `Pressed`; only the drag state tells.
    let pressed = slider.pressed || slider.drag.dragging;
    VisualState::from_flags(pressed, slider.hovered.get(), is_focused, slider.disabled)
}

fn thumb_position(slider: &SliderRowItem) -> Val {
    let share = slider.range.thumb_position(slider.value.0).clamp(0.0, 1.0);
    Val::Percent(share * 100.0)
}

fn write_ring(outline: &mut Mut<Outline>, look: &SliderLook) {
    outline.set_if_neq(Outline::new(
        look.ring.width,
        look.ring.offset,
        look.ring.color,
    ));
}

fn write_parts(
    parts: &mut Query<(&SliderPart, &mut BackgroundColor, &mut Node), Without<ValueSlider>>,
    children: &Query<&Children>,
    slider: Entity,
    look: &SliderLook,
    position: Val,
) {
    for part in children.iter_descendants(slider) {
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

/// Finds the sliders whose look may have changed from the last run.
#[derive(SystemParam)]
struct SliderTargets<'w, 's> {
    changed: ChangedSliders<'w, 's>,
    all: Query<'w, 's, Entity, With<ValueSlider>>,
    released: RemovedComponents<'w, 's, Pressed>,
    enabled: RemovedComponents<'w, 's, InteractionDisabled>,
    stopped_editing: RemovedComponents<'w, 's, SliderEditing>,
}

impl SliderTargets<'_, '_> {
    /// Every slider after a theme change, otherwise only the ones that changed
    /// (including both ends of a focus move).
    fn collect(&mut self, resync: bool, focus_moved: Vec<Entity>) -> Vec<Entity> {
        let mut removed = drain_removed(&mut self.released);
        removed.extend(drain_removed(&mut self.stopped_editing));
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
            Added<SliderEditing>,
        )>,
    ),
>;
