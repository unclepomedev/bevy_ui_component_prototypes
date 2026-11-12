//! Tests of changing a stepper's settings after it is spawned.

use super::{cells, color_of, is_disabled, prev_button, spawn_default, spawn_stepper};
use crate::{probe::probe_theme, support::test_app};
use bevy_ui_component_prototypes::stepper::{Stepper, StepperFill, stepper};

#[test]
fn turning_wrap_on_and_off_enables_and_disables_the_button_at_an_end() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5));
    app.update();
    let prev = prev_button(&app, stepper);
    assert!(is_disabled(&app, prev));

    app.world_mut()
        .get_mut::<Stepper>(stepper)
        .unwrap()
        .set_wrap(true);
    app.update();
    assert!(!is_disabled(&app, prev));

    app.world_mut()
        .get_mut::<Stepper>(stepper)
        .unwrap()
        .set_wrap(false);
    app.update();
    assert!(is_disabled(&app, prev));
}

#[test]
fn changing_the_fill_relights_the_passed_cells() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);
    app.update();
    let style = probe_theme().stepper;
    let passed = cells(&app, stepper)[1];
    // The probe theme lights only the current cell, and the stepper has no fill of its own yet.
    assert_eq!(color_of(&app, passed), style.upcoming.normal);

    app.world_mut()
        .get_mut::<Stepper>(stepper)
        .unwrap()
        .set_fill(Some(StepperFill::UpTo));
    app.update();
    assert_eq!(color_of(&app, passed), style.passed.normal);

    app.world_mut()
        .get_mut::<Stepper>(stepper)
        .unwrap()
        .set_fill(None);
    app.update();
    assert_eq!(color_of(&app, passed), style.upcoming.normal);
}
