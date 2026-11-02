//! Behavior tests for the toast: structure and the pointer.
//!
//! Colors, sizes and the theme come with the style sync; the slide and the real screen are covered by
//! `examples/toast_check.rs`.

use crate::support::test_app;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use bevy_ui_component_prototypes::{Toast, toast};
use std::time::Duration;

mod host;
mod life;
mod limit;
mod stacking;
mod structure;
mod styling;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_toast(app: &mut App, text: &str) -> Entity {
    app.world_mut().spawn(toast().text(text).build()).id()
}

fn label(app: &App, toast: Entity) -> Entity {
    app.world().get::<Children>(toast).unwrap()[0]
}

fn text(app: &App, toast: Entity) -> String {
    app.world()
        .get::<Text>(label(app, toast))
        .unwrap()
        .0
        .clone()
}

fn label_color(app: &App, toast: Entity) -> Color {
    app.world().get::<TextColor>(label(app, toast)).unwrap().0
}

/// An app whose clock moves by `step` seconds every update.
fn timed_app(step: f32) -> App {
    let mut app = test_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        step,
    )));
    // The first update only sets the clock; there is no delta before the second.
    app.update();
    app
}

fn run(app: &mut App, updates: usize) {
    for _ in 0..updates {
        app.update();
    }
}

fn alive(app: &App, entity: Entity) -> bool {
    app.world().get_entity(entity).is_ok()
}

/// Inside the hold of the probe theme (0.5 to 2.5): fully visible.
const HELD: f32 = 1.0;

fn settle(app: &mut App, toast: Entity) -> Entity {
    app.world_mut().entity_mut(toast).insert(Toast {
        elapsed: HELD,
        duration: None,
    });
    toast
}

fn set_elapsed(app: &mut App, toast: Entity, elapsed: f32) {
    app.world_mut().entity_mut(toast).insert(Toast {
        elapsed,
        duration: None,
    });
}

fn elapsed(app: &App, toast: Entity) -> f32 {
    app.world().get::<Toast>(toast).unwrap().elapsed
}
