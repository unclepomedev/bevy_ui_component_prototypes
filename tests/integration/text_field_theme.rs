//! Behavior tests for the text field: its look in each state, the focus ring, the placeholder, the options
//! of the builder, what is forwarded to the input, and theme resolution.
//!
//! Typing and the cursor are covered by `examples/text_field_check.rs`, the real layout by `text_field_layout.rs`.

use bevy::prelude::*;
use bevy_ui_component_prototypes::text_field::text_field;

mod input;
mod look;
mod options;
mod placeholder;
mod structure;

fn spawn_field(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(text_field().placeholder(Text::new("Name")).build())
        .id()
}

// The structure the tests rely on: the input is the first child of the field, and the placeholder the second.
fn input(app: &App, field: Entity) -> Entity {
    app.world().get::<Children>(field).unwrap()[0]
}

fn placeholder(app: &App, field: Entity) -> Entity {
    app.world().get::<Children>(field).unwrap()[1]
}

fn text_color(app: &App, entity: Entity) -> Color {
    app.world().get::<TextColor>(entity).unwrap().0
}
