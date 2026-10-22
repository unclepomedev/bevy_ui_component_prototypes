use crate::{
    style::slider::{SliderGeometry, SliderShape},
    view::KeyboardNavigable,
};
use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Slider, SliderOrientation, TrackClick},
};

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
/// The shape comes from the theme's `SliderStyle::shape`.
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
    Node = slider_node(&SliderShape::default().geometry())
)]
pub struct ValueSlider;

/// Insert this on a [`ValueSlider`] while the app lets the keys change its value (an "editing"
/// mode, entered with Enter, for instance), and remove it afterward. It changes nothing but the look.
#[derive(Component, Clone, Copy, Default)]
pub struct SliderEditing;

/// The part of a [`ValueSlider`] a style system writes to.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SliderPart {
    Rail,
    /// The fill and the thumb are inside it.
    Travel,
    Fill,
    Thumb,
}

pub(crate) fn slider_node(geometry: &SliderGeometry) -> Node {
    Node {
        width: percent(100),
        height: px(geometry.height),
        ..default()
    }
}

pub(crate) fn rail_node(geometry: &SliderGeometry) -> Node {
    // The rail spans the whole width, centered vertically.
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        right: px(0),
        top: px(geometry.rail_top),
        height: px(geometry.rail_height),
        border_radius: BorderRadius::all(px(geometry.rail_radius)),
        ..default()
    }
}

pub(crate) fn travel_node(geometry: &SliderGeometry) -> Node {
    // The thumb's left edge travels from 0 to `width - thumb`, so this area is shortened by the thumb width;
    // `left: 0%..100%` of it is then where the thumb must be. Bevy's pointer math assumes the same travel.
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        right: px(geometry.travel_inset),
        top: px(0),
        bottom: px(0),
        ..default()
    }
}

pub(crate) fn fill_node(geometry: &SliderGeometry) -> Node {
    // Grows from the left edge of the travel area to the thumb's left edge; the thumb covers the rest.
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(geometry.rail_top),
        width: percent(0),
        height: px(geometry.rail_height),
        border_radius: BorderRadius::all(px(geometry.rail_radius)),
        ..default()
    }
}

pub(crate) fn thumb_node(geometry: &SliderGeometry) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(0),
        top: px(geometry.thumb_top),
        width: px(geometry.thumb_size),
        height: px(geometry.thumb_size),
        border_radius: BorderRadius::all(px(geometry.thumb_radius)),
        ..default()
    }
}

fn hidden_outline() -> Outline {
    Outline::new(Val::Px(0.0), Val::Px(0.0), Color::NONE)
}
