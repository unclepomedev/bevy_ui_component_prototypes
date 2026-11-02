//! Tests of interaction states.

use super::{label_color, primary, primary_mut, spawn_primary};
use crate::{
    probe::probe_theme,
    support::{bg, border_color, test_app},
};
use bevy::{picking::hover::PickingInteraction, prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::{GlobalTheme, StateColors};

#[test]
fn hover_and_press_change_colors_and_release_restores_them() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    let style = primary(&probe_theme());
    app.update();

    for (state, expected) in [
        (PickingInteraction::Hovered, style.bg.hovered),
        (PickingInteraction::Pressed, style.bg.pressed),
        (PickingInteraction::None, style.bg.normal),
    ] {
        app.world_mut().entity_mut(button).insert(state);
        app.update();
        assert_eq!(bg(&app, button), expected);
    }
}

#[test]
fn the_border_color_follows_the_state() {
    let mut app = test_app();
    let mut theme = probe_theme();
    primary_mut(&mut theme).border = StateColors {
        normal: Color::srgb(0.1, 0.0, 0.0),
        hovered: Color::srgb(0.2, 0.0, 0.0),
        pressed: Color::srgb(0.3, 0.0, 0.0),
        disabled: Color::srgb(0.4, 0.0, 0.0),
        focused: Color::srgb(0.5, 0.0, 0.0),
    };
    let border = primary(&theme).border;
    app.insert_resource(GlobalTheme(theme));
    let button = spawn_primary(&mut app);
    app.update();

    for (state, expected) in [
        (PickingInteraction::Hovered, border.hovered),
        (PickingInteraction::Pressed, border.pressed),
        (PickingInteraction::None, border.normal),
    ] {
        app.world_mut().entity_mut(button).insert(state);
        app.update();
        assert_eq!(border_color(&app, button), BorderColor::all(expected));
    }
}

#[test]
fn interaction_disabled_grays_the_button_and_removal_restores_it() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    let style = primary(&probe_theme());
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(bg(&app, button), style.bg.disabled);
    assert_eq!(label_color(&app, button), style.fg.disabled);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(bg(&app, button), style.bg.normal);
    assert_eq!(label_color(&app, button), style.fg.normal);
}
