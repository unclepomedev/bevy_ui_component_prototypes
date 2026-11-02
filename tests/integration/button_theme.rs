//! Behavior tests for button styling and theme resolution.
//!
//! Real pointer picking, layout, and text shaping are not covered by `examples/button_check.rs`.

use bevy::prelude::*;
use bevy_ui_component_prototypes::{ButtonStyle, ButtonVariantId, Theme, icon_button};

mod behavior;
mod focus;
mod initial_style;
mod interaction_states;
mod size_shape;
mod tab_order;
mod theme;
mod variants;

fn spawn_primary(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(
            icon_button()
                .label("button")
                .variant(ButtonVariantId::PRIMARY)
                .build(),
        )
        .id()
}

fn spawn_primary_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((
            icon_button()
                .label("button")
                .variant(ButtonVariantId::PRIMARY)
                .build(),
            ChildOf(parent),
        ))
        .id()
}

fn label_color(app: &App, button: Entity) -> Color {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextColor>(child).map(|c| c.0))
        .expect("button has a label child")
}

fn label_font(app: &App, button: Entity) -> FontSource {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextFont>(child).map(|f| f.font.clone()))
        .expect("button has a label child")
}

fn primary(theme: &Theme) -> ButtonStyle {
    theme.buttons.named[&ButtonVariantId::PRIMARY].clone()
}

fn primary_mut(theme: &mut Theme) -> &mut ButtonStyle {
    theme
        .buttons
        .named
        .get_mut(&ButtonVariantId::PRIMARY)
        .unwrap()
}
