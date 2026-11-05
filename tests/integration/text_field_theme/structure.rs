//! The parts of the field and the width it takes.

use super::{input, spawn_field};
use crate::support::{node, test_app};
use bevy::{
    prelude::*,
    text::{EditableText, TextReadWriteMode},
};
use bevy_ui_component_prototypes::TextFieldInput;

#[test]
fn the_text_is_in_an_input_inside_the_frame() {
    let mut app = test_app();
    let field = spawn_field(&mut app);

    app.update();

    let input = input(&app, field);
    assert!(app.world().get::<TextFieldInput>(input).is_some());
    assert!(app.world().get::<EditableText>(input).is_some());
    assert!(app.world().get::<EditableText>(field).is_none());
}

// A node with children is not measured: the frame has the input, and the placeholder is a sibling of it.
#[test]
fn the_input_has_no_children_so_that_it_is_measured() {
    let mut app = test_app();
    let field = spawn_field(&mut app);

    app.update();

    assert!(app.world().get::<Children>(input(&app, field)).is_none());
}

#[test]
fn the_input_fills_the_frame() {
    let mut app = test_app();
    let field = spawn_field(&mut app);

    app.update();

    let node = node(&app, input(&app, field));
    assert_eq!(node.flex_grow, 1.0);
    assert_eq!(node.min_width, px(0));
}

#[test]
fn the_default_field_takes_the_width_of_its_parent_and_is_editable() {
    let mut app = test_app();
    let field = spawn_field(&mut app);

    app.update();

    assert_eq!(node(&app, field).width, percent(100));
    assert_eq!(
        app.world().get::<TextReadWriteMode>(input(&app, field)),
        Some(&TextReadWriteMode::Editable)
    );
}
