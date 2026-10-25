//! Sample for verifying that the widgets can be driven with the arrow keys alone (no Tab).
//!
//! Execution: cargo run --example arrow_navigation
//!
//! Bevy has the directional navigation (`AutoDirectionalNavigator`) but no default key bindings for
//! it, and a directional navigation does not make the focus ring visible either. Both are this
//! example's policy, not the crate's.
//!
//! The widgets are in one row on purpose: moving to the slider and out of it both need Left / Right,
//! which the slider also uses for its value. The app settles it with two modes on the slider:
//! while it is only focused, the arrows move the focus, and Enter switches it to editing, where
//! Left / Right / Home / End change the value; Enter again goes back.
//!
//! What to verify:
//!   1. The first arrow key press only puts the focus on "Apply" and shows the ring. Nothing is
//!      focused before that.
//!   2. Right moves Apply -> Toggle -> Slider; the disabled toggle between them is skipped.
//!   3. On the focused slider, Left goes back to the toggle and the value does not change.
//!   4. Enter on the slider starts editing: the ring and the thumb turn amber and the status line says
//!      so. Left / Right change the value by one step, Home / End jump to the ends, and the focus
//!      stays. Enter ends editing and the colors return.
//!   5. Clicking elsewhere while editing ends it. Dragging the slider with the mouse works in both
//!      modes.
//!   6. Enter / Space on a button shows "Activated: ..." in the status line, and on the toggle flips
//!      it (the app applies the change).
//!   7. Clicking with the mouse hides the ring; the next arrow key press shows it again.
//!
//! Where things are:
//!   - Layout: `setup` and the `spawn_*` functions.
//!   - Key policy, one observer each: `toggle_editing` (Enter), `hold_slider_keys` (the slider's own
//!     keys while it is not edited), `navigate_with_arrows` (the arrows).
//!   - Reactions to the widgets: `on_activate`, `on_toggle_change`, `on_slider_change`.

use bevy::{
    ecs::system::SystemParam,
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::{
        FocusCause, FocusedInput, InputFocus, InputFocusVisible,
        directional_navigation::DirectionalNavigationPlugin,
    },
    math::CompassOctant,
    prelude::*,
    ui::{
        Checked, InteractionDisabled,
        auto_directional_navigation::{AutoDirectionalNavigation, AutoDirectionalNavigator},
    },
    ui_widgets::{Activate, Slider, SliderValue, ValueChange},
};
use bevy_ui_component_prototypes::{
    SliderEditing, UiComponentPrototypesPlugin, icon_button, slider, toggle_switch,
};

/// The widget that gets the focus on the first arrow key press.
#[derive(Component)]
struct FirstFocus;

/// Name of the widget, shown in the status line.
#[derive(Component)]
struct Tag(&'static str);

#[derive(Component)]
struct StatusText;

/// Set while a key press must not change a slider's value (it is for the focus). Cleared every frame.
#[derive(Resource, Default)]
struct SliderKeysHeld(bool);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DirectionalNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .init_resource::<SliderKeysHeld>()
        .add_systems(Startup, setup)
        .add_systems(Update, end_editing_when_unfocused)
        .add_systems(Last, |mut held: ResMut<SliderKeysHeld>| held.0 = false)
        .add_observer(toggle_editing)
        .add_observer(hold_slider_keys)
        .add_observer(navigate_with_arrows)
        .add_observer(on_activate)
        .add_observer(on_toggle_change)
        .add_observer(on_slider_change)
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
            row_gap: px(16),
            ..default()
        })
        .with_children(|root| {
            spawn_widget_row(root);
            spawn_help(root);
        });
}

fn spawn_widget_row(root: &mut ChildSpawnerCommands) {
    root.spawn(Node {
        column_gap: px(24),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn(apply_button());
        row.spawn(toggle());
        row.spawn(disabled_toggle());
        // The slider fills the width with the box it is put in.
        row.spawn(Node {
            width: px(320),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn(range_slider());
        });
    });
}

fn apply_button() -> impl Bundle {
    (
        icon_button().label("Apply").build(),
        AutoDirectionalNavigation::default(),
        FirstFocus,
        Tag("Apply"),
    )
}

fn toggle() -> impl Bundle {
    (
        toggle_switch().build(),
        AutoDirectionalNavigation::default(),
        AccessibleLabel::new("Toggle"),
        Tag("Toggle"),
    )
}

/// Disabled from the start: the arrows skip it.
fn disabled_toggle() -> impl Bundle {
    (
        toggle_switch().build(),
        AutoDirectionalNavigation::default(),
        AccessibleLabel::new("Disabled toggle"),
        InteractionDisabled,
        Checked,
    )
}

fn range_slider() -> impl Bundle {
    (
        slider().range(0.0, 100.0).step(10.0).value(50.0).build(),
        AutoDirectionalNavigation::default(),
        AccessibleLabel::new("Slider"),
        Tag("Slider"),
    )
}

fn spawn_help(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Text::new(
            "[Arrows] move   [Enter / Space] activate; Enter on the slider starts / ends editing",
        ),
        Node {
            margin: UiRect::top(px(24)),
            ..default()
        },
    ));
    root.spawn((Text::new("(nothing yet)"), StatusText));
}

// ---------------------------------------------------------------------------
// Key policy
// ---------------------------------------------------------------------------

// Keys are dispatched to the focused entity and bubble up to the window. A global observer runs at
// every step with the entity of that step as `focused_entity`, so each observer below acts only
// where it matters and stops the event after handling it.

/// Enter on the slider switches between navigating and editing.
/// Bevy itself activates buttons and toggles, so they need nothing here.
fn toggle_editing(
    mut key: On<FocusedInput<KeyboardInput>>,
    mut commands: Commands,
    sliders: Query<Has<SliderEditing>, With<Slider>>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    if key.input.state != ButtonState::Pressed || key.input.key_code != KeyCode::Enter {
        return;
    }
    let Ok(editing) = sliders.get(key.focused_entity) else {
        return;
    };
    key.propagate(false);
    if key.input.repeat {
        return;
    }
    if editing {
        commands
            .entity(key.focused_entity)
            .remove::<SliderEditing>();
        set_status(&mut status, "Slider: navigating".into());
    } else {
        commands.entity(key.focused_entity).insert(SliderEditing);
        set_status(
            &mut status,
            "Slider: editing (Left / Right / Home / End, Enter to finish)".into(),
        );
    }
}

/// The slider reacts to Left / Right / Home / End on its own and asks for a new value. While it is
/// not being edited, those keys are for the focus, so the requests of this frame are refused in `on_slider_change`.
fn hold_slider_keys(
    key: On<FocusedInput<KeyboardInput>>,
    sliders: Query<Has<SliderEditing>, With<Slider>>,
    mut held: ResMut<SliderKeysHeld>,
) {
    let slider_key = matches!(
        key.input.key_code,
        KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::Home | KeyCode::End
    );
    if slider_key && sliders.get(key.focused_entity) == Ok(false) {
        held.0 = true;
    }
}

/// Arrow keys move the focus, except while a slider is being edited.
fn navigate_with_arrows(
    mut key: On<FocusedInput<KeyboardInput>>,
    mut navigation: Navigation,
    editing: Query<(), With<SliderEditing>>,
) {
    if key.input.state != ButtonState::Pressed {
        return;
    }
    let Some(direction) = arrow_direction(key.input.key_code) else {
        return;
    };
    key.propagate(false);
    // A directional navigation does not show the ring, and a click may have hidden it.
    navigation.show_ring();

    let focused = key.focused_entity;
    if !navigation.can_navigate_from(focused) {
        // Nothing of ours has the focus yet (it is on the window): the first key press takes it.
        navigation.focus_first();
    } else if !editing.contains(focused) {
        navigation.go(direction);
    }
}

fn arrow_direction(key_code: KeyCode) -> Option<CompassOctant> {
    match key_code {
        KeyCode::ArrowUp => Some(CompassOctant::North),
        KeyCode::ArrowDown => Some(CompassOctant::South),
        KeyCode::ArrowLeft => Some(CompassOctant::West),
        KeyCode::ArrowRight => Some(CompassOctant::East),
        _ => None,
    }
}

/// What the arrow keys need to move the focus.
#[derive(SystemParam)]
struct Navigation<'w, 's> {
    navigator: AutoDirectionalNavigator<'w, 's>,
    navigable: Query<'w, 's, (), With<AutoDirectionalNavigation>>,
    first: Query<'w, 's, Entity, With<FirstFocus>>,
    visible: ResMut<'w, InputFocusVisible>,
}

impl Navigation<'_, '_> {
    fn show_ring(&mut self) {
        self.visible.0 = true;
    }

    fn can_navigate_from(&self, entity: Entity) -> bool {
        self.navigable.contains(entity)
    }

    fn focus_first(&mut self) {
        if let Ok(first) = self.first.single() {
            self.navigator
                .manual_directional_navigation
                .focus
                .set(first, FocusCause::Navigated);
        }
    }

    fn go(&mut self, direction: CompassOctant) {
        // No neighbor in that direction is normal at the edges of the layout.
        let _ = self.navigator.navigate(direction);
    }
}

/// Editing ends when the focus leaves the slider (a click elsewhere, for instance).
fn end_editing_when_unfocused(
    mut commands: Commands,
    focus: Res<InputFocus>,
    editing: Query<Entity, With<SliderEditing>>,
) {
    for slider in &editing {
        if focus.get() != Some(slider) {
            commands.entity(slider).remove::<SliderEditing>();
        }
    }
}

// ---------------------------------------------------------------------------
// Reactions to the widgets
// ---------------------------------------------------------------------------

fn on_activate(
    activate: On<Activate>,
    tags: Query<&Tag>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    let name = tags.get(activate.entity).map_or("?", |tag| tag.0);
    set_status(&mut status, format!("Activated: {name}"));
}

fn on_toggle_change(
    change: On<ValueChange<bool>>,
    mut commands: Commands,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    if change.value {
        commands.entity(change.source).insert(Checked);
    } else {
        commands.entity(change.source).remove::<Checked>();
    }
    set_status(
        &mut status,
        format!("Toggle -> {}", if change.value { "on" } else { "off" }),
    );
}

fn on_slider_change(
    change: On<ValueChange<f32>>,
    mut commands: Commands,
    held: Res<SliderKeysHeld>,
    mut status: Query<&mut Text, With<StatusText>>,
) {
    if held.0 {
        return;
    }
    commands
        .entity(change.source)
        .insert(SliderValue(change.value));
    set_status(&mut status, format!("Slider -> {:.0}", change.value));
}

fn set_status(status: &mut Query<&mut Text, With<StatusText>>, text: String) {
    for mut status in status {
        status.0 = text.clone();
    }
}
