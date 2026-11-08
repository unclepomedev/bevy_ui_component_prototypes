//! Tests of the host.

use crate::support::{passes_through, test_app};
use bevy::{picking::Pickable, prelude::*};
use bevy_ui_component_prototypes::toast::{ToastHost, ToastStack, toast_host};

#[test]
fn a_host_has_the_defaults_and_lets_the_pointer_through() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();

    let world = app.world();
    assert_eq!(
        world.get::<ToastHost>(host),
        Some(&ToastHost {
            max: 5,
            stack: ToastStack::NewestAtBottom,
            slide: Val2::ZERO,
        })
    );
    assert!(world.get::<Node>(host).is_some());
    assert!(passes_through(world.get::<Pickable>(host).unwrap()));
}

#[test]
fn the_host_options_reach_the_component() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn(
            toast_host()
                .max(2)
                .stack(ToastStack::NewestAtTop)
                .slide(Val2::percent(100, 0))
                .build(),
        )
        .id();

    assert_eq!(
        app.world().get::<ToastHost>(host),
        Some(&ToastHost {
            max: 2,
            stack: ToastStack::NewestAtTop,
            slide: Val2::percent(100, 0),
        })
    );
}
