//! Tests of which parts are disabled, and when.

use super::{cells, color_of, is_disabled, next_button, prev_button, spawn_stepper};
use crate::{probe::probe_theme, support::test_app};
use bevy::{prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::stepper::{StepperValue, stepper};

/// Whether the previous and the next button are disabled.
fn buttons(app: &App, stepper: Entity) -> (bool, bool) {
    (
        is_disabled(app, prev_button(app, stepper)),
        is_disabled(app, next_button(app, stepper)),
    )
}

#[test]
fn at_the_first_position_only_the_previous_button_is_disabled() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5));
    app.update();
    assert_eq!(buttons(&app, stepper), (true, false));
}

#[test]
fn at_the_last_position_only_the_next_button_is_disabled() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5).value(4));
    app.update();
    assert_eq!(buttons(&app, stepper), (false, true));
}

#[test]
fn in_between_both_buttons_work() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5).value(2));
    app.update();
    assert_eq!(buttons(&app, stepper), (false, false));
}

#[test]
fn the_buttons_follow_the_value() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5));
    app.update();

    app.world_mut()
        .entity_mut(stepper)
        .insert(StepperValue::new(4));
    app.update();
    assert_eq!(buttons(&app, stepper), (false, true));

    app.world_mut()
        .entity_mut(stepper)
        .insert(StepperValue::new(2));
    app.update();
    assert_eq!(buttons(&app, stepper), (false, false));
}

#[test]
fn a_value_past_the_end_counts_as_the_last_position() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5).value(99));
    app.update();
    assert_eq!(buttons(&app, stepper), (false, true));
}

#[test]
fn with_wrap_the_ends_do_not_disable_the_buttons() {
    let mut app = test_app();
    let first = spawn_stepper(&mut app, stepper().steps(5).wrap(true));
    let last = spawn_stepper(&mut app, stepper().steps(5).value(4).wrap(true));
    app.update();
    assert_eq!(buttons(&app, first), (false, false));
    assert_eq!(buttons(&app, last), (false, false));
}

#[test]
fn a_single_position_disables_both_buttons_even_with_wrap() {
    let mut app = test_app();
    let plain = spawn_stepper(&mut app, stepper().steps(1));
    let wrapping = spawn_stepper(&mut app, stepper().steps(1).wrap(true));
    app.update();
    assert_eq!(buttons(&app, plain), (true, true));
    assert_eq!(buttons(&app, wrapping), (true, true));
}

#[test]
fn a_disabled_button_is_painted_disabled() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5));
    app.update();

    let style = probe_theme().stepper;
    assert_eq!(
        color_of(&app, prev_button(&app, stepper)),
        style.button.disabled
    );
    assert_eq!(
        color_of(&app, next_button(&app, stepper)),
        style.button.normal
    );
}

#[test]
fn a_disabled_stepper_disables_its_cells_and_buttons() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5).value(2));
    app.update();

    app.world_mut()
        .entity_mut(stepper)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(buttons(&app, stepper), (true, true));
    assert!(
        cells(&app, stepper)
            .into_iter()
            .all(|c| is_disabled(&app, c))
    );
    let style = probe_theme().stepper;
    assert_eq!(
        color_of(&app, cells(&app, stepper)[2]),
        style.current.disabled
    );
}

#[test]
fn enabling_the_stepper_restores_the_parts() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5));
    app.world_mut()
        .entity_mut(stepper)
        .insert(InteractionDisabled);
    app.update();

    app.world_mut()
        .entity_mut(stepper)
        .remove::<InteractionDisabled>();
    app.update();

    // The previous button stays disabled: this is the first position.
    assert_eq!(buttons(&app, stepper), (true, false));
    assert!(
        !cells(&app, stepper)
            .into_iter()
            .any(|c| is_disabled(&app, c))
    );
}
