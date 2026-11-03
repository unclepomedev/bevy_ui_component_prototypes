use super::{components::WindowRect, requests::WindowEdge};
use bevy::prelude::*;

/// How far a window may be moved out of the node it is placed in, and how small it may be made.
///
/// Horizontally, at least `visible_width` of the window stays inside. Vertically, the top edge of the window
/// is not above the top edge of the node, and is at least `visible_height` above its bottom edge.
///
/// Resizing does not make the window smaller than `min_width` by `min_height` (a window that is smaller
/// already does not get smaller), and does not move an edge further out of the node.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct WindowLimits {
    pub visible_width: f32,
    pub visible_height: f32,
    pub min_width: f32,
    pub min_height: f32,
}

impl Default for WindowLimits {
    fn default() -> Self {
        Self {
            visible_width: 64.0,
            visible_height: 32.0,
            min_width: 120.0,
            min_height: 60.0,
        }
    }
}

impl WindowLimits {
    /// `position` of a window of `size`, moved into the limits in a node of `area`.
    /// An `area` that is empty (the node has not been laid out yet) puts no limit.
    pub(crate) fn clamp(&self, position: Vec2, size: Vec2, area: Vec2) -> Vec2 {
        if area.x <= 0.0 || area.y <= 0.0 {
            return position;
        }
        let visible = self.visible_width.min(size.x);
        let min_x = visible - size.x;
        let max_x = (area.x - visible).max(min_x);
        let max_y = (area.y - self.visible_height).max(0.0);
        Vec2::new(position.x.clamp(min_x, max_x), position.y.clamp(0.0, max_y))
    }

    /// `rect` with `edge` moved by `delta`, in a node of `area`.
    /// An `area` that is empty (the node has not been laid out yet) does not hold the edges in.
    pub(crate) fn resize(
        &self,
        rect: WindowRect,
        edge: WindowEdge,
        delta: Vec2,
        area: Vec2,
    ) -> WindowRect {
        let known = area.x > 0.0 && area.y > 0.0;
        let (old_left, old_top) = (rect.x, rect.y);
        let (old_right, old_bottom) = (rect.x + rect.width, rect.y + rect.height);
        let (mut left, mut top, mut right, mut bottom) = (old_left, old_top, old_right, old_bottom);

        if edge.has_left() {
            left += delta.x;
            if known {
                left = left.max(old_left.min(0.0));
            }
        }
        if edge.has_right() {
            right += delta.x;
            if known {
                right = right.min(old_right.max(area.x));
            }
        }
        if edge.has_top() {
            top += delta.y;
            if known {
                top = top.max(old_top.min(0.0));
            }
        }
        if edge.has_bottom() {
            bottom += delta.y;
            if known {
                bottom = bottom.min(old_bottom.max(area.y));
            }
        }

        let min_width = self.min_width.min(rect.width);
        let min_height = self.min_height.min(rect.height);
        if edge.has_left() {
            left = left.min(right - min_width);
        }
        if edge.has_right() {
            right = right.max(left + min_width);
        }
        if edge.has_top() {
            top = top.min(bottom - min_height);
        }
        if edge.has_bottom() {
            bottom = bottom.max(top + min_height);
        }
        WindowRect::new(left, top, right - left, bottom - top)
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: WindowLimits = WindowLimits {
        visible_width: 50.0,
        visible_height: 30.0,
        min_width: 80.0,
        min_height: 40.0,
    };
    const SIZE: Vec2 = Vec2::new(200.0, 100.0);
    const AREA: Vec2 = Vec2::new(800.0, 600.0);

    fn clamp(x: f32, y: f32) -> Vec2 {
        LIMITS.clamp(Vec2::new(x, y), SIZE, AREA)
    }

    #[test]
    fn a_position_inside_the_limits_is_kept() {
        assert_eq!(clamp(100.0, 200.0), Vec2::new(100.0, 200.0));
    }

    #[test]
    fn the_top_edge_cannot_go_above_the_node() {
        assert_eq!(clamp(100.0, -40.0).y, 0.0);
    }

    #[test]
    fn the_top_edge_stays_above_the_bottom_by_the_visible_height() {
        assert_eq!(clamp(100.0, 900.0).y, 570.0);
    }

    #[test]
    fn the_window_can_leave_at_the_left_up_to_the_visible_width() {
        assert_eq!(clamp(-500.0, 0.0).x, -150.0);
    }

    #[test]
    fn the_window_can_leave_at_the_right_up_to_the_visible_width() {
        assert_eq!(clamp(900.0, 0.0).x, 750.0);
    }

    #[test]
    fn a_window_narrower_than_the_visible_width_stays_whole() {
        let narrow = Vec2::new(30.0, 100.0);
        assert_eq!(LIMITS.clamp(Vec2::new(-500.0, 0.0), narrow, AREA).x, 0.0);
        assert_eq!(LIMITS.clamp(Vec2::new(900.0, 0.0), narrow, AREA).x, 770.0);
    }

    #[test]
    fn an_empty_area_puts_no_limit() {
        let position = Vec2::new(-999.0, -999.0);
        assert_eq!(LIMITS.clamp(position, SIZE, Vec2::ZERO), position);
    }

    #[test]
    fn a_node_smaller_than_the_limits_does_not_panic() {
        let tiny = Vec2::new(10.0, 10.0);
        let _ = LIMITS.clamp(Vec2::new(5.0, 5.0), SIZE, tiny);
    }

    // The window of the tests below: from (100, 100) to (300, 200) in the node of 800 x 600.
    const RECT: WindowRect = WindowRect::new(100.0, 100.0, 200.0, 100.0);

    fn resized(edge: WindowEdge, dx: f32, dy: f32) -> WindowRect {
        LIMITS.resize(RECT, edge, Vec2::new(dx, dy), AREA)
    }

    #[test]
    fn the_right_and_the_bottom_edge_change_only_the_size() {
        assert_eq!(
            resized(WindowEdge::Right, 30.0, 99.0),
            WindowRect::new(100.0, 100.0, 230.0, 100.0)
        );
        assert_eq!(
            resized(WindowEdge::Bottom, 99.0, 20.0),
            WindowRect::new(100.0, 100.0, 200.0, 120.0)
        );
    }

    #[test]
    fn the_left_and_the_top_edge_change_the_position_and_the_size() {
        assert_eq!(
            resized(WindowEdge::Left, -30.0, 99.0),
            WindowRect::new(70.0, 100.0, 230.0, 100.0)
        );
        assert_eq!(
            resized(WindowEdge::Top, 99.0, 20.0),
            WindowRect::new(100.0, 120.0, 200.0, 80.0)
        );
    }

    #[test]
    fn a_corner_moves_its_two_edges() {
        assert_eq!(
            resized(WindowEdge::TopLeft, -10.0, -20.0),
            WindowRect::new(90.0, 80.0, 210.0, 120.0)
        );
        assert_eq!(
            resized(WindowEdge::BottomRight, 10.0, 20.0),
            WindowRect::new(100.0, 100.0, 210.0, 120.0)
        );
    }

    #[test]
    fn the_window_stops_at_the_minimum_size_and_the_far_edge_stays() {
        assert_eq!(
            resized(WindowEdge::Left, 500.0, 0.0),
            WindowRect::new(220.0, 100.0, 80.0, 100.0)
        );
        assert_eq!(
            resized(WindowEdge::Top, 0.0, 500.0),
            WindowRect::new(100.0, 160.0, 200.0, 40.0)
        );
        assert_eq!(
            resized(WindowEdge::Right, -500.0, 0.0),
            WindowRect::new(100.0, 100.0, 80.0, 100.0)
        );
        assert_eq!(
            resized(WindowEdge::BottomLeft, 500.0, -500.0),
            WindowRect::new(220.0, 100.0, 80.0, 40.0)
        );
    }

    #[test]
    fn an_edge_does_not_go_out_of_the_node() {
        assert_eq!(resized(WindowEdge::Right, 9000.0, 0.0).width, 700.0);
        assert_eq!(resized(WindowEdge::Bottom, 0.0, 9000.0).height, 500.0);
        assert_eq!(
            resized(WindowEdge::TopLeft, -9000.0, -9000.0),
            WindowRect::new(0.0, 0.0, 300.0, 200.0)
        );
    }

    #[test]
    fn an_edge_that_is_out_of_the_node_already_may_come_back_but_not_go_further() {
        let out = WindowRect::new(700.0, 100.0, 200.0, 100.0);
        let right = |dx| LIMITS.resize(out, WindowEdge::Right, Vec2::new(dx, 0.0), AREA);
        assert_eq!(right(50.0).width, 200.0);
        assert_eq!(right(-50.0).width, 150.0);
    }

    #[test]
    fn a_window_smaller_than_the_minimum_does_not_grow_when_an_edge_is_touched() {
        let small = WindowRect::new(100.0, 100.0, 50.0, 20.0);
        let moved = LIMITS.resize(small, WindowEdge::Left, Vec2::new(0.0, 0.0), AREA);
        assert_eq!(moved, small);
        let shrunk = LIMITS.resize(small, WindowEdge::Left, Vec2::new(10.0, 0.0), AREA);
        assert_eq!(shrunk, small);
    }

    #[test]
    fn an_empty_area_does_not_hold_the_edges_in() {
        let moved = LIMITS.resize(
            RECT,
            WindowEdge::TopLeft,
            Vec2::new(-500.0, -500.0),
            Vec2::ZERO,
        );
        assert_eq!(moved, WindowRect::new(-400.0, -400.0, 700.0, 600.0));
    }
}
