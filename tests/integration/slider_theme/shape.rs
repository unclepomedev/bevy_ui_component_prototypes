//! Tests of the shape.

use super::{fill_part, rail_part, spawn_slider, thumb_part, travel_part};
use crate::{
    probe::probe_theme,
    support::{node, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{GlobalTheme, Theme, slider::SliderShape};

#[test]
fn the_default_shape_is_a_thin_rail_and_a_round_thumb() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);

    app.update();

    let slider_node = node(&app, slider);
    let rail_node = node(&app, rail_part(&app, slider));
    let thumb_node = node(&app, thumb_part(&app, slider));
    let travel_node = node(&app, travel_part(&app, slider));

    // The slider is as tall as its thumb, which is larger than the rail.
    assert!(px(rail_node.height) < px(thumb_node.height));
    assert_eq!(slider_node.height, thumb_node.height);

    // The thin rail is centered on the slider, and its radius is half its height.
    assert_eq!(
        rail_node.top,
        Val::Px((px(slider_node.height) - px(rail_node.height)) / 2.0)
    );
    assert_eq!(
        rail_node.border_radius,
        BorderRadius::all(Val::Px(px(rail_node.height) / 2.0))
    );

    // The thumb is a circle as tall as the slider.
    assert_eq!(thumb_node.width, thumb_node.height);
    assert_eq!(thumb_node.top, Val::Px(0.0));
    assert_eq!(
        thumb_node.border_radius,
        BorderRadius::all(Val::Px(px(thumb_node.height) / 2.0))
    );

    // The travel area is shortened by the thumb width so the thumb stays inside the slider.
    assert_eq!(travel_node.right, thumb_node.width);
}

#[test]
fn a_custom_shape_reaches_every_part_after_one_update() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(squared_theme()));
    let slider = spawn_slider(&mut app);

    app.update();

    let square = BorderRadius::all(Val::Px(0.0));
    // The slider is as tall as its tallest part: here the rail.
    assert_eq!(node(&app, slider).height, Val::Px(10.0));
    assert_eq!(node(&app, rail_part(&app, slider)).height, Val::Px(10.0));
    assert_eq!(node(&app, rail_part(&app, slider)).border_radius, square);
    assert_eq!(node(&app, fill_part(&app, slider)).height, Val::Px(10.0));
    assert_eq!(node(&app, fill_part(&app, slider)).border_radius, square);
    // No thumb: it takes no room, and the fill may use the whole width.
    assert_eq!(node(&app, thumb_part(&app, slider)).width, Val::Px(0.0));
    assert_eq!(node(&app, thumb_part(&app, slider)).height, Val::Px(0.0));
    assert_eq!(node(&app, travel_part(&app, slider)).right, Val::Px(0.0));
}

/// The probe theme with a thick, square rail and no thumb.
fn squared_theme() -> Theme {
    let mut theme = probe_theme();
    theme.slider.shape = SliderShape {
        rail_height: 10.0,
        thumb_size: 0.0,
        rounded: false,
        ..default()
    };
    theme
}

fn px(val: Val) -> f32 {
    match val {
        Val::Px(pixels) => pixels,
        other => panic!("expected Val::Px, got {other:?}"),
    }
}
