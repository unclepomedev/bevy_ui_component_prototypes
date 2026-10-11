//! The same menu as `terminal_menu`'s buttons, spawned as BSN scenes.
//!
//! Execution: cargo run --example bsn_menu --features bsn
//!
//! What to verify: the buttons look and behave like the ones in `button_check`
//! (hover, press, Tab focus ring, disabled CONTINUE).

use bevy::{
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::Activate,
};
use bevy_ui_component_prototypes::{
    ButtonSize, ButtonVariant, UiComponentPrototypesPlugin, icon_button_scene,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TabNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .add_systems(Startup, menu.spawn())
        .run();
}

fn menu() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @root()
    }
}

fn root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(8),
        }
        TabGroup
        Children [
            @item("NEW INVESTIGATION", ButtonVariant::Primary)
            on(|_: On<Activate>| info!("Activate: NEW INVESTIGATION"))
            --
            // Extra components are patched onto the scene like any other entry.
            @item("CONTINUE", ButtonVariant::Neutral)
            InteractionDisabled
            --
            @item("EXIT", ButtonVariant::Danger)
            on(|_: On<Activate>| info!("Activate: EXIT"))
        ]
    }
}

fn item(label: &'static str, variant: ButtonVariant) -> impl Scene {
    icon_button_scene(label, ButtonSize::medium(), variant)
}
