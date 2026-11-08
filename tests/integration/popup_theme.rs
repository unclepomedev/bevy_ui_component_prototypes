//! Behavior tests for the popup: structure, the pointer, the look and the layer.

use bevy::prelude::*;
use bevy_ui_component_prototypes::popup;

mod buttons;
mod layer;
mod structure;
mod styling;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_popup(app: &mut App, message: &str) -> Entity {
    app.world_mut().spawn(popup().message(message).build()).id()
}

fn panel(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(root).unwrap()[0]
}

fn message(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(panel(app, root)).unwrap()[0]
}
