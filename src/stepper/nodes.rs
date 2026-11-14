use super::{components::StepperButton, style::StepperShape};
use crate::common::{
    chevron::{Direction, chevron},
    layout::write_changed,
};
use bevy::prelude::*;

const MARK_GAP: f32 = 3.0;

/// How far the marks hang below the cells, which the track reserves.
fn mark_room(shape: &StepperShape) -> f32 {
    shape.mark_size + MARK_GAP
}

// The share of the cell height taken by the side of the square an arrow is cut from.
const ARROW_SHARE: f32 = 0.3;
const ICON_SHARE: f32 = 0.6;

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
    write_changed!(
        node,
        wanted,
        flex_direction,
        align_items,
        row_gap,
        column_gap
    );
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
        padding: UiRect::bottom(px(mark_room(shape))),
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
        bottom: px(-mark_room(shape)),
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

/// The arrow of a button: a chevron pointing left or right.
pub(crate) fn arrow_glyph(shape: &StepperShape, button: StepperButton) -> (Node, UiTransform) {
    let direction = match button {
        StepperButton::Prev => Direction::Left,
        StepperButton::Next => Direction::Right,
    };
    chevron(shape.cell_height * ARROW_SHARE, direction)
}

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
}
