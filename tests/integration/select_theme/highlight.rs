//! Tests of The highlight under the pointer.

use super::{list, menu_event, point_at, pointer_moves_over, rows, spawn_fruit, trigger};
use crate::support::test_app;
use bevy::{
    picking::hover::PickingInteraction,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{ActiveDescendant, MenuAction},
};

#[test]
fn the_row_the_pointer_moves_over_becomes_the_highlighted_one() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);

    pointer_moves_over(&mut app, rows[1]);
    assert_eq!(highlighted(&app, root), Some(rows[1]));

    // The pointer goes to another row, and the highlight goes with it.
    pointer_moves_over(&mut app, rows[2]);
    assert_eq!(highlighted(&app, root), Some(rows[2]));
}

#[test]
fn the_highlight_stays_where_it_was_when_the_pointer_leaves() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);
    pointer_moves_over(&mut app, rows[1]);

    point_at(&mut app, rows[1], PickingInteraction::None);

    assert_eq!(highlighted(&app, root), Some(rows[1]));
}

#[test]
fn a_row_that_comes_under_a_resting_pointer_does_not_take_the_highlight() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);
    pointer_moves_over(&mut app, rows[0]);

    point_at(&mut app, rows[1], PickingInteraction::Hovered);

    assert_eq!(highlighted(&app, root), Some(rows[0]));
}

#[test]
fn a_closed_list_does_not_follow_the_pointer() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let rows = rows(&app, root);

    pointer_moves_over(&mut app, rows[1]);

    assert_eq!(highlighted(&app, root), None);
}

#[test]
fn a_disabled_row_is_not_highlighted_by_the_pointer() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);
    app.world_mut()
        .entity_mut(rows[1])
        .insert(InteractionDisabled);

    pointer_moves_over(&mut app, rows[1]);

    assert_eq!(highlighted(&app, root), None);
}

fn highlighted(app: &App, root: Entity) -> Option<Entity> {
    app.world()
        .get::<ActiveDescendant>(list(app, root))
        .unwrap()
        .0
}
