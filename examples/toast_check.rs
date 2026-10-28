//! Sample for verifying the toast.
//!
//! Execution: cargo run --example toast_check
//!
//! A toast is shown by spawning it under a host. Where a host sits, which way it stacks and where its
//! toasts slide in from are all this example's choices: there are three hosts at three places.
//!
//! What to verify:
//!   1. [1] [2] [3] show an info, a success and a danger toast at the bottom right. Each slides in from
//!      the right while it fades in, stays, and fades out. The newest one is at the bottom and the older
//!      ones are pushed up. (The older ones jump rather than glide: that is known for now.)
//!   2. [4] shows a long text with a line break: it wraps at the width of the theme and the toast grows
//!      taller.
//!   3. [5] shows six toasts at once at a host that keeps four: the oldest ones start fading out at once,
//!      the newest ones stay, and nothing is dropped.
//!   4. [6] shows a toast at the top left: it comes in from the left, the newest one is at the top, and
//!      the older ones are pushed down. [7] shows one at the top center, coming in from above.
//!   5. The button at the bottom right counts clicks, also while toasts are on top of it: toasts let the
//!      pointer through, and hovering a toast does nothing.
//!   6. [P] pauses the game time: the clock stops, and toasts go on fading and leaving.
//!   7. [8] shows a toast of a variant this example defines itself (amber), in both themes.
//!   8. [T] swaps the global theme between the default one and a terminal one (square, outlined,
//!      monospace). Toasts that are on screen take the new look as they are.

use bevy::{input::common_conditions::input_just_pressed, prelude::*, ui_widgets::Activate};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, ToastShape, ToastStack, ToastStyle, ToastStyles, ToastVariantId,
    UiComponentPrototypesPlugin, icon_button, toast, toast_host,
};
use std::collections::HashMap;

/// The stacking order is the app's to decide; the hosts are drawn in front of everything else here.
const TOAST_Z: i32 = 100;
/// How many toasts the bottom right host keeps.
const KEPT: usize = 4;
const BURST: usize = 6;
const EDGE: f32 = 24.0;

const WARNING: ToastVariantId = ToastVariantId::new("warning");
const LONG: &str = "This text is long enough to wrap at the width of the theme.\nThis one is on a line of its own.";

const GREEN: Color = Color::srgb(0.3, 1.0, 0.4);
const BRIGHT: Color = Color::srgb(0.85, 1.0, 0.85);
const RED: Color = Color::srgb(1.0, 0.4, 0.35);
const AMBER: Color = Color::srgb(1.0, 0.75, 0.25);

#[derive(Clone, Copy)]
enum Place {
    BottomRight,
    TopLeft,
    TopCenter,
}

#[derive(Resource)]
struct Hosts {
    bottom_right: Entity,
    top_left: Entity,
    top_center: Entity,
}

impl Hosts {
    fn get(&self, place: Place) -> Entity {
        match place {
            Place::BottomRight => self.bottom_right,
            Place::TopLeft => self.top_left,
            Place::TopCenter => self.top_center,
        }
    }
}

const SHOWS: [(KeyCode, Place, &str, ToastVariantId); 7] = [
    (
        KeyCode::Digit1,
        Place::BottomRight,
        "Saved.",
        ToastVariantId::INFO,
    ),
    (
        KeyCode::Digit2,
        Place::BottomRight,
        "Upload finished.",
        ToastVariantId::SUCCESS,
    ),
    (
        KeyCode::Digit3,
        Place::BottomRight,
        "Connection lost.",
        ToastVariantId::DANGER,
    ),
    (
        KeyCode::Digit4,
        Place::BottomRight,
        LONG,
        ToastVariantId::INFO,
    ),
    (
        KeyCode::Digit6,
        Place::TopLeft,
        "From the left, newest at the top.",
        ToastVariantId::INFO,
    ),
    (
        KeyCode::Digit7,
        Place::TopCenter,
        "From above, in the middle.",
        ToastVariantId::SUCCESS,
    ),
    (
        KeyCode::Digit8,
        Place::BottomRight,
        "A variant of this example's own.",
        WARNING,
    ),
];

#[derive(Component)]
struct ClockText;

#[derive(Component)]
struct ClickText;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UiComponentPrototypesPlugin))
        .insert_resource(GlobalTheme(theme_for(false)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                show_toasts,
                update_clock,
                toggle_pause.run_if(input_just_pressed(KeyCode::KeyP)),
                swap_theme.run_if(input_just_pressed(KeyCode::KeyT)),
            ),
        )
        .add_observer(on_activate)
        .run();
}

// ---------------------------------------------------------------------------
// Themes
// ---------------------------------------------------------------------------

/// The default theme, or a terminal one. Both define `WARNING`.
fn theme_for(terminal: bool) -> Theme {
    if !terminal {
        let mut theme = Theme::default();
        let mut warning = theme.toasts.base;
        warning.bg = AMBER;
        warning.fg = Color::srgb(0.1, 0.1, 0.1);
        theme.toasts.named.insert(WARNING, warning);
        return theme;
    }
    let style = |color: Color| ToastStyle {
        bg: Color::srgb(0.0, 0.05, 0.02),
        fg: color,
        border: color,
        shape: ToastShape {
            border_width: 2.0,
            radius: 0.0,
        },
    };
    let base = style(GREEN);
    Theme {
        toasts: ToastStyles {
            base,
            named: HashMap::from([
                (ToastVariantId::INFO, base),
                (ToastVariantId::SUCCESS, style(BRIGHT)),
                (ToastVariantId::DANGER, style(RED)),
                (WARNING, style(AMBER)),
            ]),
        },
        font: FontSource::monospace(),
        ..default()
    }
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12),
            ..default()
        })
        .with_children(|root| {
            root.spawn(Text::new(
                "[1] [2] [3] bottom right   [4] long text   [5] burst of six\n\
                 [6] top left   [7] top center   [8] variant of this example\n\
                 [P] pause the game time   [T] swap global theme",
            ));
            root.spawn((Text::new("Game time: 0.0s"), ClockText));
            root.spawn((Text::new("Clicks: 0"), ClickText));
        });

    // Under the toasts of the bottom right host: clicks have to get through them.
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: px(EDGE),
            bottom: px(EDGE),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn(icon_button().label("Click me, even under a toast").build());
        });

    let bottom_right = commands
        .spawn((
            toast_host().max(KEPT).slide(Val2::percent(100, 0)).build(),
            Node {
                position_type: PositionType::Absolute,
                right: px(EDGE),
                bottom: px(EDGE),
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            GlobalZIndex(TOAST_Z),
        ))
        .id();
    let top_left = commands
        .spawn((
            toast_host()
                .stack(ToastStack::NewestAtTop)
                .slide(Val2::percent(-100, 0))
                .build(),
            Node {
                position_type: PositionType::Absolute,
                left: px(EDGE),
                top: px(EDGE),
                align_items: AlignItems::FlexStart,
                ..default()
            },
            GlobalZIndex(TOAST_Z),
        ))
        .id();
    let top_center = commands
        .spawn((
            toast_host()
                .stack(ToastStack::NewestAtTop)
                .slide(Val2::percent(0, -100))
                .build(),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(EDGE),
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(TOAST_Z),
        ))
        .id();
    commands.insert_resource(Hosts {
        bottom_right,
        top_left,
        top_center,
    });
}

// ---------------------------------------------------------------------------
// Behavior
// ---------------------------------------------------------------------------

fn show_toasts(keys: Res<ButtonInput<KeyCode>>, hosts: Res<Hosts>, mut commands: Commands) {
    for (key, place, text, variant) in SHOWS {
        if keys.just_pressed(key) {
            show(&mut commands, hosts.get(place), text, variant);
        }
    }
    if keys.just_pressed(KeyCode::Digit5) {
        for n in 1..=BURST {
            let text = format!("Burst {n} of {BURST}");
            show(
                &mut commands,
                hosts.bottom_right,
                &text,
                ToastVariantId::INFO,
            );
        }
    }
}

fn show(commands: &mut Commands, host: Entity, text: &str, variant: ToastVariantId) {
    commands.spawn((toast().text(text).variant(variant).build(), ChildOf(host)));
}

fn toggle_pause(mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    } else {
        time.pause();
    }
}

fn update_clock(time: Res<Time<Virtual>>, mut clock: Query<&mut Text, With<ClockText>>) {
    let state = if time.is_paused() { " (paused)" } else { "" };
    for mut text in &mut clock {
        text.0 = format!("Game time: {:.1}s{state}", time.elapsed_secs());
    }
}

fn swap_theme(mut global: ResMut<GlobalTheme>, mut terminal: Local<bool>) {
    *terminal = !*terminal;
    *global = GlobalTheme(theme_for(*terminal));
}

fn on_activate(
    _activate: On<Activate>,
    mut clicks: Local<u32>,
    mut text: Query<&mut Text, With<ClickText>>,
) {
    *clicks += 1;
    for mut text in &mut text {
        text.0 = format!("Clicks: {}", *clicks);
    }
}
