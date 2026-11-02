//! Tests of focus.

use super::{fill, spawn_slider};
use crate::{
    probe::probe_theme,
    support::{focus, outline, test_app},
};
use bevy::{prelude::*, ui::InteractionDisabled};

#[test]
fn visible_focus_draws_the_ring_and_uses_the_focused_colors() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    app.update();

    focus(&mut app, slider, true);
    app.update();

    let style = probe_theme().slider;
    assert_eq!(fill(&app, slider), style.fill.focused);
    let ring = outline(&app, slider);
    assert_eq!(ring.color, style.focus_ring.color);
    assert_eq!(ring.width, style.focus_ring.width);
}

// Pointer clicks hide the indicator while the entity keeps focus.
#[test]
fn focus_is_not_shown_while_the_indicator_is_hidden() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    app.update();

    focus(&mut app, slider, false);
    app.update();

    assert_eq!(fill(&app, slider), probe_theme().slider.fill.normal);
    assert_eq!(outline(&app, slider).color, Color::NONE);
}

#[test]
fn moving_focus_moves_the_ring() {
    let mut app = test_app();
    let first = spawn_slider(&mut app);
    let second = spawn_slider(&mut app);
    app.update();
    focus(&mut app, first, true);
    app.update();

    focus(&mut app, second, true);
    app.update();

    assert_eq!(outline(&app, first).color, Color::NONE);
    assert_eq!(
        outline(&app, second).color,
        probe_theme().slider.focus_ring.color
    );
}

#[test]
fn disabled_slider_shows_neither_ring_nor_focused_colors() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    app.update();
    focus(&mut app, slider, true);
    app.world_mut()
        .entity_mut(slider)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(fill(&app, slider), probe_theme().slider.fill.disabled);
    assert_eq!(outline(&app, slider).color, Color::NONE);
}
