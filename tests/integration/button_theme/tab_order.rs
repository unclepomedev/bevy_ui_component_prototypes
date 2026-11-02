//! Tests of tab order.

use super::{spawn_primary, spawn_primary_under};
use crate::support::{tab_index, test_app};
use bevy::{
    ecs::system::RunSystemOnce,
    input_focus::{
        InputFocus,
        tab_navigation::{NavAction, TabGroup, TabIndex, TabNavigation},
    },
    prelude::*,
    ui::{InteractionDisabled, auto_directional_navigation::AutoDirectionalNavigation},
};
use bevy_ui_component_prototypes::{ButtonVariantId, icon_button};

#[test]
fn a_custom_tab_index_is_restored_after_re_enabling() {
    let mut app = test_app();
    let button = app.world_mut().spawn((spawn_bundle(), TabIndex(3))).id();
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(tab_index(&app, button), -1);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, button), 3);
}

#[test]
fn a_button_spawned_disabled_starts_outside_the_tab_order() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((spawn_bundle(), InteractionDisabled))
        .id();
    app.update();
    assert_eq!(tab_index(&app, button), -1);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, button), 0);
}

// End to end: Tab skips the disabled middle button, in both directions.
#[test]
fn tab_navigation_skips_a_disabled_button() {
    let mut app = test_app();
    let group = app.world_mut().spawn(TabGroup::default()).id();
    let first = spawn_primary_under(&mut app, group);
    let middle = spawn_primary_under(&mut app, group);
    let last = spawn_primary_under(&mut app, group);
    // Without the visibility plugins nothing is ever propagated, and Tab navigation skips hidden entities.
    for button in [first, middle, last] {
        app.world_mut()
            .entity_mut(button)
            .insert(InheritedVisibility::VISIBLE);
    }

    // Control: with every button enabled, Tab visits the middle one.
    app.update();
    assert_eq!(navigate(&mut app, first, NavAction::Next), middle);

    app.world_mut()
        .entity_mut(middle)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(navigate(&mut app, first, NavAction::Next), last);
    assert_eq!(navigate(&mut app, last, NavAction::Previous), first);
}

// Bevy's automatic arrow-key navigation ignores `InteractionDisabled` too.
#[test]
fn disabled_button_leaves_directional_navigation_and_returns_when_enabled() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((spawn_bundle(), AutoDirectionalNavigation::default()))
        .id();
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_none()
    );

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_some()
    );
}

// Arrow keys must not reach a disabled button, also when the component is added to it later.
#[test]
fn directional_navigation_added_to_a_disabled_button_is_held_back_until_it_is_enabled() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((spawn_bundle(), InteractionDisabled))
        .id();
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(AutoDirectionalNavigation::default());
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_none()
    );

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_some()
    );
}

// Re-enabling must not give directional navigation to a button that never had it.
#[test]
fn re_enabling_does_not_add_directional_navigation() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_none()
    );
}

fn spawn_bundle() -> impl Bundle {
    icon_button()
        .label("button")
        .variant(ButtonVariantId::PRIMARY)
        .build()
}

fn navigate(app: &mut App, from: Entity, action: NavAction) -> Entity {
    app.world_mut()
        .run_system_once(move |navigation: TabNavigation| {
            navigation.navigate(&InputFocus::from_entity(from), action)
        })
        .unwrap()
        .expect("tab navigation finds a target")
}
