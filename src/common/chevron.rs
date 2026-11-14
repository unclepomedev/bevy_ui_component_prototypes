use bevy::prelude::*;

const STROKE: f32 = 2.0;
// The visible half of the turned square sits to one side of its center, so it is moved back by
// this share of the side.
const NUDGE: f32 = 0.2;

/// Where a chevron points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Down,
    Left,
    Right,
}

impl Direction {
    /// The clockwise turn that takes the bottom right corner of a square to this side.
    fn degrees(self) -> f32 {
        match self {
            Self::Down => 45.0,
            Self::Left => 135.0,
            Self::Right => -45.0,
        }
    }

    /// The direction of the pointing side, with y down.
    fn unit(self) -> Vec2 {
        match self {
            Self::Down => Vec2::Y,
            Self::Left => Vec2::NEG_X,
            Self::Right => Vec2::X,
        }
    }
}

/// A chevron that needs no glyph: two edges of a square of `side` pixels, turned so that the corner
/// where they meet points `direction`. That corner reaches about 1.4 times `side` from the center.
///
/// The color is the border's, which the caller sets.
pub(crate) fn chevron(side: f32, direction: Direction) -> (Node, UiTransform) {
    let nudge = -direction.unit() * side * NUDGE;
    (
        Node {
            width: px(side),
            height: px(side),
            border: UiRect {
                right: px(STROKE),
                bottom: px(STROKE),
                ..default()
            },
            ..default()
        },
        UiTransform {
            translation: Val2::px(nudge.x, nudge.y),
            rotation: Rot2::degrees(direction.degrees()),
            ..default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_chevron_is_nudged_back_from_the_side_it_points_to() {
        let nudge = |direction| chevron(10.0, direction).1.translation;
        assert!(matches!(nudge(Direction::Left).x, Val::Px(x) if x > 0.0));
        assert!(matches!(nudge(Direction::Right).x, Val::Px(x) if x < 0.0));
        assert!(matches!(nudge(Direction::Down).y, Val::Px(y) if y < 0.0));
    }

    #[test]
    fn the_tips_point_to_their_own_sides() {
        // The tip is the bottom right corner of the square, where its two borders meet.
        // With y pointing down, a clockwise rotation by an angle moves (1, 1) to this.
        let tip = |direction: Direction| {
            let (sin, cos) = direction.degrees().to_radians().sin_cos();
            (cos - sin, sin + cos)
        };

        let (x, y) = tip(Direction::Left);
        assert!(x < -1.0 && y.abs() < 1e-5, "{x}, {y}");
        let (x, y) = tip(Direction::Right);
        assert!(x > 1.0 && y.abs() < 1e-5, "{x}, {y}");
        let (x, y) = tip(Direction::Down);
        assert!(y > 1.0 && x.abs() < 1e-5, "{x}, {y}");
    }
}
