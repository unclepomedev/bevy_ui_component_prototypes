//! Tests of theme resolution.

use super::{fill, spawn_slider, thumb};
use crate::{
    probe::probe_theme,
    support::{focus, node, outline, red, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{
    GlobalTheme, SliderEditing, SliderShape, StateColors, Theme, slider,
};

#[test]
fn global_theme_change_updates_unscoped_sliders_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_slider(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_slider_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_fill_theme()));
    app.update();

    assert_eq!(node(&app, unscoped).height, Val::Px(10.0));
    assert_eq!(fill(&app, unscoped), red());
    assert_eq!(
        node(&app, scoped).height,
        Val::Px(probe_theme().slider.shape.thumb_size)
    );
    assert_eq!(fill(&app, scoped), probe_theme().slider.fill.normal);
}

#[test]
fn scoped_theme_change_updates_only_sliders_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_slider(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_slider_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_fill_theme();
    app.update();

    assert_eq!(node(&app, inside).height, Val::Px(10.0));
    assert_eq!(fill(&app, inside), red());
    assert_eq!(
        node(&app, outside).height,
        Val::Px(probe_theme().slider.shape.thumb_size)
    );
    assert_eq!(fill(&app, outside), probe_theme().slider.fill.normal);
}

#[test]
fn slider_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_fill_theme()).id();
    app.update();

    let late = spawn_slider_under(&mut app, panel);
    app.update();

    assert_eq!(fill(&app, late), red());
}

#[test]
fn editing_follows_a_custom_theme() {
    let mut app = test_app();
    let mut theme = probe_theme();
    theme.slider.editing_thumb = red();
    theme.slider.editing_ring = red();
    app.insert_resource(GlobalTheme(theme));
    let slider = app
        .world_mut()
        .spawn((slider().build(), SliderEditing))
        .id();
    app.update();
    focus(&mut app, slider, true);
    app.update();

    assert_eq!(thumb(&app, slider), red());
    assert_eq!(outline(&app, slider).color, red());
}

fn spawn_slider_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((slider().build(), ChildOf(parent)))
        .id()
}

/// The probe theme with a red fill.
fn red_fill_theme() -> Theme {
    let mut theme = probe_theme();
    theme.slider.shape = SliderShape {
        rail_height: 10.0,
        thumb_size: 0.0,
        rounded: false,
        ..default()
    };
    theme.slider.fill = StateColors::uniform(red());
    theme
}
