//! Sample for verifying the stepper.
//!
//! Execution: cargo run --example stepper_check
//!
//! The stepper is controlled: it only reports `ValueChange<usize>`, and this example's observer decides
//! what to do with it (insert a new `StepperValue`, and write the number into the label). That policy
//! lives here, not in the crate.
//!
//! What to verify:
//!   1. "10 steps + marks": clicking a cell requests that position and moves the lit cells there; the
//!      cells up to the current one are lit. Marks sit under the cells given to `marks` for that row. Clicking the
//!      current cell requests nothing (the status line does not change).
//!   2. The arrows on the buttons point left and right, and sit in the middle of their buttons.
//!      The previous button is dimmed and does not react at the first position; the next button at the
//!      last one. With a single position both are dimmed.
//!   3. "Current only": only the current cell is lit, whatever the position.
//!   4. "5 steps, compact": no cells, only "< label >". The buttons step by one and dim at the ends.
//!   5. "Wrap": the buttons never dim; next at the last position goes to the first, and back.
//!   6. "Disabled" is gray, ignores the pointer and the keys, and Tab skips it.
//!   7. "Locked" asks for a change, but the observer refuses it: the status line shows the request, and
//!      the display stays where it is.
//!   8. "20 steps, 160px": the cells get narrow. The log shows one warning about the cell width, and the
//!      stepper keeps working.
//!   9. Tab / Shift+Tab move keyboard focus between the steppers (one stop each, never the buttons or
//!      the cells), and a focus ring appears around the focused one. Clicking hides the ring again.
//!  10. With a stepper focused, [Left] / [Right] step by one and [Home] / [End] go to the ends. The
//!      focus does not jump to another widget with the arrows, also at an end. [Up] / [Down] may move it.
//!  11. [T] swaps the global theme (the current cell turns orange). The framed panel, which has its own
//!      `Theme`, does not change.
//!  12. [S] swaps the framed panel's own `Theme`. Only the panel changes.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::ValueChange,
};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, UiComponentPrototypesPlugin,
    common::StateColors,
    stepper::{StepperBuilder, StepperFill, StepperSteps, StepperValue, stepper},
};

/// Name of the stepper, shown in the status line.
#[derive(Component)]
struct Tag {
    name: &'static str,
}

/// The app refuses every change request made by this stepper.
#[derive(Component)]
struct Locked;

/// The label that shows the position as a number.
#[derive(Component)]
struct ValueLabel;

/// Panel that carries its own `Theme`.
#[derive(Component)]
struct ScopedPanel;

#[derive(Component)]
struct StatusText;

fn main() {
    App::new()
        .add_plugins((
            // Tall enough for every row.
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (1280, 960).into(),
                    ..default()
                }),
                ..default()
            }),
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
        .add_observer(apply_value)
        .add_observer(show_request)
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
            spawn_row(
                root,
                "10 steps + marks",
                360,
                numbered(10, 3).marks([3, 7]),
                (),
            );
            spawn_row(
                root,
                "Current only",
                360,
                numbered(10, 3).fill(StepperFill::Current),
                (),
            );
            spawn_row(root, "5 steps, compact", 360, numbered(5, 2).compact(), ());
            spawn_row(root, "Wrap", 360, numbered(5, 0).wrap(true), ());
            spawn_row(root, "Disabled", 360, numbered(5, 2), InteractionDisabled);
            spawn_row(
                root,
                "Locked (the app refuses changes)",
                360,
                numbered(5, 1),
                Locked,
            );
            spawn_row(root, "20 steps, 160px", 160, numbered(20, 9), ());
            spawn_row(root, "1 step", 360, numbered(1, 0), ());
            spawn_scoped_panel(root);
            spawn_help(root);
        });
}

/// A stepper whose label shows the position. The label is the stepper's own text between the buttons.
fn numbered(steps: usize, value: usize) -> StepperBuilder {
    stepper()
        .steps(steps)
        .value(value)
        .label_bundle((ValueLabel, Text::new(caption(value, steps))))
}

fn caption(value: usize, steps: usize) -> String {
    format!("{} / {steps}", value + 1)
}

/// The name is a sibling `Text`; the stepper takes the full width of the box it is put in.
fn spawn_row(
    root: &mut ChildSpawnerCommands,
    name: &'static str,
    width: i32,
    builder: StepperBuilder,
    extra: impl Bundle,
) {
    root.spawn(Node {
        column_gap: px(16),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new(name),
            Node {
                width: px(260),
                ..default()
            },
        ));
        row.spawn(Node {
            width: px(width),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn((builder.build(), Tag { name }, extra));
        });
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
                width: px(236),
                ..default()
            },
        ));
        panel
            .spawn(Node {
                width: px(360),
                ..default()
            })
            .with_children(|slot| {
                slot.spawn((numbered(10, 4).build(), Tag { name: "Scoped" }));
            });
    });
}

fn spawn_help(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Text::new(
            "[Tab] move focus   [Left / Right] one step   [Home / End] ends\n\
             [T] swap GLOBAL theme; the framed panel is unaffected\n\
             [S] swap the framed panel's own Theme only",
        ),
        Node {
            margin: UiRect::top(px(16)),
            ..default()
        },
    ));
    root.spawn((Text::new("Request: (nothing yet)"), StatusText));
}

// ---------------------------------------------------------------------------
// Behavior
// ---------------------------------------------------------------------------

/// The app's policy: apply the requested value and show it in the label, unless the stepper is locked.
fn apply_value(
    change: On<ValueChange<usize>>,
    mut commands: Commands,
    locked: Query<(), With<Locked>>,
    steps: Query<&StepperSteps>,
    children: Query<&Children>,
    mut labels: Query<&mut Text, (With<ValueLabel>, Without<StatusText>)>,
) {
    if locked.contains(change.source) {
        return;
    }
    commands
        .entity(change.source)
        .insert(StepperValue::new(change.value));
    let Ok(steps) = steps.get(change.source) else {
        return;
    };
    for entity in children.iter_descendants(change.source) {
        if let Ok(mut label) = labels.get_mut(entity) {
            *label = Text::new(caption(change.value, steps.get()));
        }
    }
}

/// Shows the last request on the status line, whether or not it was applied.
fn show_request(
    change: On<ValueChange<usize>>,
    tags: Query<&Tag>,
    locked: Query<(), With<Locked>>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    let name = tags.get(change.source).map_or("?", |tag| tag.name);
    let suffix = if locked.contains(change.source) {
        " (locked)"
    } else {
        ""
    };
    for mut text in &mut status {
        *text = Text::new(format!("Request: {name} -> {}{suffix}", change.value));
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

/// When `swapped`, returns a theme whose current cell is orange, so that the change is easy to see.
fn theme_for(swapped: bool) -> Theme {
    let mut theme = Theme::default();
    if swapped {
        let base = theme.stepper.current;
        theme.stepper.current = StateColors {
            normal: Color::srgb(0.85, 0.45, 0.10),
            hovered: Color::srgb(0.95, 0.58, 0.20),
            pressed: Color::srgb(0.65, 0.33, 0.07),
            focused: Color::srgb(0.95, 0.58, 0.20),
            disabled: base.disabled,
        };
    }
    theme
}
