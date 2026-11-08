//! Tests of initial styling and the position of the parts.

use super::{fill, fill_width, rail, spawn_slider, thumb, thumb_left};
use crate::{probe::probe_theme, support::test_app};
use bevy::{
    prelude::*,
    ui_widgets::{SliderRange, SliderValue},
};
use bevy_ui_component_prototypes::slider::slider;

#[test]
fn a_slider_is_styled_after_one_update() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);

    app.update();

    let style = probe_theme().slider;
    assert_eq!(rail(&app, slider), style.rail.normal);
    assert_eq!(fill(&app, slider), style.fill.normal);
    assert_eq!(thumb(&app, slider), style.thumb.normal);
    assert_eq!(fill_width(&app, slider), Val::Percent(0.0));
    assert_eq!(thumb_left(&app, slider), Val::Percent(0.0));
}

#[test]
fn the_initial_value_places_the_fill_and_the_thumb_after_one_update() {
    let mut app = test_app();
    let slider = app.world_mut().spawn(slider().value(0.5).build()).id();

    app.update();

    assert_eq!(fill_width(&app, slider), Val::Percent(50.0));
    assert_eq!(thumb_left(&app, slider), Val::Percent(50.0));
}

#[test]
fn the_position_follows_the_value_and_the_range() {
    let mut app = test_app();
    let slider = app
        .world_mut()
        .spawn(slider().range(100.0, 300.0).value(150.0).build())
        .id();
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(25.0));

    app.world_mut()
        .entity_mut(slider)
        .insert(SliderValue(300.0));
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(100.0));
    assert_eq!(fill_width(&app, slider), Val::Percent(100.0));

    // The range changing under a fixed value moves the thumb as well.
    app.world_mut()
        .entity_mut(slider)
        .insert(SliderRange::new(100.0, 500.0));
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(50.0));
}

// A value outside the range must not push the thumb out of the rail.
#[test]
fn the_position_never_leaves_the_rail() {
    let mut app = test_app();
    let slider = app.world_mut().spawn(slider().value(7.0).build()).id();
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(100.0));

    app.world_mut().entity_mut(slider).insert(SliderValue(-3.0));
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(0.0));
}
