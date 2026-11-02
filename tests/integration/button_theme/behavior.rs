//! Tests of behavior wiring.

use super::{label_font, spawn_primary};
use crate::{
    probe::probe_theme,
    support::{outline, test_app},
};
use bevy::{
    a11y::AccessibilityNode, input_focus::tab_navigation::TabIndex,
    picking::hover::PickingInteraction, prelude::*, ui_widgets::Button,
};
use bevy_ui_component_prototypes::{ButtonVariantId, icon_button};

#[test]
fn icon_button_carries_the_bevy_button_behavior_components() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    let world = app.world();
    assert!(world.get::<Button>(button).is_some());
    assert!(world.get::<AccessibilityNode>(button).is_some());
    assert!(world.get::<PickingInteraction>(button).is_some());
    assert_eq!(world.get::<TabIndex>(button).map(|t| t.0), Some(0));
    // `Outline::default()` would be a visible white line; the ring must start hidden.
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[derive(Component)]
struct Marker(u8);

#[derive(Component)]
struct Other;

#[test]
fn label_bundles_land_on_the_label_only_and_accumulate() {
    let mut app = test_app();
    let icon = Handle::<Image>::default();
    let button = app
        .world_mut()
        .spawn(
            icon_button()
                .label("button")
                .icon(icon)
                .variant(ButtonVariantId::PRIMARY)
                .label_bundle(Marker(7))
                .label_bundle(Other)
                .build(),
        )
        .id();
    app.update();

    let world = app.world();
    let children: Vec<Entity> = world.get::<Children>(button).unwrap().iter().collect();
    let labels: Vec<_> = children
        .iter()
        .filter(|c| world.get::<Text>(**c).is_some())
        .collect();
    assert_eq!(labels.len(), 1);
    assert_eq!(world.get::<Marker>(*labels[0]).map(|m| m.0), Some(7));
    assert!(world.get::<Other>(*labels[0]).is_some());
    // Neither the root nor the icon gets them.
    assert!(world.get::<Marker>(button).is_none());
    assert!(
        children
            .iter()
            .filter(|c| world.get::<Text>(**c).is_none())
            .all(|c| world.get::<Marker>(*c).is_none())
    );
    // The theme still styles the label.
    assert_eq!(label_font(&app, button), probe_theme().font);
}

#[test]
fn icon_bundles_land_on_the_icon_only_and_accumulate() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn(
            icon_button()
                .label("button")
                .icon(Handle::<Image>::default())
                .icon_bundle(Marker(3))
                .icon_bundle(Other)
                .build(),
        )
        .id();
    app.update();

    let world = app.world();
    let children: Vec<Entity> = world.get::<Children>(button).unwrap().iter().collect();
    let icons: Vec<_> = children
        .iter()
        .filter(|c| world.get::<ImageNode>(**c).is_some())
        .collect();
    assert_eq!(icons.len(), 1);
    assert_eq!(world.get::<Marker>(*icons[0]).map(|m| m.0), Some(3));
    assert!(world.get::<Other>(*icons[0]).is_some());
    // Neither the root nor the label gets them.
    assert!(world.get::<Marker>(button).is_none());
    assert!(
        children
            .iter()
            .filter(|c| world.get::<ImageNode>(**c).is_none())
            .all(|c| world.get::<Marker>(*c).is_none())
    );
}

#[test]
fn a_button_without_a_label_has_no_label_entity() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn(icon_button().icon(Handle::<Image>::default()).build())
        .id();
    app.update();

    let world = app.world();
    let children: Vec<Entity> = world.get::<Children>(button).unwrap().iter().collect();
    assert_eq!(children.len(), 1);
    assert!(world.get::<Text>(children[0]).is_none());
}

#[test]
fn a_label_bundle_spawns_an_empty_label_to_fill_in_later() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn(icon_button().label_bundle(Marker(1)).build())
        .id();
    app.update();

    let world = app.world();
    let children: Vec<Entity> = world.get::<Children>(button).unwrap().iter().collect();
    assert_eq!(children.len(), 1);
    assert_eq!(
        world.get::<Text>(children[0]).map(|t| t.0.as_str()),
        Some("")
    );
    assert!(world.get::<Marker>(children[0]).is_some());
}

#[test]
fn the_label_text_is_the_one_given() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn(icon_button().label("OK").label_bundle(Marker(1)).build())
        .id();
    app.update();

    let world = app.world();
    let label = world.get::<Children>(button).unwrap()[0];
    assert_eq!(world.get::<Text>(label).map(|t| t.0.as_str()), Some("OK"));
}

#[test]
fn a_label_never_wraps_because_the_button_is_one_line_high() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    assert_eq!(label_layout(&app, button).linebreak, LineBreak::NoWrap);
}

fn label_layout(app: &App, button: Entity) -> TextLayout {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextLayout>(child).copied())
        .expect("button has a label child")
}
