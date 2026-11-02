//! Tests of variants.

use crate::{
    probe::probe_theme,
    support::{bg, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{
    ButtonStyle, ButtonVariant, ButtonVariantId, GlobalTheme, StateColors, Theme, icon_button,
};

const GOLD: ButtonVariantId = ButtonVariantId::new("gold");

#[test]
fn the_default_variant_is_the_base_style_of_the_theme() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(gold_theme()));
    let button = app
        .world_mut()
        .spawn(icon_button().label("button").build())
        .id();

    app.update();

    assert_eq!(bg(&app, button), gold_theme().buttons.base.bg.normal);
}

#[test]
fn changing_the_variant_of_a_button_restyles_it() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(gold_theme()));
    let button = spawn_variant(&mut app, ButtonVariantId::PRIMARY);
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(ButtonVariant::Named(GOLD));
    app.update();

    assert_eq!(bg(&app, button), gold_style().bg.normal);
}

#[test]
fn a_theme_change_that_drops_a_name_restyles_its_buttons_with_the_base_style() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(gold_theme()));
    let button = spawn_variant(&mut app, GOLD);
    app.update();

    app.insert_resource(GlobalTheme(probe_theme()));
    app.update();

    assert_eq!(bg(&app, button), probe_theme().buttons.base.bg.normal);
}

fn spawn_variant(app: &mut App, variant: impl Into<ButtonVariant>) -> Entity {
    app.world_mut()
        .spawn(icon_button().label("button").variant(variant).build())
        .id()
}

/// A style that differs from every style of the probe theme.
fn gold_style() -> ButtonStyle {
    let mut style = ButtonStyle::ghost(StateColors::uniform(Color::srgb(1.0, 0.85, 0.3)));
    style.bg = StateColors::uniform(Color::srgb(0.3, 0.2, 0.0));
    style
}

/// The probe theme plus `GOLD`.
fn gold_theme() -> Theme {
    let mut theme = probe_theme();
    theme.buttons.named.insert(GOLD, gold_style());
    theme
}
