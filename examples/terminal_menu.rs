//! A themed main menu: ghost buttons (no background, no border) on a dark screen.
//!
//! Execution: cargo run --example terminal_menu
//!
//! What to verify:
//!   1. The menu is text only. Hover brightens a label, press dims it, and CONTINUE is grayed out.
//!   2. Hovering an item moves keyboard focus to it, so the pointer and Tab / Shift+Tab share one selection, and
//!      there is a single ">" marker. The marker is drawn by this example, from the crate's `ShowsFocus`;
//! 　　　the theme only supplies colors, so the focus ring is turned off (`FocusRing::NONE`).
//!   3. Enter / Space on the focused item updates the "Activate:" line. While disabled, CONTINUE never activates,
//!      and Tab skips it (the crate takes disabled buttons out of the tab order).
//!   4. [Up] / [Down] move the focus too, and skip CONTINUE while it is disabled. Under the default theme
//!      the focus ring shows after an arrow key.
//!   5. [D] enables / disables CONTINUE.
//!   6. [T] swaps the global theme between this terminal theme and `Theme::default()`.
//!      Under the default theme the same buttons get solid backgrounds and the crate's focus ring.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::{
        AutoFocus, FocusCause, InputFocus, InputFocusVisible,
        directional_navigation::DirectionalNavigationPlugin,
        tab_navigation::{TabGroup, TabNavigationPlugin},
    },
    math::CompassOctant,
    picking::{Pickable, hover::PickingInteraction},
    prelude::*,
    ui::{
        InteractionDisabled,
        auto_directional_navigation::{AutoDirectionalNavigation, AutoDirectionalNavigator},
    },
    ui_widgets::Activate,
};
use bevy_ui_component_prototypes::{
    ButtonSize, ButtonStyle, ButtonStyles, GlobalTheme, ShowsFocus, StateColors, Theme,
    UiComponentPrototypesPlugin, icon_button,
};

// Sampled from the reference mock-up: a pale phosphor green on a near-black desk.
const SCREEN: Color = Color::srgb(0.03, 0.04, 0.045);
const PHOSPHOR_BRIGHT: Color = Color::srgb(0.86, 0.96, 0.85);
const PHOSPHOR_PRESSED: Color = Color::srgb(0.45, 0.52, 0.44);
const PHOSPHOR_OFF: Color = Color::srgb(0.28, 0.32, 0.28);

/// Label of the button, shown in the status line when it is activated.
#[derive(Component)]
struct Label(&'static str);

/// The CONTINUE item, whose disabled state is toggled with [D].
#[derive(Component)]
struct ContinueItem;

/// The ">" drawn next to the hovered or focused item. Belongs to this example, not to the crate.
#[derive(Component)]
struct Marker {
    button: Entity,
}

#[derive(Component)]
struct StatusText;

/// Whether the terminal theme (and therefore the marker) is active.
#[derive(Resource)]
struct TerminalLook(bool);

fn main() {
    App::new()
        // `DefaultPlugins` already includes `UiWidgetsPlugins` and the input stack;
        // Tab and directional (arrow key) navigation have to be added.
        .add_plugins((
            DefaultPlugins,
            TabNavigationPlugin,
            DirectionalNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .insert_resource(ClearColor(SCREEN))
        .insert_resource(GlobalTheme(terminal_theme()))
        .insert_resource(TerminalLook(true))
        .insert_resource(InputFocusVisible(true))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_continue.run_if(input_just_pressed(KeyCode::KeyD)),
                swap_theme.run_if(input_just_pressed(KeyCode::KeyT)),
                arrow_keys_move_focus,
                // Console-style menu: hover moves focus, and the focus indicator stays visible
                // (the crate's pointer focus would hide it on every click).
                (hover_moves_focus, keep_focus_visible)
                    .chain()
                    .run_if(terminal_look),
                update_markers,
            ),
        )
        .add_observer(on_activate)
        .run();
}

/// Ghost buttons: no background, no border, no ring. Only the text color carries the state.
fn terminal_theme() -> Theme {
    let fg = |normal: Color, lit: Color| StateColors {
        normal,
        hovered: lit,
        pressed: lit,
        disabled: PHOSPHOR_OFF,
        focused: lit,
    };
    Theme {
        buttons: ButtonStyles {
            neutral: ButtonStyle::ghost(fg(PHOSPHOR_PRESSED, PHOSPHOR_BRIGHT)),
            primary: ButtonStyle::ghost(fg(PHOSPHOR_BRIGHT, Color::WHITE)),
            ..default()
        },
        font: FontSource::serif(),
        ..default()
    }
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            // Buttons are only reachable with Tab when they sit under a `TabGroup`.
            TabGroup::default(),
        ))
        .with_children(|root| {
            spawn_item(root, "NEW INVESTIGATION", true, false);
            spawn_item(root, "CONTINUE", false, true);
            spawn_item(root, "OPTIONS", false, false);
            spawn_item(root, "EXIT", false, false);
            spawn_help(root);
        });
}

/// One menu row: the button, with the marker laid over its left side.
fn spawn_item(root: &mut ChildSpawnerCommands, label: &'static str, autofocus: bool, off: bool) {
    root.spawn(Node {
        width: Val::Px(480.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        let mut button = row.spawn((
            icon_button(label).size(ButtonSize::medium()).build(),
            Label(label),
            AutoDirectionalNavigation::default(),
        ));
        if autofocus {
            button.insert(AutoFocus);
        }
        if off {
            button.insert((InteractionDisabled, ContinueItem));
        }
        let button = button.id();

        row.spawn((
            Text::new(">"),
            TextFont {
                font: FontSource::serif(),
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(PHOSPHOR_BRIGHT),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(40.0),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
            Marker { button },
        ));
    });
}

fn spawn_help(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Text::new("[Tab] [Up] [Down] move   [Enter / Space] activate   [D] toggle CONTINUE   [T] swap theme"),
        TextColor(PHOSPHOR_OFF),
        Node {
            margin: UiRect::top(Val::Px(48.0)),
            ..default()
        },
    ));
    root.spawn((
        Text::new("Activate: (nothing yet)"),
        TextColor(PHOSPHOR_OFF),
        StatusText,
    ));
}

// ---------------------------------------------------------------------------
// Behavior
// ---------------------------------------------------------------------------

fn on_activate(
    trigger: On<Activate>,
    labels: Query<&Label>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    let name = labels.get(trigger.entity).map_or("?", |label| label.0);
    for mut text in &mut status {
        text.0 = format!("Activate: {name}");
    }
}

fn terminal_look(look: Res<TerminalLook>) -> bool {
    look.0
}

type HoveredMenuButtons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static PickingInteraction,
        Has<InteractionDisabled>,
    ),
    (Changed<PickingInteraction>, With<Label>),
>;

/// The hovered item takes focus, so there is one selection shared by the pointer and the keyboard.
fn hover_moves_focus(mut focus: ResMut<InputFocus>, hovered: HoveredMenuButtons) {
    for (entity, interaction, disabled) in &hovered {
        if *interaction != PickingInteraction::None && !disabled && focus.get() != Some(entity) {
            focus.set(entity, FocusCause::Navigated);
        }
    }
}

/// Up / Down move the focus to the nearest button in that direction and show the focus ring.
fn arrow_keys_move_focus(
    keys: Res<ButtonInput<KeyCode>>,
    mut navigator: AutoDirectionalNavigator,
    mut focus_visible: ResMut<InputFocusVisible>,
) {
    for (key, direction) in [
        (KeyCode::ArrowUp, CompassOctant::North),
        (KeyCode::ArrowDown, CompassOctant::South),
    ] {
        if keys.just_pressed(key) {
            // Directional navigation leaves `InputFocusVisible` alone, so the ring would stay hidden.
            focus_visible.0 = true;
            // No neighbor (first or last item) just leaves the focus where it is.
            let _ = navigator.navigate(direction);
        }
    }
}

fn keep_focus_visible(mut focus_visible: ResMut<InputFocusVisible>) {
    if !focus_visible.0 {
        focus_visible.0 = true;
    }
}

/// Shows the marker next to the item that holds the visible focus, under the terminal look only.
fn update_markers(
    look: Res<TerminalLook>,
    shown: Query<(), With<ShowsFocus>>,
    mut markers: Query<(&Marker, &mut Visibility)>,
) {
    for (marker, mut visibility) in &mut markers {
        let active = look.0 && shown.contains(marker.button);
        visibility.set_if_neq(if active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

fn toggle_continue(
    mut commands: Commands,
    items: Query<(Entity, Has<InteractionDisabled>), With<ContinueItem>>,
    mut navigator: AutoDirectionalNavigator,
) {
    for (entity, disabled) in &items {
        if disabled {
            commands.entity(entity).remove::<InteractionDisabled>();
        } else {
            // The arrows cannot leave a disabled button, so move off it while it still can be left.
            if navigator.input_focus() == Some(entity) {
                let _ = navigator
                    .navigate(CompassOctant::South)
                    .or_else(|_| navigator.navigate(CompassOctant::North));
            }
            commands.entity(entity).insert(InteractionDisabled);
        }
    }
}

fn swap_theme(mut global: ResMut<GlobalTheme>, mut look: ResMut<TerminalLook>) {
    look.0 = !look.0;
    *global = GlobalTheme(if look.0 {
        terminal_theme()
    } else {
        Theme::default()
    });
}
