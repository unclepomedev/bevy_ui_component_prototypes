//! A button theme with a pixel font loaded through `bevy_asset_loader`.
//!
//! Execution: cargo run --example fonted_button
//!
//! Needs `assets/fonts/PixelMplus12-Regular.ttf`.
//!
//! What to verify:
//!   1. The labels are crisp pixel text, not blurred (`FontSmoothing::None`, size = 12 x 2).
//!   2. Hover / press / Tab focus behave as in `button_check`.
//!   3. [S] toggles `FontSmoothing` on the global theme: the labels switch between crisp and smoothed.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    text::Font,
};
use bevy_asset_loader::prelude::*;
use bevy_ui_component_prototypes::{
    ButtonSize, GlobalTheme, Theme, UiComponentPrototypesPlugin, icon_button,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TabNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .init_state::<Phase>()
        .add_loading_state(
            LoadingState::new(Phase::Loading)
                .continue_to_state(Phase::Ready)
                .load_collection::<PixelFonts>(),
        )
        .add_systems(OnEnter(Phase::Ready), (use_pixel_font, setup).chain())
        .add_systems(
            Update,
            toggle_smoothing
                .run_if(in_state(Phase::Ready))
                .run_if(input_just_pressed(KeyCode::KeyS)),
        )
        .run();
}

#[derive(States, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Phase {
    #[default]
    Loading,
    Ready,
}

#[derive(AssetCollection, Resource)]
struct PixelFonts {
    #[asset(path = "fonts/PixelMplus12-Regular.ttf")]
    main: Handle<Font>,
}

/// Set the theme once the font has finished loading, before any button exists.
fn use_pixel_font(mut commands: Commands, fonts: Res<PixelFonts>) {
    commands.insert_resource(GlobalTheme(Theme {
        font: fonts.main.clone().into(),
        font_smoothing: FontSmoothing::None,
        ..default()
    }));
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(16),
                ..default()
            },
            TabGroup::default(),
        ))
        .with_children(|root| {
            // Pixel fonts stay crisp at integer multiples of their native size 12 px.
            let size = ButtonSize {
                font_size: FontSize::Px(24.0),
                ..ButtonSize::medium()
            };
            root.spawn(icon_button("back").size(size).build());
            root.spawn(icon_button("next").size(size).build());
        });
}

fn toggle_smoothing(mut theme: ResMut<GlobalTheme>) {
    theme.font_smoothing = match theme.font_smoothing {
        FontSmoothing::None => FontSmoothing::AntiAliased,
        _ => FontSmoothing::None,
    };
}
