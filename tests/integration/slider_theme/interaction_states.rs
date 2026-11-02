//! Tests of interaction states.

use super::{fill, rail, spawn_slider, thumb};
use crate::{probe::probe_theme, support::test_app};
use bevy::{
    picking::hover::Hovered,
    ui::{InteractionDisabled, Pressed},
    ui_widgets::SliderDragState,
};

#[test]
fn hovering_the_slider_changes_the_colors() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = probe_theme().slider;
    app.update();

    // `Hovered` already includes the descendants, so it covers the pickable thumb.
    app.world_mut().entity_mut(slider).insert(Hovered(true));
    app.update();
    assert_eq!(fill(&app, slider), style.fill.hovered);

    app.world_mut().entity_mut(slider).insert(Hovered(false));
    app.update();
    assert_eq!(fill(&app, slider), style.fill.normal);
}

#[test]
fn a_rail_click_shows_the_pressed_colors_until_release() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = probe_theme().slider;
    app.update();

    app.world_mut().entity_mut(slider).insert(Pressed);
    app.update();
    assert_eq!(fill(&app, slider), style.fill.pressed);

    // Removal is not a `Changed`; it has to be picked up through `RemovedComponents`.
    app.world_mut().entity_mut(slider).remove::<Pressed>();
    app.update();
    assert_eq!(fill(&app, slider), style.fill.normal);
}

// Pressing the thumb itself inserts no `Pressed`: only the drag state tells.
#[test]
fn dragging_shows_the_pressed_colors_until_the_drag_ends() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = probe_theme().slider;
    app.update();

    app.world_mut()
        .get_mut::<SliderDragState>(slider)
        .unwrap()
        .dragging = true;
    app.update();
    assert_eq!(fill(&app, slider), style.fill.pressed);

    app.world_mut()
        .get_mut::<SliderDragState>(slider)
        .unwrap()
        .dragging = false;
    app.update();
    assert_eq!(fill(&app, slider), style.fill.normal);
}

#[test]
fn interaction_disabled_grays_the_slider_and_removal_restores_it() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = probe_theme().slider;
    app.update();

    app.world_mut()
        .entity_mut(slider)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(rail(&app, slider), style.rail.disabled);
    assert_eq!(fill(&app, slider), style.fill.disabled);
    assert_eq!(thumb(&app, slider), style.thumb.disabled);

    app.world_mut()
        .entity_mut(slider)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(fill(&app, slider), style.fill.normal);
    assert_eq!(thumb(&app, slider), style.thumb.normal);
}
