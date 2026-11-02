//! Tests of initial styling.

use super::primary;
use crate::{
    probe::probe_theme,
    support::{bg, test_app},
};
use bevy::{picking::hover::PickingInteraction, prelude::*, ui_widgets::Button};
use bevy_ui_component_prototypes::{ButtonVariant, ButtonVariantId};

#[test]
fn a_button_attached_to_an_existing_entity_is_styled_after_one_update() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            PickingInteraction::None,
            ButtonVariant::Named(ButtonVariantId::PRIMARY),
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Outline::default(),
        ))
        .id();
    app.update();

    app.world_mut().entity_mut(entity).insert(Button);
    app.update();

    assert_eq!(bg(&app, entity), primary(&probe_theme()).bg.normal);
}
