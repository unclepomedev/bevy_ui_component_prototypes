use crate::window::{
    components::{UiWindow, WindowRect},
    limits::WindowLimits,
    requests::{MoveWindow, ResizeWindow},
};
use bevy::{prelude::*, ui::ComputedNode};

pub(super) fn on_move_window(
    mv: On<MoveWindow>,
    mut windows: Query<(&mut WindowRect, &WindowLimits, Option<&ChildOf>), With<UiWindow>>,
    areas: Query<&ComputedNode>,
) {
    let Ok((mut rect, limits, parent)) = windows.get_mut(mv.window) else {
        return;
    };
    let area = area_of(parent, &areas);
    let target = Vec2::new(rect.x, rect.y) + mv.delta;
    let position = limits.clamp(target, Vec2::new(rect.width, rect.height), area);
    let moved = WindowRect {
        x: position.x,
        y: position.y,
        ..*rect
    };
    rect.set_if_neq(moved);
}

pub(super) fn on_resize_window(
    resize: On<ResizeWindow>,
    mut windows: Query<(&mut WindowRect, &WindowLimits, Option<&ChildOf>), With<UiWindow>>,
    areas: Query<&ComputedNode>,
) {
    let Ok((mut rect, limits, parent)) = windows.get_mut(resize.window) else {
        return;
    };
    let area = area_of(parent, &areas);
    let resized = limits.resize(*rect, resize.edge, resize.delta, area);
    rect.set_if_neq(resized);
}

/// The size of the node a window is placed in, in logical pixels; zero if it is not laid out yet.
fn area_of(parent: Option<&ChildOf>, areas: &Query<&ComputedNode>) -> Vec2 {
    parent
        .and_then(|parent| areas.get(parent.parent()).ok())
        .map_or(Vec2::ZERO, |node| node.size() * node.inverse_scale_factor())
}
