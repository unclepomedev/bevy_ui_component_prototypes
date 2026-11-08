//! Tests of the stacking direction of a host.

use crate::support::test_app;
use bevy::prelude::*;
use bevy_ui_component_prototypes::toast::{ToastHost, ToastStack, toast_host};

#[test]
fn the_newest_toast_goes_to_the_end_the_stack_names() {
    // The newest toast is the last child.
    for (stack, direction) in [
        (ToastStack::NewestAtBottom, FlexDirection::Column),
        (ToastStack::NewestAtTop, FlexDirection::ColumnReverse),
    ] {
        let mut app = test_app();
        let host = app
            .world_mut()
            .spawn(toast_host().stack(stack).build())
            .id();

        app.update();

        assert_eq!(flex_direction(&app, host), direction, "{stack:?}");
    }
}

#[test]
fn changing_the_stack_changes_the_direction() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();
    app.update();

    app.world_mut().entity_mut(host).insert(ToastHost {
        stack: ToastStack::NewestAtTop,
        ..default()
    });
    app.update();

    assert_eq!(flex_direction(&app, host), FlexDirection::ColumnReverse);
}

// A `Node` spawned together with the host replaces the one the host requires.
#[test]
fn the_apps_node_keeps_its_own_fields_and_gets_the_direction() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn((
            toast_host().stack(ToastStack::NewestAtTop).build(),
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                top: px(24),
                align_items: AlignItems::FlexEnd,
                ..default()
            },
        ))
        .id();

    app.update();

    let node = app.world().get::<Node>(host).unwrap();
    assert_eq!(node.flex_direction, FlexDirection::ColumnReverse);
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.align_items, AlignItems::FlexEnd);
}

#[test]
fn a_node_inserted_later_gets_the_direction_back() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn(toast_host().stack(ToastStack::NewestAtTop).build())
        .id();
    app.update();

    app.world_mut().entity_mut(host).insert(Node::default());
    app.update();

    assert_eq!(flex_direction(&app, host), FlexDirection::ColumnReverse);
}

fn flex_direction(app: &App, host: Entity) -> FlexDirection {
    app.world().get::<Node>(host).unwrap().flex_direction
}
