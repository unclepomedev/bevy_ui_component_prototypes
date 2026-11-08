//! Tests of the segments.

use super::{fill, fill_part, rail, rail_part, thumb_part, travel_part};
use crate::{
    gradient::{painted_blocks, stops},
    probe::probe_theme,
    support::{node, test_app},
};
use bevy::{picking::hover::Hovered, prelude::*, ui_widgets::SliderValue};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme,
    common::BarSegments,
    slider::{SliderShape, slider},
};

#[test]
fn a_segmented_rail_and_fill_are_blocks_drawn_by_gradients() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let slider = app.world_mut().spawn(slider().value(0.8).build()).id();

    app.update();

    let style = blocks_theme().slider;
    // The plain backgrounds are out of the way, so that the gaps show through.
    assert_eq!(rail(&app, slider), Color::NONE);
    assert_eq!(fill(&app, slider), Color::NONE);
    let rail_stops = stops(&app, rail_part(&app, slider));
    assert_eq!(painted_blocks(&rail_stops, style.rail.normal), 10);
    assert_eq!(
        painted_blocks(&stops(&app, fill_part(&app, slider)), style.fill.normal),
        8
    );
}

#[test]
fn the_lit_blocks_follow_the_value() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let slider = app.world_mut().spawn(slider().value(0.8).build()).id();
    app.update();

    app.world_mut().entity_mut(slider).insert(SliderValue(0.3));
    app.update();

    let fill_color = blocks_theme().slider.fill.normal;
    assert_eq!(
        painted_blocks(&stops(&app, fill_part(&app, slider)), fill_color),
        3
    );
}

// The fill spans the whole bar and lights blocks; it does not grow, and nothing is left of the thumb.
#[test]
fn a_segmented_fill_spans_the_bar_and_there_is_no_thumb() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let slider = app.world_mut().spawn(slider().value(0.4).build()).id();

    app.update();

    assert_eq!(
        node(&app, fill_part(&app, slider)).width,
        Val::Percent(100.0)
    );
    assert_eq!(node(&app, thumb_part(&app, slider)).width, Val::Px(0.0));
    assert_eq!(node(&app, travel_part(&app, slider)).right, Val::Px(0.0));
}

#[test]
fn colors_of_the_blocks_follow_the_state() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let slider = app.world_mut().spawn(slider().value(0.5).build()).id();
    app.update();

    app.world_mut().entity_mut(slider).insert(Hovered(true));
    app.update();

    let style = blocks_theme().slider;
    assert_eq!(
        painted_blocks(&stops(&app, fill_part(&app, slider)), style.fill.hovered),
        5
    );
}

#[test]
fn a_plain_slider_has_no_gradients_and_keeps_its_colors() {
    let mut app = test_app();
    let slider = app.world_mut().spawn(slider().value(0.5).build()).id();

    app.update();

    let style = probe_theme().slider;
    assert!(stops(&app, rail_part(&app, slider)).is_empty());
    assert!(stops(&app, fill_part(&app, slider)).is_empty());
    assert_eq!(rail(&app, slider), style.rail.normal);
    assert_eq!(fill(&app, slider), style.fill.normal);
    assert_eq!(
        node(&app, fill_part(&app, slider)).width,
        Val::Percent(50.0)
    );
}

#[test]
fn swapping_between_segmented_and_plain_themes_switches_the_look_back_and_forth() {
    let mut app = test_app();
    let slider = app.world_mut().spawn(slider().value(0.5).build()).id();
    app.update();

    app.insert_resource(GlobalTheme(blocks_theme()));
    app.update();
    assert!(!stops(&app, rail_part(&app, slider)).is_empty());
    assert_eq!(fill(&app, slider), Color::NONE);

    app.insert_resource(GlobalTheme(probe_theme()));
    app.update();
    assert!(stops(&app, rail_part(&app, slider)).is_empty());
    assert!(stops(&app, fill_part(&app, slider)).is_empty());
    assert_eq!(fill(&app, slider), probe_theme().slider.fill.normal);
    assert_eq!(
        node(&app, fill_part(&app, slider)).width,
        Val::Percent(50.0)
    );
}

/// The probe theme with a ten-block bar.
fn blocks_theme() -> Theme {
    let mut theme = probe_theme();
    theme.slider.shape = SliderShape {
        rounded: false,
        segments: Some(BarSegments {
            count: 10,
            gap: 0.2,
        }),
        ..default()
    };
    theme
}
