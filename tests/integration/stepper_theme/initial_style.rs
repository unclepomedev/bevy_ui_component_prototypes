//! Tests of initial styling.

use super::{
    cells, children_of, color_of, header, mark, next_button, prev_button, spawn_default,
    spawn_stepper,
};
use crate::{
    probe::probe_theme,
    support::{border_color, focus, node, outline, test_app},
};
use bevy::{prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::{
    common::FocusRing,
    stepper::{StepperFill, StepperValue, stepper},
};

#[test]
fn the_cells_are_styled_after_one_update() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);

    app.update();

    // The probe theme lights only the current cell, and nothing here chooses otherwise.
    let style = probe_theme().stepper;
    let colors: Vec<Color> = cells(&app, stepper)
        .into_iter()
        .map(|cell| color_of(&app, cell))
        .collect();
    let upcoming = style.upcoming.normal;
    assert_eq!(
        colors,
        [upcoming, upcoming, style.current.normal, upcoming, upcoming]
    );
}

#[test]
fn up_to_lights_the_passed_cells_too() {
    let mut app = test_app();
    let stepper = spawn_stepper(
        &mut app,
        stepper().steps(5).value(2).fill(StepperFill::UpTo),
    );

    app.update();

    let style = probe_theme().stepper;
    let cells = cells(&app, stepper);
    assert_eq!(color_of(&app, cells[1]), style.passed.normal);
    assert_eq!(color_of(&app, cells[2]), style.current.normal);
    assert_eq!(color_of(&app, cells[3]), style.upcoming.normal);
}

#[test]
fn the_value_moves_the_current_cell_only_when_the_app_applies_it() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(stepper)
        .insert(StepperValue::new(4));
    app.update();

    let style = probe_theme().stepper;
    let cells = cells(&app, stepper);
    assert_eq!(color_of(&app, cells[2]), style.upcoming.normal);
    assert_eq!(color_of(&app, cells[4]), style.current.normal);
}

#[test]
fn a_value_past_the_end_shows_the_last_cell() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5).value(99));

    app.update();

    let cells = cells(&app, stepper);
    assert_eq!(
        color_of(&app, cells[4]),
        probe_theme().stepper.current.normal
    );
}

#[test]
fn only_the_marked_cells_show_a_mark() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5).marks([1, 3]));

    app.update();

    let shown: Vec<bool> = cells(&app, stepper)
        .into_iter()
        .map(|cell| color_of(&app, mark(&app, cell)) == probe_theme().stepper.mark.normal)
        .collect();
    assert_eq!(shown, [false, true, false, true, false]);
}

#[test]
fn the_buttons_and_their_arrows_take_the_theme_colors() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);

    app.update();

    let style = probe_theme().stepper;
    for button in [prev_button(&app, stepper), next_button(&app, stepper)] {
        assert_eq!(color_of(&app, button), style.button.normal);
        let arrow = children_of(&app, button)[0];
        let border = border_color(&app, arrow);
        assert_eq!(border, BorderColor::all(style.arrow.normal));
    }
}

#[test]
fn the_shape_comes_from_the_theme() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);

    app.update();

    let shape = probe_theme().stepper.shape;
    assert_eq!(node(&app, stepper).row_gap, Val::Px(shape.gap));
    assert_eq!(
        node(&app, cells(&app, stepper)[0]).height,
        Val::Px(shape.cell_height)
    );
    assert_eq!(
        node(&app, prev_button(&app, stepper)).width,
        Val::Px(shape.cell_height)
    );
    assert_eq!(
        node(&app, header(&app, stepper)[0]).border_radius,
        BorderRadius::all(Val::Px(shape.radius))
    );
}

#[test]
fn the_ring_shows_only_while_the_focus_indicator_is_visible() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);
    app.update();
    assert_eq!(ring_of(&app, stepper), FocusRing::NONE);

    focus(&mut app, stepper, true);
    app.update();
    assert_eq!(ring_of(&app, stepper), probe_theme().stepper.focus_ring);

    focus(&mut app, stepper, false);
    app.update();
    assert_eq!(ring_of(&app, stepper), FocusRing::NONE);
}

#[test]
fn a_disabled_stepper_shows_no_ring_even_when_it_has_the_focus() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);
    focus(&mut app, stepper, true);
    app.update();
    assert_eq!(ring_of(&app, stepper), probe_theme().stepper.focus_ring);

    app.world_mut()
        .entity_mut(stepper)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(ring_of(&app, stepper), FocusRing::NONE);
}

fn ring_of(app: &App, stepper: Entity) -> FocusRing {
    let outline = outline(app, stepper);
    FocusRing {
        color: outline.color,
        width: outline.width,
        offset: outline.offset,
    }
}
