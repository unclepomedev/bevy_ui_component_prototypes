//! A terminal-style settings screen made with the crate's own widgets.
//!
//! Execution: cargo run --example terminal_settings
//!
//! What it shows:
//!   - A green-on-black theme: ghost buttons (no background, no border, no ring) made only from `Theme`
//!     colors and the system monospace font. The selected language is a `Primary` button and the others
//!     are `Neutral`, so the app changes the variant to move the selection.
//!   - The crate's slider, shaped by `SliderShape` into a thin square bar without a thumb. The app rounds the
//!     requested value to a notch of 10 and writes the `NN%` text itself.
//!   - `[ON]` is a ghost button whose label the app rewrites.
//!   - Focus is shown by a `>` marker, not by a ring. Up / Down move the focus (the app's key policy);
//!     Left / Right change the focused value.
//!
//! What to verify:
//!   1. The first Up / Down press focuses a row; after that Up / Down walk through VOLUME, FULLSCREEN and
//!      the languages in order, and do not wrap.
//!   2. On VOLUME, Left / Right move the bar by 10% and the `NN%` text follows. A click or a drag on the
//!      bar moves it in the same notches.
//!   3. On FULLSCREEN, Enter / Space / Left / Right switch `[ON]` and `[OFF]`.
//!   4. On a language, Enter / Space makes it the bright one; the previous one turns dim.

use bevy::{
    ecs::system::SystemParam,
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::{
        FocusCause, FocusedInput, InputFocusVisible,
        directional_navigation::DirectionalNavigationPlugin,
    },
    math::CompassOctant,
    prelude::*,
    ui::auto_directional_navigation::{AutoDirectionalNavigation, AutoDirectionalNavigator},
    ui_widgets::{Activate, SliderValue, ValueChange},
};
use bevy_ui_component_prototypes::{
    ButtonSize, ButtonStyle, ButtonStyles, ButtonVariant, FocusRing, GlobalTheme, ShowsFocus,
    SliderShape, SliderStyle, StateColors, Theme, UiComponentPrototypesPlugin, icon_button, slider,
};

const VOLUME_STEP: f32 = 10.0;
const BRIGHT: Color = Color::srgb(0.30, 1.0, 0.40);
const RAIL: Color = Color::srgb(0.05, 0.22, 0.09);
const DIM: Color = Color::srgb(0.12, 0.55, 0.22);
const LANGUAGES: [&str; 5] = ["ENGLISH", "ESPANOL", "FRANCAIS", "DEUTSCH", "日本語"];

/// The widget that gets the focus on the first arrow key press.
#[derive(Component)]
struct FirstFocus;

/// Shows `>` while `.0` carries `ShowsFocus`.
#[derive(Component)]
struct FocusMarker(Entity);

#[derive(Component)]
struct VolumeText;

#[derive(Component)]
struct FullscreenButton;

#[derive(Component)]
struct Language(usize);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.03, 0.01)))
        .insert_resource(GlobalTheme(terminal_theme()))
        .add_plugins((
            DefaultPlugins,
            DirectionalNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_markers)
        .add_observer(move_focus_with_up_and_down)
        .add_observer(change_with_left_and_right)
        .add_observer(on_activate)
        .add_observer(on_volume_change)
        .run();
}

/// Ghost buttons: only the text color tells the state.
fn terminal_theme() -> Theme {
    let ghost = |normal: Color, lit: Color| ButtonStyle {
        bg: StateColors::uniform(Color::NONE),
        fg: StateColors {
            normal,
            hovered: lit,
            pressed: lit,
            disabled: DIM,
            focused: lit,
        },
        border: StateColors::uniform(Color::NONE),
        focus_ring: FocusRing::NONE,
        surface: None,
    };
    Theme {
        buttons: ButtonStyles {
            neutral: ghost(DIM, BRIGHT),
            primary: ghost(BRIGHT, Color::WHITE),
            ..default()
        },
        slider: SliderStyle {
            // A thin square bar without a thumb; only the filled part is bright.
            shape: SliderShape {
                rail_height: 14.0,
                thumb_size: 0.0,
                rounded: false,
            },
            rail: StateColors::uniform(RAIL),
            fill: StateColors::uniform(BRIGHT),
            focus_ring: FocusRing::NONE,
            ..default()
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

    let root = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(20),
            ..default()
        })
        .id();

    commands.spawn((text("--- GAME SETTINGS ---", 28.0, BRIGHT), ChildOf(root)));

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                padding: UiRect::axes(px(32), px(24)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(DIM),
            ChildOf(root),
        ))
        .id();

    spawn_volume_row(&mut commands, panel);
    spawn_fullscreen_row(&mut commands, panel);
    for (index, name) in LANGUAGES.into_iter().enumerate() {
        spawn_language_row(&mut commands, panel, index, name);
    }

    commands.spawn((
        text(
            "[UP/DOWN] NAVIGATE   [LEFT/RIGHT] CHANGE   [ENTER] CONFIRM",
            16.0,
            DIM,
        ),
        ChildOf(root),
    ));
}

fn text(content: &str, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextColor(color),
        TextFont {
            font: FontSource::monospace(),
            font_size: FontSize::Px(size),
            ..default()
        },
    )
}

/// One row: a `>` marker for the control, the row's name, and the control. The control entity is
/// reserved first so the marker can point at it and still be the first child.
fn spawn_row(commands: &mut Commands, panel: Entity, name: &str) -> (Entity, Entity) {
    let row = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            },
            ChildOf(panel),
        ))
        .id();
    let control = commands.spawn_empty().id();
    commands.spawn((
        text(" ", 20.0, BRIGHT),
        Node {
            width: px(14),
            ..default()
        },
        FocusMarker(control),
        ChildOf(row),
    ));
    commands.spawn((
        text(name, 20.0, DIM),
        Node {
            width: px(130),
            ..default()
        },
        ChildOf(row),
    ));
    (row, control)
}

fn spawn_volume_row(commands: &mut Commands, panel: Entity) {
    let (row, control) = spawn_row(commands, panel, "VOLUME");
    // The slider takes the width of the box it is put in.
    let slot = commands
        .spawn((
            Node {
                width: px(220),
                ..default()
            },
            ChildOf(row),
        ))
        .id();
    commands.entity(control).insert((
        slider()
            .range(0.0, 100.0)
            .step(VOLUME_STEP)
            .value(80.0)
            .build(),
        AccessibleLabel::new("Volume"),
        FirstFocus,
        AutoDirectionalNavigation::default(),
        ChildOf(slot),
    ));
    commands.spawn((text("80%", 20.0, BRIGHT), VolumeText, ChildOf(row)));
}

fn spawn_fullscreen_row(commands: &mut Commands, panel: Entity) {
    let (row, control) = spawn_row(commands, panel, "FULLSCREEN");
    commands.entity(control).insert((
        icon_button("[ON]")
            .size(row_size())
            .variant(ButtonVariant::Primary)
            .build(),
        FullscreenButton,
        AutoDirectionalNavigation::default(),
        ChildOf(row),
    ));
}

fn spawn_language_row(commands: &mut Commands, panel: Entity, index: usize, name: &str) {
    let label = if index == 0 { "LANGUAGE" } else { "" };
    let (row, control) = spawn_row(commands, panel, label);
    let variant = if index == 0 {
        ButtonVariant::Primary
    } else {
        ButtonVariant::Neutral
    };
    commands.entity(control).insert((
        icon_button(name).size(row_size()).variant(variant).build(),
        Language(index),
        AutoDirectionalNavigation::default(),
        ChildOf(row),
    ));
}

fn row_size() -> ButtonSize {
    ButtonSize {
        height: px(28),
        padding: UiRect::ZERO,
        ..ButtonSize::small()
    }
}

// ---------------------------------------------------------------------------
// Key policy
// ---------------------------------------------------------------------------

/// Up / Down move the focus. Left / Right are left to the focused widget (the slider's own keys).
fn move_focus_with_up_and_down(
    mut key: On<FocusedInput<KeyboardInput>>,
    mut navigation: Navigation,
) {
    if key.input.state != ButtonState::Pressed {
        return;
    }
    let direction = match key.input.key_code {
        KeyCode::ArrowUp => CompassOctant::North,
        KeyCode::ArrowDown => CompassOctant::South,
        _ => return,
    };
    key.propagate(false);
    // A directional navigation does not show the focus indicator, and a click may have hidden it.
    navigation.visible.0 = true;

    if navigation.navigable.contains(key.focused_entity) {
        // No neighbor is normal at the first and last row.
        let _ = navigation.navigator.navigate(direction);
    } else if let Ok(first) = navigation.first.single() {
        navigation
            .navigator
            .manual_directional_navigation
            .focus
            .set(first, FocusCause::Navigated);
    }
}

#[derive(SystemParam)]
struct Navigation<'w, 's> {
    navigator: AutoDirectionalNavigator<'w, 's>,
    navigable: Query<'w, 's, (), With<AutoDirectionalNavigation>>,
    first: Query<'w, 's, Entity, With<FirstFocus>>,
    visible: ResMut<'w, InputFocusVisible>,
}

/// Left / Right on FULLSCREEN do what Enter does.
fn change_with_left_and_right(
    key: On<FocusedInput<KeyboardInput>>,
    fullscreen: Query<(), With<FullscreenButton>>,
    mut commands: Commands,
) {
    let arrow = matches!(key.input.key_code, KeyCode::ArrowLeft | KeyCode::ArrowRight);
    if arrow && key.input.state == ButtonState::Pressed && fullscreen.contains(key.focused_entity) {
        commands.trigger(Activate {
            entity: key.focused_entity,
        });
    }
}

// ---------------------------------------------------------------------------
// Reactions
// ---------------------------------------------------------------------------

fn on_activate(
    activate: On<Activate>,
    fullscreen: Query<&Children, With<FullscreenButton>>,
    languages: Query<(Entity, &Language)>,
    mut labels: Query<&mut Text>,
    mut commands: Commands,
    mut on: Local<Option<bool>>,
) {
    if let Ok(children) = fullscreen.get(activate.entity) {
        let now = !on.unwrap_or(true);
        *on = Some(now);
        for child in children {
            if let Ok(mut label) = labels.get_mut(*child) {
                label.0 = if now { "[ON]" } else { "[OFF]" }.to_string();
            }
        }
        let variant = if now {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Neutral
        };
        commands.entity(activate.entity).insert(variant);
    } else if let Ok((_, chosen)) = languages.get(activate.entity) {
        for (entity, language) in &languages {
            let variant = if language.0 == chosen.0 {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Neutral
            };
            commands.entity(entity).insert(variant);
        }
    }
}

/// The app's policy: round the requested value to a notch, apply it, and show it.
fn on_volume_change(
    change: On<ValueChange<f32>>,
    mut commands: Commands,
    mut value_text: Query<&mut Text, With<VolumeText>>,
) {
    let value = (change.value / VOLUME_STEP).round() * VOLUME_STEP;
    commands.entity(change.source).insert(SliderValue(value));
    for mut text in &mut value_text {
        text.0 = format!("{value:.0}%");
    }
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

/// `>` in front of the row that holds the visible focus.
fn draw_markers(shown: Query<(), With<ShowsFocus>>, mut markers: Query<(&FocusMarker, &mut Text)>) {
    for (marker, mut text) in &mut markers {
        let wanted = if shown.contains(marker.0) { ">" } else { " " };
        if text.0 != wanted {
            text.0 = wanted.to_string();
        }
    }
}
