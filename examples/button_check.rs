//! Sample for verifying functionality.
//!
//! Execution: cargo run --example button_check
//!
//! What to verify:
//!   1. The five buttons in the top row change color on hover / press / release and return to normal afterward.
//!   2. [D] toggles InteractionDisabled on "target": it turns gray, clicking it emits no `Activate`, and removing the component restores it.
//!   3. [T] swaps the global theme (colors and font). The top row and "target" change; the framed panel, which has its own `Theme`, does not.
//!   4. [S] swaps the framed panel's own `Theme`. Only the panel changes.
//!   5. [N] spawns a new button inside the panel at runtime. It must appear already styled with the panel's theme (no unstyled flash).
//!   6. Tab moves keyboard focus between buttons, and a focus ring appears around the focused one;
//!      Enter / Space on it emits `Activate`. Clicking a button hides the ring again.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::Activate,
};
use bevy_ui_component_prototypes::{
    ButtonSizeId, ButtonStyles, ButtonVariant, GlobalTheme, Theme, UiComponentPrototypesPlugin,
    icon_button,
};

/// Marker that identifies which button emitted an `Activate` event.
#[derive(Component)]
struct Tag {
    name: &'static str,
}

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
struct ActivationCount {
    total: u32,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TabNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .init_resource::<ActivationCount>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_disabled.run_if(input_just_pressed(KeyCode::KeyD)),
                swap_global_theme.run_if(input_just_pressed(KeyCode::KeyT)),
                swap_panel_theme.run_if(input_just_pressed(KeyCode::KeyS)),
                spawn_panel_button.run_if(input_just_pressed(KeyCode::KeyN)),
            ),
        )
        .add_observer(on_activated)
        .run();
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands, global: Res<GlobalTheme>) {
    commands.spawn(Camera2d);

    let custom = custom_variant(&global);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            TabGroup::default(),
        ))
        .with_children(|root| {
            spawn_variant_row(root, custom);
            spawn_disable_target(root);
            spawn_scoped_panel(root);
            spawn_help(root);
        });
}

/// A `Custom` variant: the neutral style with only the text colors changed.
fn custom_variant(global: &GlobalTheme) -> ButtonVariant {
    let mut style = global.buttons.neutral.clone();
    style.fg.normal = Color::srgb(1.0, 0.85, 0.3);
    style.fg.hovered = Color::srgb(1.0, 0.95, 0.6);
    style.fg.pressed = Color::srgb(0.8, 0.65, 0.2);
    ButtonVariant::Custom(Box::new(style))
}

/// One button per variant. Follows the global theme.
fn spawn_variant_row(root: &mut ChildSpawnerCommands, custom: ButtonVariant) {
    let variants = [
        ("Neutral", ButtonVariant::Neutral),
        ("Primary", ButtonVariant::Primary),
        ("Affirmative", ButtonVariant::Affirmative),
        ("Danger", ButtonVariant::Danger),
        ("Custom", custom),
    ];

    root.spawn(Node {
        column_gap: Val::Px(16.0),
        ..default()
    })
    .with_children(|row| {
        for (label, variant) in variants {
            row.spawn((
                icon_button(label)
                    .size(ButtonSizeId::MEDIUM)
                    .variant(variant)
                    .build(),
                Tag { name: label },
            ));
        }
    });
}

/// Target for the disabled toggle. Follows the global theme.
fn spawn_disable_target(root: &mut ChildSpawnerCommands) {
    root.spawn((
        icon_button("target (toggle disabled with D)")
            .size(ButtonSizeId::SMALL)
            .variant(ButtonVariant::Primary)
            .build(),
        Tag { name: "target" },
        DisableTarget,
    ));
}

/// Framed panel with its own `Theme` (unaffected by the global theme).
fn spawn_scoped_panel(root: &mut ChildSpawnerCommands) {
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
                    .size(ButtonSizeId::MEDIUM)
                    .variant(variant)
                    .build(),
                Tag { name: label },
            ));
        }
    });
}

/// Key bindings and the status line.
fn spawn_help(root: &mut ChildSpawnerCommands) {
    root.spawn(Text::new(
        "[D] toggle disabled on target\n\
         [T] swap GLOBAL theme (colors + font); the framed panel is unaffected\n\
         [S] swap the framed panel's own Theme only\n\
         [N] spawn a new button inside the framed panel\n\
         [Tab] move focus   [Enter / Space] activate the focused button",
    ));
    root.spawn((Text::new("Activate: (nothing yet)"), StatusText));
}

// ---------------------------------------------------------------------------
// Behavior
// ---------------------------------------------------------------------------

fn on_activated(
    trigger: On<Activate>,
    tags: Query<&Tag>,
    mut count: ResMut<ActivationCount>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    count.total += 1;
    let name = tags.get(trigger.entity).map(|t| t.name).unwrap_or("?");
    for mut text in &mut status {
        text.0 = format!("Activate: {name} (total {})", count.total);
    }
}

fn toggle_disabled(
    mut commands: Commands,
    targets: Query<(Entity, Has<InteractionDisabled>), With<DisableTarget>>,
) {
    for (entity, disabled) in &targets {
        if disabled {
            commands.entity(entity).remove::<InteractionDisabled>();
        } else {
            commands.entity(entity).insert(InteractionDisabled);
        }
    }
}

fn swap_global_theme(mut global: ResMut<GlobalTheme>, mut swapped: Local<bool>) {
    *swapped = !*swapped;
    *global = GlobalTheme(theme_for(*swapped));
}

fn swap_panel_theme(mut panels: Query<&mut Theme, With<ScopedPanel>>, mut swapped: Local<bool>) {
    *swapped = !*swapped;
    for mut theme in &mut panels {
        *theme = theme_for(*swapped);
    }
}

fn spawn_panel_button(mut commands: Commands, panels: Query<Entity, With<ScopedPanel>>) {
    for panel in &panels {
        commands.spawn((
            icon_button("New")
                .size(ButtonSizeId::MEDIUM)
                .variant(ButtonVariant::Danger)
                .build(),
            Tag { name: "New" },
            ChildOf(panel),
        ));
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
            ..base.buttons
        },
        font: FontSource::sans_serif(),
        ..default()
    }
}
