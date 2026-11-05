//! What goes to the input: the disabled state, the name, and the focus.

use super::{input, spawn_field};
use crate::support::{focus, tab_index, test_app};
use bevy::{
    input_focus::{AcquireFocus, InputFocus},
    prelude::*,
    ui::InteractionDisabled,
    window::PrimaryWindow,
};
use bevy_ui_component_prototypes::text_field;

#[test]
fn disabling_the_field_disables_its_input_and_enabling_it_enables_the_input_again() {
    let mut app = test_app();
    let field = spawn_field(&mut app);
    app.update();
    let input = input(&app, field);

    app.world_mut()
        .entity_mut(field)
        .insert(InteractionDisabled);
    app.update();
    app.update();
    assert!(app.world().get::<InteractionDisabled>(input).is_some());
    assert_eq!(tab_index(&app, input), -1);

    app.world_mut()
        .entity_mut(field)
        .remove::<InteractionDisabled>();
    app.update();
    app.update();
    assert!(app.world().get::<InteractionDisabled>(input).is_none());
    assert_eq!(tab_index(&app, input), 0);
}

#[test]
fn a_field_that_is_spawned_disabled_has_a_disabled_input() {
    let mut app = test_app();
    let field = app
        .world_mut()
        .spawn((text_field().build(), InteractionDisabled))
        .id();

    app.update();
    app.update();

    assert!(
        app.world()
            .get::<InteractionDisabled>(input(&app, field))
            .is_some()
    );
}

#[test]
fn the_name_of_the_field_goes_to_the_input() {
    let mut app = test_app();
    let field = spawn_field(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(field)
        .insert(AccessibleLabel::new("Your name"));
    app.update();

    let label = app
        .world()
        .get::<AccessibleLabel>(input(&app, field))
        .unwrap();
    assert_eq!(label.0, "Your name");
    assert!(
        app.world()
            .get::<bevy::a11y::AccessibilityNode>(field)
            .is_none()
    );
}

#[test]
fn pressing_on_the_frame_focuses_the_input() {
    let mut app = test_app();
    let window = app.world_mut().spawn(PrimaryWindow).id();
    app.insert_resource(InputFocus::default());
    let field = spawn_field(&mut app);
    app.update();

    app.world_mut().trigger(AcquireFocus {
        focused_entity: field,
        window,
    });

    let input = input(&app, field);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(input));
}

#[test]
fn pressing_on_a_disabled_frame_does_not_focus_the_input() {
    let mut app = test_app();
    let window = app.world_mut().spawn(PrimaryWindow).id();
    app.insert_resource(InputFocus::default());
    let field = app
        .world_mut()
        .spawn((text_field().build(), InteractionDisabled))
        .id();
    app.update();

    app.world_mut().trigger(AcquireFocus {
        focused_entity: field,
        window,
    });

    assert_eq!(app.world().resource::<InputFocus>().get(), None);
}

// A disabled input does not keep the focus, whoever gave it.
#[test]
fn a_disabled_input_does_not_keep_the_focus() {
    let mut app = test_app();
    let field = spawn_field(&mut app);
    app.world_mut()
        .entity_mut(field)
        .insert(InteractionDisabled);
    app.update();
    let input = input(&app, field);

    focus(&mut app, input, true);
    app.update();

    assert_eq!(app.world().resource::<InputFocus>().get(), None);
}

#[test]
fn an_input_that_is_focused_when_its_field_is_disabled_loses_the_focus() {
    let mut app = test_app();
    let field = spawn_field(&mut app);
    app.update();
    let input = input(&app, field);
    focus(&mut app, input, true);
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(input));

    app.world_mut()
        .entity_mut(field)
        .insert(InteractionDisabled);
    app.update();
    app.update();

    assert_eq!(app.world().resource::<InputFocus>().get(), None);
}

#[test]
fn a_disabled_input_is_out_of_the_picking_and_comes_back_when_enabled() {
    let mut app = test_app();
    let field = spawn_field(&mut app);
    app.update();
    let input = input(&app, field);
    assert!(app.world().get::<Pickable>(input).is_none());

    app.world_mut()
        .entity_mut(field)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(app.world().get::<Pickable>(input), Some(&Pickable::IGNORE));

    app.world_mut()
        .entity_mut(field)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(app.world().get::<Pickable>(input).is_none());
}
