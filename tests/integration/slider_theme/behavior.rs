//! Tests of behavior wiring.

use super::spawn_slider;
use crate::support::{outline, test_app};
use accesskit::Role;
use bevy::{
    a11y::AccessibilityNode,
    input_focus::tab_navigation::TabIndex,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Slider, SliderOrientation, SliderRange, SliderStep, SliderValue, TrackClick},
};
use bevy_ui_component_prototypes::slider::slider;

#[test]
fn slider_carries_the_bevy_slider_behavior_components() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);

    let world = app.world();
    let behavior = world.get::<Slider>(slider).unwrap();
    // Clicking the rail has to move the value there, and the width must not decide the orientation.
    assert_eq!(behavior.track_click, TrackClick::Snap);
    assert_eq!(behavior.orientation, SliderOrientation::Horizontal);
    assert_eq!(
        world
            .get::<AccessibilityNode>(slider)
            .map(|node| node.role()),
        Some(Role::Slider)
    );
    assert!(world.get::<Hovered>(slider).is_some());
    assert_eq!(world.get::<TabIndex>(slider).map(|t| t.0), Some(0));
    // `Outline::default()` would be a visible white line; the ring must start hidden.
    assert_eq!(outline(&app, slider).color, Color::NONE);
}

#[test]
fn the_builder_defaults_to_a_unit_range_and_one_percent_keys() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);

    let world = app.world();
    assert_eq!(
        world.get::<SliderRange>(slider),
        Some(&SliderRange::default())
    );
    assert_eq!(world.get::<SliderValue>(slider), Some(&SliderValue(0.0)));
    assert_eq!(world.get::<SliderStep>(slider), Some(&SliderStep(0.01)));
}

#[test]
fn the_builder_options_reach_the_bevy_components() {
    let mut app = test_app();
    let slider = app
        .world_mut()
        .spawn(slider().range(10.0, 30.0).value(15.0).step(5.0).build())
        .id();

    let world = app.world();
    assert_eq!(
        world.get::<SliderRange>(slider),
        Some(&SliderRange::new(10.0, 30.0))
    );
    assert_eq!(world.get::<SliderValue>(slider), Some(&SliderValue(15.0)));
    assert_eq!(world.get::<SliderStep>(slider), Some(&SliderStep(5.0)));
}

// A step is a key increment: without one, a wide range must not move by a whole unit per key press.
#[test]
fn the_default_step_scales_with_the_range() {
    let mut app = test_app();
    let slider = app
        .world_mut()
        .spawn(slider().range(0.0, 200.0).build())
        .id();

    assert_eq!(
        app.world().get::<SliderStep>(slider),
        Some(&SliderStep(2.0))
    );
}
