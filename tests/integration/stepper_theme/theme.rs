//! Tests of theme resolution.

use super::{cells, color_of, spawn_default};
use crate::{
    probe::probe_theme,
    support::{node, red, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{GlobalTheme, Theme, common::StateColors, stepper::stepper};

#[test]
fn global_theme_change_updates_unscoped_steppers_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_default(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_theme()));
    app.update();

    assert_eq!(current(&app, unscoped), red());
    assert_eq!(height(&app, unscoped), Val::Px(50.0));
    assert_eq!(current(&app, scoped), probe_theme().stepper.current.normal);
    assert_eq!(
        height(&app, scoped),
        Val::Px(probe_theme().stepper.shape.cell_height)
    );
}

#[test]
fn scoped_theme_change_updates_only_steppers_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_default(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_theme();
    app.update();

    assert_eq!(current(&app, inside), red());
    assert_eq!(height(&app, inside), Val::Px(50.0));
    assert_eq!(current(&app, outside), probe_theme().stepper.current.normal);
}

#[test]
fn a_stepper_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_theme()).id();
    app.update();

    let late = spawn_under(&mut app, panel);
    app.update();

    assert_eq!(current(&app, late), red());
}

fn spawn_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((stepper().steps(5).value(2).build(), ChildOf(parent)))
        .id()
}

/// The color of the current cell, which these tests put at 2.
fn current(app: &App, stepper: Entity) -> Color {
    color_of(app, cells(app, stepper)[2])
}

fn height(app: &App, stepper: Entity) -> Val {
    node(app, cells(app, stepper)[2]).height
}

/// The probe theme with a red current cell and taller cells.
fn red_theme() -> Theme {
    let mut theme = probe_theme();
    theme.stepper.current = StateColors::uniform(red());
    theme.stepper.shape.cell_height = 50.0;
    theme
}
