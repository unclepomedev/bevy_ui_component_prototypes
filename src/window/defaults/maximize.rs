use super::rect::{Placed, PlacedItem, area_of};
use crate::window::{
    components::{Maximized, UiWindow, WindowRect},
    limits::WindowLimits,
    requests::ToggleMaximizeWindow,
};
use bevy::{prelude::*, ui::ComputedNode};

/// The size of the node, or the smallest size the window may have if that is larger.
pub(super) fn maximized_rect(limits: &WindowLimits, area: Vec2) -> WindowRect {
    WindowRect::new(
        0.0,
        0.0,
        area.x.max(limits.min_width),
        area.y.max(limits.min_height),
    )
}

/// Maximizes a window, or sets it back if it is maximized.
pub(super) fn on_toggle_maximize(
    toggle: On<ToggleMaximizeWindow>,
    mut windows: Query<(Placed, Option<&Maximized>), With<UiWindow>>,
    areas: Query<&ComputedNode>,
    mut commands: Commands,
) {
    let Ok((mut placed, maximized)) = windows.get_mut(toggle.window) else {
        return;
    };
    let area = area_of(placed.parent, &areas);
    if let Some(Maximized { restore }) = maximized {
        restore_rect(&mut placed, *restore, area);
        commands.entity(toggle.window).remove::<Maximized>();
        return;
    }
    // The node is not laid out yet: there is no size to give the window.
    let Some(area) = area else {
        return;
    };
    commands.entity(toggle.window).insert(Maximized {
        restore: *placed.rect,
    });
    placed.rect.set_if_neq(maximized_rect(placed.limits, area));
}

/// Sets `restore` back, moved into the node if that became smaller in the meantime.
fn restore_rect(placed: &mut PlacedItem, restore: WindowRect, area: Option<Vec2>) {
    let size = Vec2::new(restore.width, restore.height);
    let position = placed
        .limits
        .clamp(Vec2::new(restore.x, restore.y), size, area);
    placed.rect.set_if_neq(WindowRect {
        x: position.x,
        y: position.y,
        ..restore
    });
}

/// Keeps a maximized window as large as its node.
pub(super) fn fit_maximized(
    mut windows: Query<Placed, (With<UiWindow>, With<Maximized>)>,
    areas: Query<&ComputedNode>,
) {
    for mut placed in &mut windows {
        if let Some(area) = area_of(placed.parent, &areas) {
            placed.rect.set_if_neq(maximized_rect(placed.limits, area));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_maximized_window_fills_the_area() {
        let rect = maximized_rect(&WindowLimits::default(), Vec2::new(800.0, 600.0));
        assert_eq!(rect, WindowRect::new(0.0, 0.0, 800.0, 600.0));
    }

    #[test]
    fn a_maximized_window_is_not_smaller_than_its_minimum_size() {
        let limits = WindowLimits {
            min_width: 900.0,
            min_height: 700.0,
            ..WindowLimits::default()
        };
        let rect = maximized_rect(&limits, Vec2::new(800.0, 600.0));
        assert_eq!(rect, WindowRect::new(0.0, 0.0, 900.0, 700.0));
    }
}
