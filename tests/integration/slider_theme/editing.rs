//! Tests of the editing mode.

use super::{fill, rail, spawn_slider, thumb};
use crate::{
    probe::probe_theme,
    support::{focus, outline, test_app},
};
use bevy::{prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::SliderEditing;

#[test]
fn editing_changes_the_thumb_and_the_ring_and_removal_restores_them() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = probe_theme().slider;
    app.update();
    focus(&mut app, slider, true);
    app.update();
    let idle_ring = outline(&app, slider);
    assert_eq!(idle_ring.color, style.focus_ring.color);

    app.world_mut().entity_mut(slider).insert(SliderEditing);
    app.update();
    assert_eq!(thumb(&app, slider), style.editing_thumb);
    let ring = outline(&app, slider);
    assert_eq!(ring.color, style.editing_ring);
    assert_eq!(ring.width, idle_ring.width);
    // The rail and the fill are not part of it.
    assert_eq!(rail(&app, slider), style.rail.focused);
    assert_eq!(fill(&app, slider), style.fill.focused);

    // Removal is not a `Changed`; it has to be picked up through `RemovedComponents`.
    app.world_mut().entity_mut(slider).remove::<SliderEditing>();
    app.update();
    assert_eq!(thumb(&app, slider), style.thumb.focused);
    assert_eq!(outline(&app, slider).color, style.focus_ring.color);
}

#[test]
fn editing_shows_no_ring_while_the_focus_indicator_is_hidden() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    app.update();
    focus(&mut app, slider, false);
    app.world_mut().entity_mut(slider).insert(SliderEditing);
    app.update();

    assert_eq!(outline(&app, slider).color, Color::NONE);
    assert_eq!(thumb(&app, slider), probe_theme().slider.editing_thumb);
}

#[test]
fn disabled_slider_ignores_editing() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = probe_theme().slider;
    app.update();

    focus(&mut app, slider, true);
    app.world_mut()
        .entity_mut(slider)
        .insert((SliderEditing, InteractionDisabled));
    app.update();

    assert_eq!(thumb(&app, slider), style.thumb.disabled);
    assert_eq!(outline(&app, slider).color, Color::NONE);
}
