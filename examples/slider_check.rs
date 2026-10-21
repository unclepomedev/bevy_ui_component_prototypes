//! Sample for verifying the slider.
//!
//! Execution: cargo run --example slider_check
//!
//! The slider is controlled: it only reports `ValueChange<f32>`, and this example's observer decides
//! what to do with it (insert a new `SliderValue`, possibly rounded). That policy lives here, not in
//! the crate.
//!
//! What to verify:
//!   1. "Volume" (0..=1, continuous): dragging the thumb moves it with the pointer, without lag or
//!      drift, and the filled part of the rail always ends under the thumb. Clicking the rail moves
//!      the thumb there, and a drag can continue from it. Pressing the thumb itself does not make it
//!      jump.
//!   2. At both ends the thumb stays inside the rail (it does not overhang).
//!   3. "Brightness" (0..=100, step 10): the app rounds the requested value to the step, so the thumb
//!      moves in notches. The arrow keys move by one step; Home / End go to the ends.
//!   4. The two "Narrow" sliders share a width and sit in a 160px / 480px parent: the slider follows the
//!      width of its parent.
//!   5. "Locked" asks for a change, but the observer refuses it: the status line shows the request and
//!      the value it keeps (it stays at 0.20). Thumb dragging and keyboard input request values
//!      around the current value; clicking the rail requests the clicked position.
//!   6. Hover and drag change the color of the filled part. Tab / Shift+Tab move keyboard focus between
//!      the sliders, and a focus ring appears around the focused one. Clicking hides the ring again.
//!   7. "Disabled" is gray, ignores the pointer and the keys, and Tab skips it.
//!   8. [T] swaps the global theme (the fill turns orange). The framed panel, which has its own
//!      `Theme`, does not change.
//!   9. [S] swaps the framed panel's own `Theme`. Only the panel changes.

use bevy::{
    ecs::query::QueryData,
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{SliderRange, SliderStep, SliderValue, ValueChange},
};
use bevy_ui_component_prototypes::{
    GlobalTheme, StateColors, Theme, UiComponentPrototypesPlugin, slider,
};

/// Name of the slider, shown in the status line.
#[derive(Component)]
struct Tag(&'static str);

/// The app refuses every change request made by this slider.
#[derive(Component)]
struct Locked;

/// The app rounds the requested value to the slider's `SliderStep`.
#[derive(Component)]
struct Discrete;

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
                row_gap: px(20),
                ..default()
            },
            TabGroup::default(),
        ))
        .with_children(|root| {
            spawn_row(root, "Volume", 320, slider().value(0.4), ());
            spawn_row(
                root,
                "Brightness (step 10)",
                320,
                slider().range(0.0, 100.0).step(10.0).value(50.0),
                Discrete,
            );
            spawn_row(root, "Narrow 160px", 160, slider().value(0.7), ());
            spawn_row(root, "Narrow 480px", 480, slider().value(0.7), ());
            spawn_row(
                root,
                "Locked (the app refuses changes)",
                320,
                slider().value(0.2),
                Locked,
            );
            spawn_row(
                root,
                "Disabled",
                320,
                slider().value(0.6),
                InteractionDisabled,
            );
            spawn_scoped_panel(root);
            spawn_help(root);
        });
}

/// The label is a sibling `Text`: the crate's slider is the rail, fill and thumb only. Screen readers
/// do not pair the two, so the slider gets an `AccessibleLabel` of its own. The slider takes the full
/// width of the box it is put in.
fn spawn_row(
    root: &mut ChildSpawnerCommands,
    label: &'static str,
    width: i32,
    builder: bevy_ui_component_prototypes::SliderBuilder,
    extra: impl Bundle,
) {
    root.spawn(Node {
        column_gap: px(16),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new(label),
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
            slot.spawn((
                builder.build(),
                AccessibleLabel::new(label),
                Tag(label),
                extra,
            ));
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
                width: px(320),
                ..default()
            })
            .with_children(|slot| {
                slot.spawn((
                    slider().value(0.5).build(),
                    AccessibleLabel::new("Scoped (own Theme)"),
                    Tag("Scoped"),
                ));
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
            margin: UiRect::top(px(24)),
            ..default()
        },
    ));
    root.spawn((Text::new("Request: (nothing yet)"), StatusText));
}

// ---------------------------------------------------------------------------
// Behavior
// ---------------------------------------------------------------------------

/// The app's policy: apply the requested value (rounded to the step if the slider is `Discrete`),
/// unless the slider is locked.
fn on_value_change(
    change: On<ValueChange<f32>>,
    mut commands: Commands,
    tags: Query<&Tag>,
    sliders: Query<SliderPolicy>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    let name = tags.get(change.source).map_or("?", |tag| tag.0);
    let Ok(slider) = sliders.get(change.source) else {
        return;
    };

    let value = if slider.discrete {
        let notches = ((change.value - slider.range.start()) / slider.step.0).round();
        slider
            .range
            .clamp(slider.range.start() + notches * slider.step.0)
    } else {
        change.value
    };
    if !slider.locked {
        commands.entity(change.source).insert(SliderValue(value));
    }
    for mut text in &mut status {
        text.0 = if slider.locked {
            format!(
                "Request: {name} -> {value:.2} (locked, stays {:.2})",
                slider.current.0
            )
        } else {
            format!("Request: {name} -> {value:.2}")
        };
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

/// When `swapped`, returns a theme whose fill is orange, so that the change is easy to see.
fn theme_for(swapped: bool) -> Theme {
    let mut theme = Theme::default();
    if swapped {
        let base = theme.slider.fill;
        theme.slider.fill = StateColors {
            normal: Color::srgb(0.85, 0.45, 0.10),
            hovered: Color::srgb(0.95, 0.58, 0.20),
            pressed: Color::srgb(0.65, 0.33, 0.07),
            focused: Color::srgb(0.95, 0.58, 0.20),
            disabled: base.disabled,
        };
    }
    theme
}

#[derive(QueryData)]
struct SliderPolicy {
    range: &'static SliderRange,
    step: &'static SliderStep,
    current: &'static SliderValue,
    discrete: Has<Discrete>,
    locked: Has<Locked>,
}
