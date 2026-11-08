//! Tests of the structure of a popup and the pointer.

use super::{message, panel, spawn_popup};
use crate::support::{passes_through, test_app};
use bevy::{picking::Pickable, prelude::*};
use bevy_ui_component_prototypes::PopupRoot;

#[test]
fn a_popup_is_a_root_with_a_panel_that_holds_the_message() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");

    let world = app.world();
    assert!(world.get::<PopupRoot>(root).is_some());
    assert_eq!(world.get::<Children>(root).map(|c| c.len()), Some(1));
    assert_eq!(
        world.get::<Children>(panel(&app, root)).map(|c| c.len()),
        Some(1)
    );
    assert_eq!(world.get::<Text>(message(&app, root)).unwrap().0, "Quit?");
}

#[test]
fn the_root_covers_its_parent_and_centers_the_panel() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");

    let node = app.world().get::<Node>(root).unwrap();
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(
        (node.width, node.height),
        (Val::Percent(100.0), Val::Percent(100.0))
    );
    assert_eq!(node.justify_content, JustifyContent::Center);
    assert_eq!(node.align_items, AlignItems::Center);
}

#[test]
fn nothing_shows_before_the_first_style_sync() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");

    let world = app.world();
    // The defaults of these are black or white: they must not show before the sync.
    assert_eq!(world.get::<BackgroundColor>(root).unwrap().0, Color::NONE);
    assert_eq!(
        world.get::<BackgroundColor>(panel(&app, root)).unwrap().0,
        Color::NONE
    );
    assert_eq!(
        world.get::<TextColor>(message(&app, root)).unwrap().0,
        Color::NONE
    );
}

#[test]
fn the_root_blocks_the_pointer_and_the_message_lets_it_through() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");

    let world = app.world();
    let root_pickable = world.get::<Pickable>(root).unwrap();
    assert!(root_pickable.should_block_lower && root_pickable.is_hoverable);
    assert!(passes_through(
        world.get::<Pickable>(message(&app, root)).unwrap()
    ));
}
