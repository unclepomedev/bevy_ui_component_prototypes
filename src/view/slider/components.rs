use crate::view::KeyboardNavigable;
use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Slider, SliderOrientation, TrackClick},
};

const THUMB_SIZE: f32 = 20.0;
const RAIL_HEIGHT: f32 = 6.0;

/// A themed horizontal slider: a rail, the filled part of it, and a thumb.
///
/// Behavior comes from Bevy's [`Slider`]: dragging the thumb or clicking the rail, and the arrow
/// keys, Home and End on the focused slider, emit [`bevy::ui_widgets::ValueChange<f32>`] carrying
/// the requested value. The slider never changes its own value: the app applies the change by
/// inserting a new [`bevy::ui_widgets::SliderValue`] (or adds
/// [`bevy::ui_widgets::slider_self_update`] to let Bevy do exactly that).
///
/// The requested value is continuous. `SliderStep` is what a key press moves by; to make the slider
/// discrete, round the value in the app's `ValueChange` observer.
///
/// The width follows the parent layout (`100%`). A text placed next to the slider is not
/// associated with it: give the slider a name for screen readers with `AccessibleLabel`.
///
/// To reach it with the Tab key, put it under a `TabGroup`. The focus ring is drawn with the entity's
/// own `Outline` while the focus indicator is visible, so do not set an `Outline` on it yourself.
#[derive(Component)]
#[require(
    // Clicking the rail moves the value there; the thumb is still dragged relative to where it is.
    Slider = Slider { track_click: TrackClick::Snap, orientation: SliderOrientation::Horizontal },
    Hovered,
    TabIndex(0),
    KeyboardNavigable,
    Outline = hidden_outline(),
    Node = slider_node()
)]
pub struct ValueSlider;

/// The part of a [`ValueSlider`] a style system writes to.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SliderPart {
    Rail,
    Fill,
    Thumb,
}

pub(crate) fn slider_node() -> Node {
    Node {
        width: percent(100),
        height: px(THUMB_SIZE),
        ..default()
    }
}

pub(crate) fn rail_node() -> Node {
    // The rail spans the whole width, centered vertically.
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        right: px(0),
        top: px((THUMB_SIZE - RAIL_HEIGHT) / 2.0),
        height: px(RAIL_HEIGHT),
        border_radius: BorderRadius::all(px(RAIL_HEIGHT / 2.0)),
        ..default()
    }
}

pub(crate) fn travel_node() -> Node {
    // The thumb's left edge travels from 0 to `width - thumb`, so this area is shortened by the thumb width;
    // `left: 0%..100%` of it is then where the thumb must be. Bevy's pointer math assumes the same travel.
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        right: px(THUMB_SIZE),
        top: px(0),
        bottom: px(0),
        ..default()
    }
}

pub(crate) fn fill_node() -> Node {
    // Grows from the left edge of the travel area to the thumb's left edge; the thumb covers the rest.
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px((THUMB_SIZE - RAIL_HEIGHT) / 2.0),
        width: percent(0),
        height: px(RAIL_HEIGHT),
        border_radius: BorderRadius::all(px(RAIL_HEIGHT / 2.0)),
        ..default()
    }
}

pub(crate) fn thumb_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(0),
        top: px(0),
        width: px(THUMB_SIZE),
        height: px(THUMB_SIZE),
        border_radius: BorderRadius::all(px(THUMB_SIZE / 2.0)),
        ..default()
    }
}

fn hidden_outline() -> Outline {
    Outline::new(Val::Px(0.0), Val::Px(0.0), Color::NONE)
}
