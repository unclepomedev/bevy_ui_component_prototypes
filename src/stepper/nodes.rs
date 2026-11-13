use super::{components::StepperButton, style::StepperShape};
use bevy::prelude::*;

// How far a mark sits below its cell.
const MARK_GAP: f32 = 3.0;
// The share of the cell height taken by the side of the square an arrow is cut from.
// The arrow is that square's corner turned 45 degrees, so it reaches about 1.4 times the side.
const ARROW_SHARE: f32 = 0.3;
const ARROW_STROKE: f32 = 2.0;
// The share of the cell height taken by a replacement icon on a button.
const ICON_SHARE: f32 = 0.6;
// The visible half of the turned square sits to one side of its center, so it is moved back by this share of the side.
const ARROW_NUDGE: f32 = 0.2;

pub(crate) fn stepper_node(shape: &StepperShape, inline: bool) -> Node {
    if inline {
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            // The buttons stay level with the cells, not with the room reserved under them for the marks.
            align_items: AlignItems::FlexStart,
            column_gap: px(shape.gap),
            ..default()
        }
    } else {
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(shape.gap),
            ..default()
        }
    }
}

/// Writes the fields of the root that the layout decides and leaves the rest to the app.
pub(crate) fn lay_out(node: &mut Mut<Node>, shape: &StepperShape, inline: bool) {
    let wanted = stepper_node(shape, inline);
    let mut next = node.clone();
    next.flex_direction = wanted.flex_direction;
    next.align_items = wanted.align_items;
    next.row_gap = wanted.row_gap;
    next.column_gap = wanted.column_gap;
    node.set_if_neq(next);
}

pub(crate) fn header_node(shape: &StepperShape) -> Node {
    Node {
        width: percent(100),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(shape.gap),
        ..default()
    }
}

pub(crate) fn track_node(shape: &StepperShape, inline: bool) -> Node {
    let width = if inline {
        // The buttons keep their size and the cells take the rest. The zero minimum lets the cells narrow below their content.
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            ..default()
        }
    } else {
        Node {
            width: percent(100),
            ..default()
        }
    };
    Node {
        column_gap: px(shape.gap),
        // The marks hang below the cells, so the room for them is reserved here.
        padding: UiRect::bottom(px(shape.mark_size + MARK_GAP)),
        ..width
    }
}

pub(crate) fn cell_node(shape: &StepperShape) -> Node {
    // A zero basis makes the cells equally wide whatever their number. They have no minimum width:
    // below the one the theme asks for they only warn.
    Node {
        flex_grow: 1.0,
        flex_basis: px(0),
        height: px(shape.cell_height),
        border: UiRect::all(px(shape.border_width)),
        border_radius: BorderRadius::all(px(shape.radius)),
        ..default()
    }
}

pub(crate) fn mark_node(shape: &StepperShape) -> Node {
    let size = shape.mark_size;
    Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        margin: UiRect::left(px(-size / 2.0)),
        bottom: px(-(size + MARK_GAP)),
        width: px(size),
        height: px(size),
        border_radius: BorderRadius::all(px(size / 2.0)),
        ..default()
    }
}

pub(crate) fn button_node(shape: &StepperShape) -> Node {
    Node {
        width: px(shape.cell_height),
        height: px(shape.cell_height),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(shape.radius)),
        ..default()
    }
}

pub(crate) fn label_node() -> Node {
    Node {
        flex_grow: 1.0,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

pub(crate) fn icon_node(shape: &StepperShape) -> Node {
    let side = shape.cell_height * ICON_SHARE;
    Node {
        width: px(side),
        height: px(side),
        ..default()
    }
}

/// The arrow of a button: two edges of a square, turned to point left or right.
pub(crate) fn arrow_glyph(shape: &StepperShape, button: StepperButton) -> (Node, UiTransform) {
    let side = shape.cell_height * ARROW_SHARE;
    let nudge = -button.delta() as f32 * side * ARROW_NUDGE;
    (
        Node {
            width: px(side),
            height: px(side),
            border: UiRect {
                right: px(ARROW_STROKE),
                bottom: px(ARROW_STROKE),
                ..default()
            },
            ..default()
        },
        UiTransform {
            translation: Val2::px(nudge, 0.0),
            rotation: Rot2::degrees(arrow_degrees(button)),
            ..default()
        },
    )
}

// The corner of the square that points to the left or to the right. Rotation is clockwise.
fn arrow_degrees(button: StepperButton) -> f32 {
    match button {
        StepperButton::Prev => 135.0,
        StepperButton::Next => -45.0,
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arrows_are_nudged_toward_each_other() {
        let shape = StepperShape::default();
        let (_, prev) = arrow_glyph(&shape, StepperButton::Prev);
        let (_, next) = arrow_glyph(&shape, StepperButton::Next);
        // The visible half of the arrow lies on its pointing side, so each is moved back from it.
        assert!(matches!(prev.translation.x, Val::Px(x) if x > 0.0));
        assert!(matches!(next.translation.x, Val::Px(x) if x < 0.0));
    }

    #[test]
    fn the_arrow_tips_point_to_their_own_sides() {
        // The tip is the bottom right corner of the square, where its two borders meet.
        // With y pointing down, a clockwise rotation by an angle moves (1, 1) to this.
        let tip = |button| {
            let (sin, cos) = arrow_degrees(button).to_radians().sin_cos();
            (cos - sin, sin + cos)
        };

        let (x, y) = tip(StepperButton::Prev);
        assert!(x < -1.0 && y.abs() < 1e-5, "{x}, {y}");
        let (x, y) = tip(StepperButton::Next);
        assert!(x > 1.0 && y.abs() < 1e-5, "{x}, {y}");
    }

    #[test]
    fn the_track_reserves_the_room_the_marks_hang_in() {
        let shape = StepperShape::default();
        let track = track_node(&shape, false);
        let mark = mark_node(&shape);
        // The mark's far edge below its cell is where the reserved room ends.
        assert_eq!(mark.bottom, px(-track_room(&track)));
    }

    #[test]
    fn an_inline_stepper_is_a_row_and_its_track_takes_the_room_the_buttons_leave() {
        let shape = StepperShape::default();
        let root = stepper_node(&shape, true);
        assert_eq!(root.flex_direction, FlexDirection::Row);
        assert_eq!(root.align_items, AlignItems::FlexStart);
        assert_eq!(root.column_gap, px(shape.gap));
        let track = track_node(&shape, true);
        assert_eq!(track.flex_grow, 1.0);
        assert_eq!(track.width, Val::Auto);

        let stacked = stepper_node(&shape, false);
        assert_eq!(stacked.flex_direction, FlexDirection::Column);
        assert_eq!(track_node(&shape, false).width, percent(100));
    }

    #[test]
    fn a_cell_has_the_border_the_shape_asks_for() {
        let mut shape = StepperShape::default();
        assert_eq!(cell_node(&shape).border, UiRect::all(px(0)));
        shape.border_width = 1.5;
        assert_eq!(cell_node(&shape).border, UiRect::all(px(1.5)));
    }

    #[test]
    fn cells_share_the_width_equally() {
        let cell = cell_node(&StepperShape::default());
        assert_eq!(cell.flex_grow, 1.0);
        assert_eq!(cell.flex_basis, px(0));
    }

    fn track_room(track: &Node) -> f32 {
        match track.padding.bottom {
            Val::Px(room) => room,
            other => panic!("not in pixels: {other:?}"),
        }
    }
}
