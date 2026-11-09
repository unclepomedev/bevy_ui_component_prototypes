use super::style::{PopupMetrics, PopupShape};
use bevy::{input_focus::tab_navigation::TabGroup, picking::Pickable, prelude::*};

/// The `GlobalZIndex` of a popup unless the spawner sets one.
const POPUP_LAYER: i32 = 300;

/// Root of a modal popup: a scrim that covers its parent, centers the panel, and keeps the pointer
/// from reaching what is behind it. Spawn it without a parent to cover the screen. Despawn it to dismiss.
///
/// Tab stays inside it, also after a press on it. The input focus is kept in the front popup that `popup()` builds.
/// Giving the focus back after the last one is despawned is up to the app.
///
/// Its `GlobalZIndex` is the one it is spawned with, or a default above the windows of a default
/// [`WindowDefaultsPlugin`](crate::window::WindowDefaultsPlugin) without one.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(
    Node = popup_root_node(),
    GlobalZIndex = GlobalZIndex(POPUP_LAYER),
    BackgroundColor,
    Pickable = Pickable::default(),
    TabGroup = TabGroup::modal()
)]
pub struct PopupRoot;

/// The card in the middle of the scrim.
#[derive(Component, Clone, Copy, Debug)]
#[require(
    Node = popup_panel_node(&PopupMetrics::default(), &PopupShape::default()),
    BackgroundColor,
    BorderColor
)]
pub(crate) struct PopupPanel;

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct PopupMessage;

/// The row that holds the buttons.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct PopupButtons;

/// Which button of a popup the user chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupChoice {
    Confirm,
    Cancel,
}

/// A button of a popup. Activating it makes the popup emit `PopupChosen` with its `choice`.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct PopupButton {
    pub(crate) choice: PopupChoice,
}

fn popup_root_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

pub(crate) fn popup_panel_node(metrics: &PopupMetrics, shape: &PopupShape) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        min_width: metrics.min_width,
        max_width: metrics.max_width,
        padding: metrics.padding,
        row_gap: metrics.row_gap,
        border: UiRect::all(Val::Px(shape.border_width)),
        border_radius: BorderRadius::all(Val::Px(shape.radius)),
        ..default()
    }
}

pub(crate) fn popup_buttons_node(metrics: &PopupMetrics) -> Node {
    Node {
        align_items: AlignItems::Center,
        column_gap: metrics.button_gap,
        ..default()
    }
}
