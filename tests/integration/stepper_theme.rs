//! Behavior tests for the stepper: styling, theme resolution, focus, and the controlled-value
//! wiring on top of Bevy's `Button`.
//!
//! Real pointer picking and layout are covered by `examples/stepper_check.rs`.

use bevy::{prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::stepper::{StepperBuilder, stepper};

mod disabled;
mod initial_style;
mod input;
mod layout;
mod runtime;
mod states;
mod tab_order;
mod theme;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_stepper(app: &mut App, builder: StepperBuilder) -> Entity {
    app.world_mut().spawn(builder.build()).id()
}

fn spawn_default(app: &mut App) -> Entity {
    spawn_stepper(app, stepper().steps(5).value(2))
}

fn children_of(app: &App, entity: Entity) -> Vec<Entity> {
    app.world()
        .get::<Children>(entity)
        .map(|children| children.iter().collect())
        .unwrap_or_default()
}

/// The row with the buttons, and the label between them if there is one.
fn header(app: &App, stepper: Entity) -> Vec<Entity> {
    children_of(app, children_of(app, stepper)[0])
}

fn prev_button(app: &App, stepper: Entity) -> Entity {
    header(app, stepper)[0]
}

fn next_button(app: &App, stepper: Entity) -> Entity {
    *header(app, stepper).last().unwrap()
}

/// The cells, in order. The header is the first child and the track the second.
fn cells(app: &App, stepper: Entity) -> Vec<Entity> {
    children_of(app, children_of(app, stepper)[1])
}

/// The mark under `cell`.
fn mark(app: &App, cell: Entity) -> Entity {
    children_of(app, cell)[0]
}

fn color_of(app: &App, entity: Entity) -> Color {
    app.world().get::<BackgroundColor>(entity).unwrap().0
}

fn is_disabled(app: &App, entity: Entity) -> bool {
    app.world().get::<InteractionDisabled>(entity).is_some()
}

/// Every entity under `root`, not including it.
fn descendants(app: &App, root: Entity) -> Vec<Entity> {
    let mut found = Vec::new();
    let mut waiting = children_of(app, root);
    while let Some(entity) = waiting.pop() {
        found.push(entity);
        waiting.extend(children_of(app, entity));
    }
    found
}
