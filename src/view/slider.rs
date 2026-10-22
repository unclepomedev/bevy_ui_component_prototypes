mod builder;
mod components;

pub use self::{
    builder::{SliderBuilder, slider},
    components::{SliderEditing, ValueSlider},
};

use self::components::{SliderPart, fill_node, rail_node, slider_node, thumb_node, travel_node};
use super::{
    focus::FocusRingTracker,
    style_targets::{drain_removed, pick_targets},
    theme_lookup::ThemeLookup,
};
use crate::style::{
    slider::{SliderGeometry, SliderLook},
    state::VisualState,
};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    picking::hover::Hovered,
    prelude::*,
    ui::{ColorStop, Gradient, InteractionDisabled, LinearGradient, Pressed, UiSystems},
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
    mut parts: Query<PartRow, Without<ValueSlider>>,
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
        let style = &lookup.for_entity(entity).slider;
        let look = style.look(state, is_focused, slider.editing);
        let geometry = style.shape.geometry();

        write_ring(&mut slider.outline, &look);
        write_height(&mut slider.node, &geometry);
        let share = thumb_share(&slider);
        write_parts(&mut parts, &children, entity, &look, &geometry, share);
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
    node: &'static mut Node,
}

fn visual_state(slider: &SliderRowItem, is_focused: bool) -> VisualState {
    // Pressing the thumb does not insert `Pressed`; only the drag state tells.
    let pressed = slider.pressed || slider.drag.dragging;
    VisualState::from_flags(pressed, slider.hovered.get(), is_focused, slider.disabled)
}

/// Where the thumb and the end of the fill are, as a share (0 to 1) of the travel.
fn thumb_share(slider: &SliderRowItem) -> f32 {
    slider.range.thumb_position(slider.value.0).clamp(0.0, 1.0)
}

fn write_ring(outline: &mut Mut<Outline>, look: &SliderLook) {
    outline.set_if_neq(Outline::new(
        look.ring.width,
        look.ring.offset,
        look.ring.color,
    ));
}

fn write_height(node: &mut Mut<Node>, geometry: &SliderGeometry) {
    let want = slider_node(geometry);
    if node.height != want.height {
        node.height = want.height;
    }
}

/// Colors every part and moves the fill and the thumb.
fn write_parts(
    parts: &mut Query<PartRow, Without<ValueSlider>>,
    children: &Query<&Children>,
    slider: Entity,
    look: &SliderLook,
    geometry: &SliderGeometry,
    share: f32,
) {
    let position = Val::Percent(share * 100.0);
    for part in children.iter_descendants(slider) {
        let Ok(mut part) = parts.get_mut(part) else {
            continue;
        };
        match part.kind {
            SliderPart::Rail => write_rail(&mut part, look, geometry),
            SliderPart::Travel => write_travel(&mut part, geometry),
            SliderPart::Fill => write_fill(&mut part, look, geometry, share, position),
            SliderPart::Thumb => write_thumb(&mut part, look, geometry, position),
        }
    }
}

fn write_rail(part: &mut PartRowItem, look: &SliderLook, geometry: &SliderGeometry) {
    write_bar(&mut part.node, &rail_node(geometry));
    match geometry.segments {
        Some(segments) => {
            let stops = segments.stops(segments.count, look.rail);
            paint_blocks(part, stops);
        }
        None => paint_solid(part, look.rail),
    }
}

fn write_travel(part: &mut PartRowItem, geometry: &SliderGeometry) {
    let want = travel_node(geometry);
    if part.node.right != want.right {
        part.node.right = want.right;
    }
}

fn write_fill(
    part: &mut PartRowItem,
    look: &SliderLook,
    geometry: &SliderGeometry,
    share: f32,
    position: Val,
) {
    write_bar(&mut part.node, &fill_node(geometry));
    // A segmented fill spans the whole bar and lights its blocks; a solid one grows.
    let width = match geometry.segments {
        Some(segments) => {
            paint_blocks(part, segments.stops(segments.lit(share), look.fill));
            Val::Percent(100.0)
        }
        None => {
            paint_solid(part, look.fill);
            position
        }
    };
    if part.node.width != width {
        part.node.width = width;
    }
}

fn write_thumb(
    part: &mut PartRowItem,
    look: &SliderLook,
    geometry: &SliderGeometry,
    position: Val,
) {
    part.bg.set_if_neq(BackgroundColor(look.thumb));
    let want = thumb_node(geometry);
    write_bar(&mut part.node, &want);
    if part.node.width != want.width {
        part.node.width = want.width;
    }
    if part.node.left != position {
        part.node.left = position;
    }
}

/// What the sync reads from a part of a slider.
#[derive(QueryData)]
#[query_data(mutable)]
struct PartRow {
    kind: &'static SliderPart,
    bg: &'static mut BackgroundColor,
    node: &'static mut Node,
    // Only the rail and the fill carry one.
    gradient: Option<&'static mut BackgroundGradient>,
}

/// One color over the whole part.
fn paint_solid(part: &mut PartRowItem, color: Color) {
    part.bg.set_if_neq(BackgroundColor(color));
    set_gradient(&mut part.gradient, Vec::new());
}

/// Blocks given as gradient stops; the plain background stays out of the way.
fn paint_blocks(part: &mut PartRowItem, stops: Vec<ColorStop>) {
    part.bg.set_if_neq(BackgroundColor(Color::NONE));
    set_gradient(
        &mut part.gradient,
        vec![LinearGradient::to_right(stops).into()],
    );
}

fn set_gradient(gradient: &mut Option<Mut<BackgroundGradient>>, want: Vec<Gradient>) {
    if let Some(gradient) = gradient
        && gradient.0 != want
    {
        gradient.0 = want;
    }
}

fn write_bar(node: &mut Mut<Node>, want: &Node) {
    if node.top != want.top {
        node.top = want.top;
    }
    if node.height != want.height {
        node.height = want.height;
    }
    if node.border_radius != want.border_radius {
        node.border_radius = want.border_radius;
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
