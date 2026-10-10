//! Sample for verifying functionality.
//!
//! Execution: cargo run --example button_check
//!
//! What to verify:
//!   1. The five buttons in the top row change color on hover / press / release and return to normal afterward.
//!   2. [D] toggles InteractionDisabled on "target": it turns gray, clicking it emits no `Activated`, and removing the component restores it.
//!   3. [T] swaps the global theme (colors and font). The top row and "target" change; the framed panel, which has its own `Theme`, does not.
//!   4. [S] swaps the framed panel's own `Theme`. Only the panel changes.
//!   5. [N] spawns a new button inside the panel at runtime. It must appear already styled with the panel's theme (no unstyled flash).

use bevy::{prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::{
    Activated, ButtonSize, ButtonStyles, ButtonVariant, GlobalTheme, Theme,
    UiComponentPrototypesPlugin, icon_button,
};

/// Marker that identifies which button emitted an `Activated` event.
#[derive(Component)]
struct Tag(&'static str);

/// Target whose disabled state is toggled with [D].
#[derive(Component)]
struct DisableTarget;

/// Panel that carries its own `Theme` component.
#[derive(Component)]
struct ScopedPanel;

/// Text showing the most recently activated button.
#[derive(Component)]
struct StatusText;

#[derive(Resource, Default)]
struct ActivationCount(u32);

/// The current state of the [T] / [S] toggles (false = default palette).
#[derive(Default)]
struct SwapState {
    global_swapped: bool,
    panel_swapped: bool,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UiComponentPrototypesPlugin))
        .init_resource::<ActivationCount>()
        .add_systems(Startup, setup)
        .add_systems(Update, hotkeys)
        .add_observer(on_activated)
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
            // 1. One button per variant (follows the global theme).
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
                            .build(),
                        Tag(label),
                    ));
                }
            });

            // 2. Target for the disabled toggle (follows the global theme).
            root.spawn((
                icon_button("target (toggle disabled with D)")
                    .size(ButtonSize::small())
                    .variant(ButtonVariant::Primary)
                    .build(),
                Tag("target"),
                DisableTarget,
            ));

            // 3. Panel with its own `Theme` (unaffected by the global theme).
            root.spawn((
                Node {
                    column_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                Theme::default(),
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
                            .build(),
                        Tag(label),
                    ));
                }
            });

            root.spawn(Text::new(
                "[D] toggle disabled on target\n\
                 [T] swap GLOBAL theme (colors + font); the framed panel is unaffected\n\
                 [S] swap the framed panel's own Theme only\n\
                 [N] spawn a new button inside the framed panel",
            ));
            root.spawn((Text::new("Activated: (nothing yet)"), StatusText));
        });
}

fn on_activated(
    trigger: On<Activated>,
    tags: Query<&Tag>,
    mut count: ResMut<ActivationCount>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    count.0 += 1;
    let name = tags.get(trigger.entity).map(|t| t.0).unwrap_or("?");
    for mut text in &mut status {
        text.0 = format!("Activated: {name} (total {})", count.0);
    }
}

fn hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    targets: Query<(Entity, Has<InteractionDisabled>), With<DisableTarget>>,
    mut global: ResMut<GlobalTheme>,
    mut panels: Query<(Entity, &mut Theme), With<ScopedPanel>>,
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
        for (_, mut theme) in &mut panels {
            *theme = theme_for(state.panel_swapped);
        }
    }

    if keys.just_pressed(KeyCode::KeyN) {
        for (panel, _) in &panels {
            commands.spawn((
                icon_button("New")
                    .size(ButtonSize::medium())
                    .variant(ButtonVariant::Danger)
                    .build(),
                Tag("New"),
                ChildOf(panel),
            ));
        }
    }
}

/// When `swapped`, returns a theme with the Primary/Affirmative palettes
/// exchanged and a different font, so that the change is easy to see.
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
        font: FontSource::sans_serif(),
    }
}
