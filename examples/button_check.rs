//! Sample for verifying functionality.
//!
//! Execution: cargo run --example button_check
//!
//! Things to verify:
//!   1. The five buttons in the top row change color on hover/press/release and revert to their original state.
//!   2. Press the [D] key to toggle `InteractionDisabled` for the "target" button in the bottom row
//!      (turns gray / clicking does not trigger the `Pressed` state / reverts when disabled).
//!   3. Press the [T] key to swap the theme (swaps the color schemes of Primary and Affirmative).
//!   4. Replace only the panel's Theme component using [S]. Only the panel changes.

use bevy::{prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::{
    Activated, ButtonSize, ButtonStyles, ButtonVariant, GlobalTheme, Theme,
    UiComponentPrototypesPlugin, icon_button,
};

/// A visual indicator that shows which button was pressed when an event occurs.
#[derive(Component)]
struct Tag(&'static str);

/// The target for toggling "disabled" using [D].
#[derive(Component)]
struct DisableTarget;

/// A panel with its own theme.
#[derive(Component)]
struct ScopedPanel;

/// Text displaying the last button pressed.
#[derive(Component)]
struct StatusText;

#[derive(Resource, Default)]
struct PressCount(u32);

/// Current state of [T] / [S] (false = default color scheme, true = swapped).
#[derive(Default)]
struct SwapState {
    global_swapped: bool,
    panel_swapped: bool,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UiComponentPrototypesPlugin))
        .init_resource::<PressCount>()
        .add_systems(Startup, setup)
        .add_systems(Update, hotkeys)
        .add_observer(on_pressed)
        .run();
}

// TODO extract components
fn setup(mut commands: Commands, global: Res<GlobalTheme>) {
    commands.spawn(Camera2d);

    let mut custom = global.buttons.neutral;
    custom.fg.normal = Color::srgb(1.0, 0.85, 0.3);
    custom.fg.hovered = Color::srgb(1.0, 0.95, 0.6);
    custom.fg.pressed = Color::srgb(0.8, 0.65, 0.2);

    let variants: [(&'static str, ButtonVariant); 5] = [
        ("Neutral", ButtonVariant::Neutral),
        ("Primary", ButtonVariant::Primary),
        ("Affirmative", ButtonVariant::Affirmative),
        ("Danger", ButtonVariant::Danger),
        ("Custom", ButtonVariant::Custom(Box::new(custom))),
    ];

    let panel_theme = Theme::default();

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|root| {
            root.spawn(Node {
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|row| {
                for (label, variant) in variants {
                    row.spawn((
                        icon_button(label)
                            .size(ButtonSize::medium())
                            .variant(variant)
                            .build(&global),
                        Tag(label),
                    ));
                }
            });

            root.spawn((
                icon_button("target (toggle disabled with D)")
                    .size(ButtonSize::small())
                    .variant(ButtonVariant::Primary)
                    .build(&global),
                Tag("target"),
                DisableTarget,
            ));

            root.spawn((
                Node {
                    column_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                panel_theme.clone(),
                ScopedPanel,
            ))
            .with_children(|panel| {
                for (label, variant) in [
                    ("Scoped Primary", ButtonVariant::Primary),
                    ("Scoped Affirmative", ButtonVariant::Affirmative),
                ] {
                    panel.spawn((
                        icon_button(label)
                            .size(ButtonSize::medium())
                            .variant(variant)
                            .build(&panel_theme),
                        Tag(label),
                    ));
                }
            });

            root.spawn(Text::new(
                "[D] toggle disabled on target\n\
                 [T] swap GLOBAL theme (Primary <-> Affirmative); the framed panel is unaffected\n\
                 [S] swap the framed panel's own Theme only",
            ));
            root.spawn((Text::new("Pressed: (nothing yet)"), StatusText));
        });
}

fn on_pressed(
    trigger: On<Activated>,
    tags: Query<&Tag>,
    mut count: ResMut<PressCount>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    count.0 += 1;
    let name = tags.get(trigger.entity).map(|t| t.0).unwrap_or("?");
    for mut text in &mut status {
        text.0 = format!("Pressed: {name} (total {})", count.0);
    }
}

fn hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    targets: Query<(Entity, Has<InteractionDisabled>), With<DisableTarget>>,
    mut global: ResMut<GlobalTheme>,
    mut panels: Query<&mut Theme, With<ScopedPanel>>,
    mut state: Local<SwapState>,
) {
    if keys.just_pressed(KeyCode::KeyD) {
        for (entity, disabled) in &targets {
            if disabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
    }

    if keys.just_pressed(KeyCode::KeyT) {
        state.global_swapped = !state.global_swapped;
        *global = GlobalTheme(theme_for(state.global_swapped));
    }

    if keys.just_pressed(KeyCode::KeyS) {
        state.panel_swapped = !state.panel_swapped;
        for mut theme in &mut panels {
            *theme = theme_for(state.panel_swapped);
        }
    }
}

/// "swapped" refers to a theme where the Primary and Affirmative color schemes are swapped.
fn theme_for(swapped: bool) -> Theme {
    let base = Theme::default();
    if !swapped {
        return base;
    }
    Theme {
        buttons: ButtonStyles {
            primary: base.buttons.affirmative,
            affirmative: base.buttons.primary,
            ..base.buttons.clone()
        },
        ..base
    }
}
