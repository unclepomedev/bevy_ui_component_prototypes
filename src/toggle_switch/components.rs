use crate::common::{focus::hidden_outline, tab_order::KeyboardNavigable};
use bevy::{
    input_focus::tab_navigation::TabIndex, picking::hover::PickingInteraction, prelude::*,
    ui_widgets::Checkbox,
};

const TRACK_WIDTH: f32 = 52.0;
const TRACK_HEIGHT: f32 = 30.0;
const TRACK_BORDER: f32 = 2.0;
const TRACK_PADDING: f32 = 2.0;
const THUMB_SIZE: f32 = TRACK_HEIGHT - 2.0 * (TRACK_BORDER + TRACK_PADDING);

/// A themed toggle switch: a track with a sliding thumb.
///
/// Behavior comes from [`Checkbox`]: a click or Enter/Space on the focused switch emits [`bevy::ui_widgets::ValueChange<bool>`]
/// carrying the requested value, and the on/off state is the presence of [`bevy::ui::Checked`].
/// The app applies the change by inserting or removing `Checked`.
/// The initial state is the same: spawn `(toggle_switch().build(), Checked)` for a switch that starts on.
///
/// See the [Conventions](crate#conventions) for the controlled value, the Tab key and the focus ring.
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
