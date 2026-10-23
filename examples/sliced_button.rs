//! Pixel-font buttons on a white 9-sliced frame, tinted per variant.
//!
//! Execution: cargo run --example sliced_button
//!
//! Needs two assets:
//!   - `assets/images/panel-000.png`: Kenney Fantasy UI Borders (CC0), 48 x 48, ornamented corners, transparent gaps.
//!   - `assets/fonts/PixelMplus12-Regular.ttf`.
//!
//! What to verify:
//!   1. The corner ornaments keep their shape and pixel crispness, whatever the label length or size
//!      (OK / DELETE / NEW INVESTIGATION, large / medium / small). Only the edges stretch.
//!   2. The labels are crisp pixel text, not blurred (`FontSmoothing::None`, sizes are multiples of 12).
//!   3. One white image becomes four colors through the tint. Hover brightens, press darkens, and
//!      CONTINUE (disabled) turns gray.
//!   4. [D] toggles CONTINUE. Tab moves the focus ring around the frame.
//!   5. [S] toggles `FontSmoothing` on the global theme: the labels switch between crisp and smoothed.
//!   6. [T] swaps to `Theme::default()`: the frame disappears, and the plain backgrounds and the default
//!      font return.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    text::Font,
    ui::InteractionDisabled,
};
use bevy_asset_loader::prelude::*;
use bevy_ui_component_prototypes::{
    ButtonMetrics, ButtonShape, ButtonSize, ButtonSizes, ButtonStyle, ButtonStyles, ButtonVariant,
    FocusRing, GlobalTheme, SlicedImage, StateColors, Theme, UiComponentPrototypesPlugin,
    icon_button,
};

#[derive(Component)]
struct ContinueItem;

#[derive(States, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Phase {
    #[default]
    Loading,
    Ready,
}

#[derive(AssetCollection, Resource)]
struct Assets {
    #[asset(path = "images/panel-000.png")]
    frame: Handle<Image>,
    #[asset(path = "fonts/PixelMplus12-Regular.ttf")]
    font: Handle<Font>,
}

/// Whether the sliced theme (rather than `Theme::default()`) is active.
#[derive(Resource)]
struct SlicedLook(bool);

fn main() {
    App::new()
        .add_plugins((
            // The frame and the font are pixel art: sample the image without smoothing.
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TabNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .init_state::<Phase>()
        .add_loading_state(
            LoadingState::new(Phase::Loading)
                .continue_to_state(Phase::Ready)
                .load_collection::<Assets>(),
        )
        .add_systems(OnEnter(Phase::Ready), setup)
        .add_systems(
            Update,
            (
                toggle_continue.run_if(input_just_pressed(KeyCode::KeyD)),
                toggle_smoothing.run_if(input_just_pressed(KeyCode::KeyS)),
                swap_theme.run_if(input_just_pressed(KeyCode::KeyT)),
            )
                .run_if(in_state(Phase::Ready)),
        )
        .run();
}

/// Set the theme once the assets have loaded, before any button exists.
fn setup(mut commands: Commands, assets: Res<Assets>) {
    commands.insert_resource(GlobalTheme(sliced_theme(&assets)));
    commands.insert_resource(SlicedLook(true));
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(16),
                ..default()
            },
            TabGroup::default(),
        ))
        .with_children(|root| {
            for size in [ButtonSize::Large, ButtonSize::Medium, ButtonSize::Small] {
                root.spawn(Node {
                    column_gap: px(12),
                    ..default()
                })
                .with_children(|row| {
                    for (label, variant) in [
                        ("NEW INVESTIGATION", ButtonVariant::Neutral),
                        ("OPTIONS", ButtonVariant::Primary),
                        ("OK", ButtonVariant::Affirmative),
                        ("DELETE", ButtonVariant::Danger),
                    ] {
                        row.spawn(
                            icon_button(label)
                                .size(size.clone())
                                .variant(variant)
                                .build(),
                        );
                    }
                });
            }
            root.spawn((
                icon_button("CONTINUE")
                    .size(ButtonSize::Medium)
                    .variant(ButtonVariant::Primary)
                    .build(),
                InteractionDisabled,
                ContinueItem,
            ));
            root.spawn(Text::new(
                "[Tab] move   [D] toggle CONTINUE   [S] font smoothing   [T] swap theme",
            ));
        });
}

/// One white image, tinted into one color per variant. A white image can be tinted to any color,
/// so the state colors are ordinary colors (hover = the full hue, press = darker).
fn sliced_theme(assets: &Assets) -> Theme {
    // The image is 48 x 48; its corner ornaments fit in 12 x 12, and the 24 x 24 middle is uniform.
    let surface = SlicedImage {
        image: assets.frame.clone(),
        slicer: TextureSlicer {
            border: BorderRect::all(12.0),
            ..default()
        },
    };
    let style = |hue: Color| {
        let base = hue.to_srgba();
        let scaled = |k: f32| Color::srgb(base.red * k, base.green * k, base.blue * k);
        ButtonStyle {
            bg: StateColors {
                normal: scaled(0.8),
                hovered: scaled(1.0),
                pressed: scaled(0.55),
                focused: scaled(0.9),
                disabled: Color::srgb(0.4, 0.4, 0.4),
            },
            fg: StateColors::uniform(Color::srgb(0.08, 0.08, 0.12)),
            // The frame carries its own outline.
            border: StateColors::uniform(Color::NONE),
            focus_ring: FocusRing {
                color: Color::WHITE,
                width: px(2),
                offset: px(2),
            },
            shape: ButtonShape::default(),
            surface: Some(surface.clone()),
        }
    };
    // Pixel fonts stay crisp at integer multiples of their native size (12 px).
    let defaults = ButtonSizes::default();
    let pixel = |metrics: ButtonMetrics, size: f32| ButtonMetrics {
        font_size: FontSize::Px(size),
        ..metrics
    };
    Theme {
        button_sizes: ButtonSizes {
            small: pixel(defaults.small, 12.0),
            medium: pixel(defaults.medium, 24.0),
            large: pixel(defaults.large, 36.0),
        },
        buttons: ButtonStyles {
            neutral: style(Color::srgb(0.55, 0.65, 0.95)),
            primary: style(Color::srgb(0.95, 0.8, 0.45)),
            affirmative: style(Color::srgb(0.55, 0.9, 0.6)),
            danger: style(Color::srgb(0.95, 0.5, 0.5)),
        },
        font: assets.font.clone().into(),
        font_smoothing: FontSmoothing::None,
        ..default() // unrelated theme
    }
}

fn toggle_continue(
    mut commands: Commands,
    items: Query<(Entity, Has<InteractionDisabled>), With<ContinueItem>>,
) {
    for (entity, disabled) in &items {
        if disabled {
            commands.entity(entity).remove::<InteractionDisabled>();
        } else {
            commands.entity(entity).insert(InteractionDisabled);
        }
    }
}

/// Only meaningful under the sliced theme, which is the one that sets a pixel font.
fn toggle_smoothing(look: Res<SlicedLook>, mut theme: ResMut<GlobalTheme>) {
    if !look.0 {
        return;
    }
    theme.font_smoothing = match theme.font_smoothing {
        FontSmoothing::None => FontSmoothing::AntiAliased,
        _ => FontSmoothing::None,
    };
}

fn swap_theme(mut global: ResMut<GlobalTheme>, mut look: ResMut<SlicedLook>, assets: Res<Assets>) {
    look.0 = !look.0;
    *global = GlobalTheme(if look.0 {
        sliced_theme(&assets)
    } else {
        Theme::default()
    });
}
