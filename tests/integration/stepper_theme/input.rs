//! Tests of the requests a stepper makes, and of the keys it takes.

use super::{cells, color_of, next_button, prev_button, spawn_stepper};
use crate::{probe::probe_theme, support::test_app};
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyCode, KeyboardInput},
    },
    input_focus::FocusedInput,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{Activate, ValueChange},
};
use bevy_ui_component_prototypes::stepper::{
    StepperBuilder, StepperValue, stepper, stepper_self_update,
};

#[derive(Resource, Default)]
struct Requests {
    sent: Vec<(Entity, usize, bool)>,
}

/// How many times a key press got past the stepper to the window behind it.
#[derive(Resource, Default)]
struct Passed {
    count: usize,
}

struct Rig {
    app: App,
    window: Entity,
}

fn rig() -> Rig {
    let mut app = test_app();
    app.init_resource::<Requests>()
        .init_resource::<Passed>()
        .add_observer(
            |change: On<ValueChange<usize>>, mut log: ResMut<Requests>| {
                log.sent
                    .push((change.source, change.value, change.is_final));
            },
        );
    // A real window: the event reaches it only if `Window` is registered, and stops there only if it has one.
    let window = app.world_mut().spawn(Window::default()).id();
    app.world_mut().entity_mut(window).observe(
        |_: On<FocusedInput<KeyboardInput>>, mut passed: ResMut<Passed>| passed.count += 1,
    );
    Rig { app, window }
}

impl Rig {
    fn spawn(&mut self, builder: StepperBuilder) -> Entity {
        let stepper = spawn_stepper(&mut self.app, builder);
        self.app.update();
        stepper
    }

    fn activate(&mut self, button: Entity) {
        self.app.world_mut().trigger(Activate { entity: button });
        self.app.update();
    }

    fn press(&mut self, stepper: Entity, key_code: KeyCode, logical_key: Key) {
        let key = KeyboardInput {
            key_code,
            logical_key,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: self.window,
        };
        self.app
            .world_mut()
            .trigger(FocusedInput::new(stepper, key, self.window));
        self.app.update();
    }

    fn requests(&self) -> Vec<(Entity, usize, bool)> {
        self.app.world().resource::<Requests>().sent.clone()
    }

    fn passed(&self) -> usize {
        self.app.world().resource::<Passed>().count
    }
}

#[test]
fn pressing_a_cell_requests_its_position() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(5).value(1));
    let cell = cells(&rig.app, stepper)[3];

    rig.activate(cell);

    assert_eq!(rig.requests(), vec![(stepper, 3, true)]);
}

#[test]
fn pressing_the_current_cell_requests_nothing() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(5).value(1));

    let cell = cells(&rig.app, stepper)[1];
    rig.activate(cell);

    assert!(rig.requests().is_empty());
}

#[test]
fn the_buttons_request_one_position_away() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(5).value(2));

    let (prev, next) = (
        prev_button(&rig.app, stepper),
        next_button(&rig.app, stepper),
    );
    rig.activate(next);
    rig.activate(prev);

    assert_eq!(rig.requests(), vec![(stepper, 3, true), (stepper, 1, true)]);
}

#[test]
fn a_button_at_an_end_requests_nothing_without_wrap_and_wraps_with_it() {
    let mut rig = rig();
    let plain = rig.spawn(stepper().steps(5));
    let wrapping = rig.spawn(stepper().steps(5).wrap(true));

    let (prev, wrapping_prev) = (
        prev_button(&rig.app, plain),
        prev_button(&rig.app, wrapping),
    );
    rig.activate(prev);
    assert!(rig.requests().is_empty());

    rig.activate(wrapping_prev);
    assert_eq!(rig.requests(), vec![(wrapping, 4, true)]);
}

#[test]
fn a_request_does_not_change_the_display_until_the_app_applies_it() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(5).value(1));
    let cell = cells(&rig.app, stepper)[3];

    rig.activate(cell);

    let style = probe_theme().stepper;
    assert_eq!(
        rig.app.world().get::<StepperValue>(stepper),
        Some(&StepperValue::new(1))
    );
    assert_eq!(
        color_of(&rig.app, cells(&rig.app, stepper)[1]),
        style.current.normal
    );
}

#[test]
fn the_self_update_observer_applies_the_request() {
    let mut rig = rig();
    rig.app.add_observer(stepper_self_update);
    let stepper = rig.spawn(stepper().steps(5).value(1));

    let cell = cells(&rig.app, stepper)[3];
    rig.activate(cell);

    assert_eq!(
        rig.app.world().get::<StepperValue>(stepper),
        Some(&StepperValue::new(3))
    );
    assert_eq!(
        color_of(&rig.app, cells(&rig.app, stepper)[3]),
        probe_theme().stepper.current.normal
    );
}

#[test]
fn the_arrow_keys_step_and_home_and_end_go_to_the_ends() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(5).value(2));

    rig.press(stepper, KeyCode::ArrowRight, Key::ArrowRight);
    rig.press(stepper, KeyCode::ArrowLeft, Key::ArrowLeft);
    rig.press(stepper, KeyCode::Home, Key::Home);
    rig.press(stepper, KeyCode::End, Key::End);

    let values: Vec<usize> = rig.requests().iter().map(|r| r.1).collect();
    assert_eq!(values, [3, 1, 0, 4]);
}

#[test]
fn keys_that_change_nothing_request_nothing() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(5));

    rig.press(stepper, KeyCode::ArrowLeft, Key::ArrowLeft);
    rig.press(stepper, KeyCode::Home, Key::Home);

    assert!(rig.requests().is_empty());
}

#[test]
fn left_and_right_are_consumed_even_at_an_end_but_up_and_down_pass() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(5));

    rig.press(stepper, KeyCode::ArrowLeft, Key::ArrowLeft);
    rig.press(stepper, KeyCode::ArrowRight, Key::ArrowRight);
    assert_eq!(rig.passed(), 0);

    rig.press(stepper, KeyCode::ArrowUp, Key::ArrowUp);
    rig.press(stepper, KeyCode::ArrowDown, Key::ArrowDown);
    assert_eq!(rig.passed(), 2);
}

#[test]
fn a_disabled_stepper_requests_nothing() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(5).value(2));
    rig.app
        .world_mut()
        .entity_mut(stepper)
        .insert(InteractionDisabled);
    rig.app.update();

    let cell = cells(&rig.app, stepper)[4];
    rig.activate(cell);
    rig.press(stepper, KeyCode::ArrowRight, Key::ArrowRight);

    assert!(rig.requests().is_empty());
}

#[test]
fn a_single_position_requests_nothing() {
    let mut rig = rig();
    let stepper = rig.spawn(stepper().steps(1).wrap(true));

    let cell = cells(&rig.app, stepper)[0];
    rig.activate(cell);
    let (prev, next) = (
        prev_button(&rig.app, stepper),
        next_button(&rig.app, stepper),
    );
    rig.activate(prev);
    rig.activate(next);
    for (key_code, key) in [
        (KeyCode::ArrowLeft, Key::ArrowLeft),
        (KeyCode::ArrowRight, Key::ArrowRight),
        (KeyCode::Home, Key::Home),
        (KeyCode::End, Key::End),
    ] {
        rig.press(stepper, key_code, key);
    }

    assert!(rig.requests().is_empty());
}
