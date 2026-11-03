use bevy::prelude::*;

/// How far a window may be moved out of the node it is placed in.
///
/// Horizontally, at least `visible_width` of the window stays inside. Vertically, the top edge of the window
/// is not above the top edge of the node, and is at least `visible_height` above its bottom edge.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct WindowLimits {
    pub visible_width: f32,
    pub visible_height: f32,
}

impl Default for WindowLimits {
    fn default() -> Self {
        Self {
            visible_width: 64.0,
            visible_height: 32.0,
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
}
