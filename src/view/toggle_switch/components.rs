use crate::view::KeyboardNavigable;
use bevy::{
    input_focus::tab_navigation::TabIndex, picking::hover::PickingInteraction, prelude::*,
    ui_widgets::Checkbox,
};

// Track: 52 x 30 with a 2px border and 2px padding leaves a 22px square for the thumb.
const TRACK_WIDTH: f32 = 52.0;
const TRACK_HEIGHT: f32 = 30.0;
const TRACK_BORDER: f32 = 2.0;
const TRACK_PADDING: f32 = 2.0;
const THUMB_SIZE: f32 = 22.0;

/// A themed toggle switch: a track with a sliding thumb.
///
/// Behavior comes from [`Checkbox`]: a click or Enter/Space on the focused switch emits
/// [`bevy::ui_widgets::ValueChange<bool>`] carrying the requested value, and the on/off state is the
/// presence of [`bevy::ui::Checked`]. The switch never changes its own state: the app applies the
/// change by inserting or removing `Checked` (or adds
/// [`bevy::ui_widgets::checkbox_self_update`] to let Bevy do exactly that). The initial state is
/// the same: spawn `(toggle_switch().build(), Checked)` for a switch that starts on.
///
/// Accessibility tools see a checkbox (Bevy's role for [`Checkbox`]); a switch role needs a direct
/// `accesskit` dependency, which this crate does not take yet.
///
/// `TabIndex(0)` makes it reachable with the Tab key when it is inside a `TabGroup`.
/// A focus ring is drawn through `Outline` while the focus indicator is visible.
#[derive(Component)]
#[require(
    Checkbox,
    PickingInteraction,
    TabIndex(0),
    KeyboardNavigable,
    Outline = hidden_outline(),
    Node = track_node()
)]
pub struct ToggleSwitch;

/// The sliding thumb inside a [`ToggleSwitch`].
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct SwitchThumb;

/// The thumb rests at the start edge when off and at the end edge when on.
/// Flex alignment instead of pixel offsets keeps it correct for any track size.
pub(crate) fn thumb_side(checked: bool) -> JustifyContent {
    if checked {
        JustifyContent::End
    } else {
        JustifyContent::Start
    }
}

/// Size, border, and corner radius are constant (chrome, not scale).
pub(crate) fn track_node() -> Node {
    Node {
        width: Val::Px(TRACK_WIDTH),
        height: Val::Px(TRACK_HEIGHT),
        padding: UiRect::all(Val::Px(TRACK_PADDING)),
        border: UiRect::all(Val::Px(TRACK_BORDER)),
        border_radius: BorderRadius::all(Val::Px(TRACK_HEIGHT / 2.0)),
        align_items: AlignItems::Center,
        justify_content: thumb_side(false),
        ..default()
    }
}

pub(crate) fn thumb_node() -> Node {
    Node {
        width: Val::Px(THUMB_SIZE),
        height: Val::Px(THUMB_SIZE),
        border_radius: BorderRadius::all(Val::Px(THUMB_SIZE / 2.0)),
        ..default()
    }
}

/// `Outline::default()` is a visible white line, so the ring starts hidden.
fn hidden_outline() -> Outline {
    Outline::new(Val::Px(0.0), Val::Px(0.0), Color::NONE)
}
