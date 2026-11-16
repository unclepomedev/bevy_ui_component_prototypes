use crate::window::{
    components::{Maximized, UiWindow, WindowRect},
    limits::WindowLimits,
    requests::{MoveWindow, ResizeWindow},
};
use bevy::{ecs::query::QueryData, prelude::*, ui::ComputedNode};

/// Where a window is, how far it may go, and the node it is in.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct Placed {
    pub(super) rect: &'static mut WindowRect,
    pub(super) limits: &'static WindowLimits,
    pub(super) parent: Option<&'static ChildOf>,
}

pub(super) fn on_move_window(
    mv: On<MoveWindow>,
    mut windows: Query<Placed, (With<UiWindow>, Without<Maximized>)>,
    areas: Query<&ComputedNode>,
) {
    let Ok(mut window) = windows.get_mut(mv.window) else {
        return;
    };
    let rect = *window.rect;
    let area = area_of(window.parent, &areas);
    let target = Vec2::new(rect.x, rect.y) + mv.delta;
    let position = window
        .limits
        .clamp(target, Vec2::new(rect.width, rect.height), area);
    window.rect.set_if_neq(WindowRect {
        x: position.x,
        y: position.y,
        ..rect
    });
}

pub(super) fn on_resize_window(
    resize: On<ResizeWindow>,
    mut windows: Query<Placed, (With<UiWindow>, Without<Maximized>)>,
    areas: Query<&ComputedNode>,
) {
    let Ok(mut window) = windows.get_mut(resize.window) else {
        return;
    };
    let area = area_of(window.parent, &areas);
    let resized = window
        .limits
        .resize(*window.rect, resize.edge, resize.delta, area);
    window.rect.set_if_neq(resized);
}

/// The size of the node a window is placed in, in logical pixels; `None` if the window has no
/// parent or the parent is not laid out yet.
pub(super) fn area_of(parent: Option<&ChildOf>, areas: &Query<&ComputedNode>) -> Option<Vec2> {
    let node = areas.get(parent?.parent()).ok()?;
    let size = node.size() * node.inverse_scale_factor();
    (size.x > 0.0 && size.y > 0.0).then_some(size)
}
