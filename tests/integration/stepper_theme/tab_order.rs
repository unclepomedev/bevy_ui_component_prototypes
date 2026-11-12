//! Tests of the tab order.

use super::{descendants, spawn_default};
use crate::support::{tab_index, test_app};
use bevy::{input_focus::tab_navigation::TabIndex, ui::InteractionDisabled};

#[test]
fn the_stepper_is_the_only_tab_stop() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);
    app.update();

    assert_eq!(tab_index(&app, stepper), 0);
    let with_index = descendants(&app, stepper)
        .into_iter()
        .filter(|part| app.world().get::<TabIndex>(*part).is_some())
        .count();
    assert_eq!(with_index, 0);
}

#[test]
fn a_disabled_stepper_leaves_the_tab_order_and_returns_when_enabled() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(stepper)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(tab_index(&app, stepper), -1);

    app.world_mut()
        .entity_mut(stepper)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, stepper), 0);
}
