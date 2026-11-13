//! Tests of the inline layout and of the border of a cell.

use super::{children_of, color_of, spawn_stepper};
use crate::{
    probe::probe_theme,
    support::{node, red, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{GlobalTheme, common::StateColors, stepper::stepper};

#[test]
fn an_inline_stepper_is_a_row_with_the_theme_gap() {
    let mut app = test_app();
    let root = spawn_stepper(&mut app, stepper().steps(5).inline());
    app.update();

    let root_node = node(&app, root);
    assert_eq!(root_node.flex_direction, FlexDirection::Row);
    assert_eq!(
        root_node.column_gap,
        Val::Px(probe_theme().stepper.shape.gap)
    );
    let parts = children_of(&app, root);
    assert_eq!(parts.len(), 3);
    assert_eq!(node(&app, parts[1]).flex_grow, 1.0);
}

#[test]
fn a_theme_change_keeps_an_inline_stepper_inline_and_follows_the_new_gap() {
    let mut app = test_app();
    let root = spawn_stepper(&mut app, stepper().steps(5).inline());
    app.update();

    let mut theme = probe_theme();
    theme.stepper.shape.gap = 12.0;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    let root_node = node(&app, root);
    assert_eq!(root_node.flex_direction, FlexDirection::Row);
    assert_eq!(root_node.column_gap, Val::Px(12.0));
}

#[test]
fn the_layout_leaves_the_rest_of_the_node_to_the_app() {
    let mut app = test_app();
    let root = spawn_stepper(&mut app, stepper().steps(5).inline());
    app.update();

    app.world_mut().get_mut::<Node>(root).unwrap().margin = UiRect::all(Val::Px(5.0));
    let mut theme = probe_theme();
    theme.stepper.shape.gap = 12.0;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    assert_eq!(node(&app, root).margin, UiRect::all(Val::Px(5.0)));
}

#[test]
fn a_cell_has_the_border_of_the_theme() {
    let mut app = test_app();
    let root = spawn_stepper(&mut app, stepper().steps(5).value(2));
    app.update();

    let cell = children_of(&app, children_of(&app, root)[1])[0];
    let style = probe_theme().stepper;
    assert_eq!(
        node(&app, cell).border,
        UiRect::all(Val::Px(style.shape.border_width))
    );
    assert_eq!(border_of(&app, cell), style.cell_border.normal);

    let mut theme = probe_theme();
    theme.stepper.cell_border = StateColors::uniform(red());
    theme.stepper.shape.border_width = 0.0;
    app.insert_resource(GlobalTheme(theme));
    app.update();
    assert_eq!(border_of(&app, cell), red());
    assert_eq!(node(&app, cell).border, UiRect::all(Val::Px(0.0)));
    // The fill is not the border.
    assert_ne!(color_of(&app, cell), red());
}

fn border_of(app: &App, entity: Entity) -> Color {
    app.world().get::<BorderColor>(entity).unwrap().top
}
