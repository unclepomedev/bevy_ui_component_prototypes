//! Sample for verifying the popup.
//!
//! Execution: cargo run --example popup_check
//!
//! A popup is shown by spawning it. What its buttons mean, and when it goes away, are this example's
//! choices: it despawns the popup on every choice and shows the choice on the screen.
//!
//! What to verify:
//!   1. [1] shows an alert (one button), [2] a confirm and [3] a destructive confirm. A dark scrim covers
//!      the screen, and the panel is in the middle. A choice closes the popup and shows it below.
//!   2. While a popup is open, clicking the "Click me" button behind it does nothing: the scrim keeps the
//!      pointer from reaching it. Tab goes only through the buttons of the popup and wraps around, and
//!      Enter or Space activates the one that has the focus.
//!   3. [4] shows a long message: it wraps at the width of the theme and the panel grows taller.
//!   4. [5] shows two popups: the second one is spawned later but is behind the first, because the first has
//!      a `GlobalZIndex` of its own and the second takes the one of the theme. Close the front one to reach
//!      the other.
//!   5. [T] swaps the global theme between the default one and a terminal one (square, outlined,
//!      monospace). A popup that is on screen takes the new look as it is.

use bevy::{input::common_conditions::input_just_pressed, prelude::*, ui_widgets::Activate};
use bevy_ui_component_prototypes::{
    ButtonVariant, ButtonVariantId, GlobalTheme, PopupChoice, PopupChosen, PopupShape, PopupStyle,
    Theme, UiComponentPrototypesPlugin, icon_button, popup,
};

/// The stacking order is the app's to decide; this one is in front of the popups that take the theme's.
const FRONT_Z: i32 = 1000;
const LONG: &str = "This message is long enough to wrap at the width of the theme.\nThis one is on a line of its own.";

const GREEN: Color = Color::srgb(0.3, 1.0, 0.4);

#[derive(Component)]
struct ChoiceText;

#[derive(Component)]
struct ClickText;

#[derive(Component)]
struct ClickMe;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UiComponentPrototypesPlugin))
        .insert_resource(GlobalTheme(Theme::default()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                show_popups,
                swap_theme.run_if(input_just_pressed(KeyCode::KeyT)),
            ),
        )
        .add_observer(on_chosen)
        .add_observer(on_activate)
        .run();
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
                "[1] alert   [2] confirm   [3] destructive   [4] long message\n\
                 [5] two popups, the first one in front   [T] swap global theme",
            ));
            root.spawn((Text::new("Choice: none"), ChoiceText));
            root.spawn((Text::new("Clicks: 0"), ClickText));
            root.spawn((icon_button().label("Click me").build(), ClickMe));
        });
}

// ---------------------------------------------------------------------------
// Behavior
// ---------------------------------------------------------------------------

fn show_popups(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Digit1) {
        commands.spawn(
            popup()
                .message("Your settings were saved.")
                .button("OK", PopupChoice::Confirm, ButtonVariantId::PRIMARY)
                .build(),
        );
    }
    if keys.just_pressed(KeyCode::Digit2) {
        commands.spawn(confirm("Save your changes?", "Save", "Cancel", false));
    }
    if keys.just_pressed(KeyCode::Digit3) {
        commands.spawn(confirm(
            "Delete this file for good?",
            "Delete",
            "Keep",
            true,
        ));
    }
    if keys.just_pressed(KeyCode::Digit4) {
        commands.spawn(confirm(LONG, "Yes", "No", false));
    }
    if keys.just_pressed(KeyCode::Digit5) {
        commands.spawn((
            confirm("In front, whatever was spawned later.", "Yes", "No", false),
            GlobalZIndex(FRONT_Z),
        ));
        commands.spawn(confirm(
            "Behind, with the layer of the theme.",
            "Yes",
            "No",
            true,
        ));
    }
}

/// The first button is the one that goes ahead.
fn confirm(message: &str, yes: &str, no: &str, destructive: bool) -> impl Bundle {
    let yes_variant = if destructive {
        ButtonVariantId::DANGER
    } else {
        ButtonVariantId::AFFIRMATIVE
    };
    popup()
        .message(message)
        .button(yes, PopupChoice::Confirm, yes_variant)
        .button(no, PopupChoice::Cancel, ButtonVariant::Default)
        .build()
}

fn on_chosen(
    chosen: On<PopupChosen>,
    mut text: Query<&mut Text, With<ChoiceText>>,
    mut commands: Commands,
) {
    for mut text in &mut text {
        text.0 = format!("Choice: {:?}", chosen.choice);
    }
    commands.entity(chosen.popup).despawn();
}

fn swap_theme(mut global: ResMut<GlobalTheme>, mut terminal: Local<bool>) {
    *terminal = !*terminal;
    *global = GlobalTheme(if *terminal {
        terminal_theme()
    } else {
        Theme::default()
    });
}

fn terminal_theme() -> Theme {
    Theme {
        popup: PopupStyle {
            scrim: Color::srgba(0.0, 0.05, 0.02, 0.8),
            bg: Color::srgb(0.0, 0.05, 0.02),
            border: GREEN,
            fg: GREEN,
            shape: PopupShape {
                border_width: 2.0,
                radius: 0.0,
            },
        },
        font: FontSource::monospace(),
        ..default()
    }
}

/// Counts the clicks of the button behind the popups only.
fn on_activate(
    activate: On<Activate>,
    click_me: Query<(), With<ClickMe>>,
    mut clicks: Local<u32>,
    mut text: Query<&mut Text, With<ClickText>>,
) {
    if !click_me.contains(activate.entity) {
        return;
    }
    *clicks += 1;
    for mut text in &mut text {
        text.0 = format!("Clicks: {}", *clicks);
    }
}
