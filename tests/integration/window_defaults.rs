//! Behavior tests for `WindowDefaultsPlugin`: the stack of windows, their layers, and the requests made to windows.

use crate::support::{focus, test_app};
use bevy::{
    camera::NormalizedRenderTarget,
    picking::{
        backend::HitData,
        events::{Pointer, PointerDrag, PointerPress},
        pointer::{Location, PointerButton, PointerId},
    },
    prelude::*,
    ui::{ComputedNode, OverrideClip},
    ui_widgets::{Activate, popover::Popover},
};
use bevy_ui_component_prototypes::{
    ActiveWindow, CloseWindow, MoveWindow, RaiseWindow, ResizeWindow, WindowContent,
    WindowDefaultsPlugin, WindowEdge, WindowLimits, WindowRect, WindowStack, select, window,
};

mod close;
mod raise;
mod rect;
mod resize;
mod select;
mod stack;

pub(crate) fn app_with_defaults(plugin: WindowDefaultsPlugin) -> App {
    let mut app = test_app();
    app.add_plugins(plugin);
    app
}

pub(crate) fn spawn_window(app: &mut App) -> Entity {
    app.world_mut().spawn(window().build()).id()
}

pub(crate) fn layer(app: &App, window: Entity) -> Option<i32> {
    app.world().get::<GlobalZIndex>(window).map(|z| z.0)
}

pub(crate) fn is_active(app: &App, window: Entity) -> bool {
    app.world().get::<ActiveWindow>(window).is_some()
}

/// The pointer presses on `target`.
pub(crate) fn press(app: &mut App, target: Entity) {
    let location = Location {
        target: NormalizedRenderTarget::None {
            width: 1,
            height: 1,
        },
        position: Vec2::ZERO,
    };
    let camera = app.world_mut().spawn_empty().id();
    app.world_mut().trigger(PointerPress {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, location),
        button: PointerButton::Primary,
        hit: HitData::new(camera, 0.0, None, None),
        count: 1,
    });
    app.update();
}

pub(crate) fn content_of(app: &App, window: Entity) -> Entity {
    let children = app.world().get::<Children>(window).unwrap();
    children
        .iter()
        .find(|child| app.world().get::<WindowContent>(*child).is_some())
        .unwrap()
}

pub(crate) fn rect(app: &App, window: Entity) -> WindowRect {
    *app.world().get::<WindowRect>(window).unwrap()
}

pub(crate) fn title_bar(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(window).unwrap()[0]
}

pub(crate) fn close_button(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(title_bar(app, window)).unwrap()[0]
}

/// A window of 200 x 100 at (100, 100) in a node of 800 x 600.
pub(crate) fn window_in_area(app: &mut App) -> Entity {
    let area = app
        .world_mut()
        .spawn(ComputedNode {
            size: Vec2::new(800.0, 600.0),
            inverse_scale_factor: 1.0,
            ..default()
        })
        .id();
    let window = app
        .world_mut()
        .spawn((
            window()
                .rect(WindowRect::new(100.0, 100.0, 200.0, 100.0))
                .build(),
            WindowLimits {
                visible_width: 50.0,
                visible_height: 30.0,
                min_width: 80.0,
                min_height: 40.0,
            },
            ChildOf(area),
        ))
        .id();
    app.update();
    window
}

pub(crate) fn drag(app: &mut App, target: Entity, button: PointerButton, delta: Vec2) {
    let location = Location {
        target: NormalizedRenderTarget::None {
            width: 1,
            height: 1,
        },
        position: Vec2::ZERO,
    };
    app.world_mut().trigger(PointerDrag {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, location),
        button,
        distance: delta,
        delta,
    });
    app.update();
}
