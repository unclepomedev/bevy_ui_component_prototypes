//! Behavior tests for the slider: styling, the position of the fill and the thumb, theme resolution,
//! focus, tab order, and the controlled-value wiring on top of Bevy's `Slider`.
//!
//! Real pointer picking, layout, and dragging are covered by `examples/slider_check.rs`.

use bevy::prelude::*;
use bevy_ui_component_prototypes::slider::slider;

mod behavior;
mod controlled;
mod editing;
mod focus;
mod initial_style;
mod interaction_states;
mod segments;
mod shape;
mod tab_order;
mod theme;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_slider(app: &mut App) -> Entity {
    app.world_mut().spawn(slider().build()).id()
}

/// The parts, in the order the builder spawns them: the rail, then a travel area holding the fill
/// and the thumb.
fn parts(app: &App, slider: Entity) -> (Entity, Entity, Entity) {
    let world = app.world();
    let children = world.get::<Children>(slider).unwrap();
    let travel = world.get::<Children>(children[1]).unwrap();
    (children[0], travel[0], travel[1])
}

fn color_of(app: &App, entity: Entity) -> Color {
    app.world().get::<BackgroundColor>(entity).unwrap().0
}

fn rail(app: &App, slider: Entity) -> Color {
    color_of(app, parts(app, slider).0)
}

fn fill(app: &App, slider: Entity) -> Color {
    color_of(app, parts(app, slider).1)
}

fn thumb(app: &App, slider: Entity) -> Color {
    color_of(app, parts(app, slider).2)
}

fn fill_width(app: &App, slider: Entity) -> Val {
    app.world().get::<Node>(parts(app, slider).1).unwrap().width
}

fn thumb_left(app: &App, slider: Entity) -> Val {
    app.world().get::<Node>(parts(app, slider).2).unwrap().left
}

fn rail_part(app: &App, slider: Entity) -> Entity {
    parts(app, slider).0
}

fn fill_part(app: &App, slider: Entity) -> Entity {
    parts(app, slider).1
}

fn thumb_part(app: &App, slider: Entity) -> Entity {
    parts(app, slider).2
}

fn travel_part(app: &App, slider: Entity) -> Entity {
    app.world().get::<Children>(slider).unwrap()[1]
}
