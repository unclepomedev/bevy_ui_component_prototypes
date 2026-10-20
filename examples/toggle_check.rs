//! Sample for verifying the toggle switch.
//!
//! Execution: cargo run --example toggle_check
//!
//! The switch is controlled: it only reports `ValueChange<bool>`, and this example's observer decides
//! what to do with it (insert / remove `Checked`). That policy lives here, not in the crate.
//!
//! What to verify:
//!   1. Clicking "Sound" or "Music" flips it: the track changes color and the thumb slides to the other
//!      end. "Music" starts on (it was spawned with `Checked`).
//!   2. Hover brightens the track, and press darkens it, in both the on and the off state.
//!   3. "Locked" asks for a change on click, but the observer refuses it: the status line shows the
//!      request, and the switch stays where it is. This is the controlled model at work.
//!   4. Tab / Shift+Tab move keyboard focus between the switches, and a focus ring appears around the
//!      focused one; Enter / Space asks for a change. Clicking hides the ring again.
//!   5. The two "Disabled" switches (spawned with `InteractionDisabled`, off and on) are gray, show no
//!      hover / press / focus ring, make no request when clicked, and Tab skips them. Unlike "Locked", the
//!      widget itself refuses here.
//!   6. [T] swaps the global theme (the on color turns orange). The framed panel, which has its own
//!      `Theme`, does not change.
//!   7. [S] swaps the framed panel's own `Theme`. Only the panel changes.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::{Checked, InteractionDisabled},
    ui_widgets::ValueChange,
};
use bevy_ui_component_prototypes::{
    GlobalTheme, StateColors, Theme, UiComponentPrototypesPlugin, toggle_switch,
};

/// Name of the switch, shown in the status line.
#[derive(Component)]
struct Tag(&'static str);

/// The app refuses every change request made by this switch.
#[derive(Component)]
struct Locked;

/// Panel that carries its own `Theme`.
#[derive(Component)]
struct ScopedPanel;

#[derive(Component)]
struct StatusText;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TabNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                swap_global_theme.run_if(input_just_pressed(KeyCode::KeyT)),
                swap_panel_theme.run_if(input_just_pressed(KeyCode::KeyS)),
            ),
        )
        .add_observer(on_value_change)
        .run();
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands) {
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
            spawn_row(root, "Sound", false, ());
            spawn_row(root, "Music (starts on)", true, ());
            spawn_row(root, "Locked (the app refuses changes)", false, Locked);
            spawn_row(root, "Disabled (off)", false, InteractionDisabled);
            spawn_row(root, "Disabled (on)", true, InteractionDisabled);
            spawn_scoped_panel(root);
            spawn_help(root);
        });
}

/// The label is a sibling `Text`: the crate's switch is the track and thumb only.
fn spawn_row(root: &mut ChildSpawnerCommands, label: &'static str, on: bool, extra: impl Bundle) {
    root.spawn(Node {
        column_gap: px(16),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new(label),
            Node {
                width: px(320),
                ..default()
            },
        ));
        let mut switch = row.spawn((toggle_switch().build(), Tag(label), extra));
        if on {
            switch.insert(Checked);
        }
    });
}

/// Framed panel with its own `Theme` (unaffected by the global theme).
fn spawn_scoped_panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            column_gap: px(16),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(2)),
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
        Theme::default(),
        ScopedPanel,
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Scoped (own Theme)"),
            Node {
                width: px(296),
                ..default()
            },
        ));
        panel.spawn((toggle_switch().build(), Checked, Tag("Scoped")));
    });
}

fn spawn_help(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Text::new(
            "[Tab] move focus   [Enter / Space] ask for a change\n\
             [T] swap GLOBAL theme; the framed panel is unaffected\n\
             [S] swap the framed panel's own Theme only",
        ),
        Node {
            margin: UiRect::top(px(24)),
            ..default()
        },
    ));
    root.spawn((Text::new("Request: (nothing yet)"), StatusText));
}

// ---------------------------------------------------------------------------
// Behavior
// ---------------------------------------------------------------------------

/// The app's policy: apply the requested value, unless the switch is locked.
fn on_value_change(
    change: On<ValueChange<bool>>,
    mut commands: Commands,
    tags: Query<&Tag>,
    locked: Query<(), With<Locked>>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    let name = tags.get(change.source).map_or("?", |tag| tag.0);
    let refused = locked.contains(change.source);

    if !refused {
        if change.value {
            commands.entity(change.source).insert(Checked);
        } else {
            commands.entity(change.source).remove::<Checked>();
        }
    }
    for mut text in &mut status {
        text.0 = format!(
            "Request: {name} -> {}{}",
            if change.value { "on" } else { "off" },
            if refused { " (refused)" } else { "" }
        );
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

/// When `swapped`, returns a theme whose on color is orange, so that the change is easy to see.
fn theme_for(swapped: bool) -> Theme {
    let mut theme = Theme::default();
    if swapped {
        let base = theme.switch.track.checked;
        theme.switch.track.checked = StateColors {
            normal: Color::srgb(0.85, 0.45, 0.10),
            hovered: Color::srgb(0.95, 0.58, 0.20),
            pressed: Color::srgb(0.65, 0.33, 0.07),
            focused: Color::srgb(0.95, 0.58, 0.20),
            disabled: base.disabled,
        };
    }
    theme
}
