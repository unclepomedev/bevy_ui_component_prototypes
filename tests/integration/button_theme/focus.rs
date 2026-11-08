//! Tests of focus.

use super::{label_color, primary, spawn_primary};
use crate::{
    probe::probe_theme,
    support::{bg, focus, outline, test_app},
};
use bevy::{picking::hover::PickingInteraction, prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::common::FocusIndicator;

#[test]
fn visible_focus_draws_the_ring_and_uses_the_focused_colors() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, true);
    app.update();

    let style = primary(&probe_theme());
    assert_eq!(bg(&app, button), style.bg.focused);
    assert_eq!(label_color(&app, button), style.fg.focused);
    let ring = outline(&app, button);
    assert_eq!(ring.color, style.focus_ring.color);
    assert_eq!(ring.width, style.focus_ring.width);
}

#[test]
fn moving_focus_moves_the_ring() {
    let mut app = test_app();
    let first = spawn_primary(&mut app);
    let second = spawn_primary(&mut app);
    app.update();
    focus(&mut app, first, true);
    app.update();

    focus(&mut app, second, true);
    app.update();

    let style = primary(&probe_theme());
    assert_eq!(outline(&app, first).color, Color::NONE);
    assert_eq!(bg(&app, first), style.bg.normal);
    assert_eq!(outline(&app, second).color, style.focus_ring.color);
    assert_eq!(bg(&app, second), style.bg.focused);
}

#[test]
fn an_always_visible_indicator_draws_the_ring_while_bevy_hides_it() {
    let mut app = test_app();
    app.insert_resource(FocusIndicator::Always);
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, false);
    app.update();

    let style = primary(&probe_theme());
    assert_eq!(outline(&app, button).color, style.focus_ring.color);
    assert_eq!(bg(&app, button), style.bg.focused);

    app.insert_resource(FocusIndicator::Auto);
    app.update();
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[test]
fn hover_wins_over_focus_for_colors_but_the_ring_stays() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.world_mut()
        .entity_mut(button)
        .insert(PickingInteraction::Hovered);
    app.update();

    let style = primary(&probe_theme());
    assert_eq!(bg(&app, button), style.bg.hovered);
    assert_eq!(outline(&app, button).color, style.focus_ring.color);
}

#[test]
fn disabled_button_shows_neither_ring_nor_focused_colors() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.disabled);
    assert_eq!(outline(&app, button).color, Color::NONE);
}
