use super::{
    components::{
        StepperButton, StepperCell, StepperOwners, StepperValue, StepperView, StepperViewItem,
    },
    logic::Move,
};
use bevy::{
    input::{
        ButtonState,
        keyboard::{KeyCode, KeyboardInput},
    },
    input_focus::FocusedInput,
    prelude::*,
    ui_widgets::{Activate, ValueChange},
};

/// Applies each [`ValueChange<usize>`] to the stepper that sent it.
/// Add it as an observer to let the stepper update itself, instead of applying the requests in the app.
pub fn stepper_self_update(change: On<ValueChange<usize>>, mut commands: Commands) {
    commands
        .entity(change.source)
        .insert(StepperValue::new(change.value));
}

/// Turns presses on a stepper into requests.
pub(crate) struct StepperInputPlugin;
impl Plugin for StepperInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_activate).add_observer(on_key);
    }
}

/// A cell or a button was pressed.
fn on_activate(
    activate: On<Activate>,
    parts: Query<(Option<&StepperCell>, Option<&StepperButton>)>,
    owners: StepperOwners,
    steppers: Query<StepperView>,
    mut commands: Commands,
) {
    let Ok((cell, button)) = parts.get(activate.entity) else {
        return;
    };
    let request = match (cell, button) {
        (Some(cell), _) => Move::To(cell.index()),
        (None, Some(button)) => Move::By(button.delta()),
        (None, None) => return,
    };
    let Some(owner) = owners.of(activate.entity) else {
        return;
    };
    if let Ok(stepper) = steppers.get(owner)
        && !stepper.is_disabled()
    {
        report(&mut commands, owner, &stepper, request);
    }
}

fn on_key(
    mut input: On<FocusedInput<KeyboardInput>>,
    steppers: Query<StepperView>,
    mut commands: Commands,
) {
    let source = input.focused_entity;
    let Ok(stepper) = steppers.get(source) else {
        return;
    };
    if stepper.is_disabled() {
        return;
    }
    if input.input.state != ButtonState::Pressed {
        return;
    }
    let request = match input.input.key_code {
        KeyCode::ArrowLeft => Move::By(-1),
        KeyCode::ArrowRight => Move::By(1),
        KeyCode::Home => Move::First,
        KeyCode::End => Move::Last,
        _ => return,
    };
    // Consumed even when it changes nothing: left to propagate, an arrow at an end would let
    // `AutoDirectionalNavigation` move the focus away.
    input.propagate(false);
    report(&mut commands, source, &stepper, request);
}

/// Sends the change `request` asks of the stepper, unless it would not change the value.
fn report(commands: &mut Commands, source: Entity, stepper: &StepperViewItem, request: Move) {
    if let Some(index) = stepper.change_for(request) {
        commands.trigger(ValueChange {
            source,
            value: index,
            is_final: true,
        });
    }
}
