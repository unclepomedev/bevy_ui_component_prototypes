//! The placeholder.

use super::{input, placeholder, spawn_field, text_color};
use crate::{
    probe::probe_theme,
    support::{node, passes_through, test_app},
};
use bevy::{prelude::*, text::EditableText};
use bevy_ui_component_prototypes::text_field;

#[test]
fn the_placeholder_shows_while_the_field_is_empty() {
    let mut app = test_app();
    let style = probe_theme().text_field;
    let field = spawn_field(&mut app);

    app.update();

    let placeholder = placeholder(&app, field);
    assert_eq!(node(&app, placeholder).display, Display::Flex);
    assert_eq!(text_color(&app, placeholder), style.placeholder);
    assert_eq!(
        app.world().get::<TextFont>(placeholder).unwrap().font_size,
        probe_theme().text_field_metrics.font_size
    );
}

#[test]
fn the_placeholder_goes_when_the_field_has_text() {
    let mut app = test_app();
    let field = app
        .world_mut()
        .spawn(
            text_field()
                .initial("Ada")
                .placeholder(Text::new("Name"))
                .build(),
        )
        .id();

    app.update();

    assert_eq!(node(&app, placeholder(&app, field)).display, Display::None);
}

#[test]
fn the_placeholder_follows_the_text() {
    let mut app = test_app();
    let field = spawn_field(&mut app);
    app.update();
    let input = input(&app, field);

    let mut text = app.world_mut().get_mut::<EditableText>(input).unwrap();
    text.editor_mut().set_text("x");
    app.update();
    assert_eq!(node(&app, placeholder(&app, field)).display, Display::None);

    let mut text = app.world_mut().get_mut::<EditableText>(input).unwrap();
    text.editor_mut().set_text("");
    app.update();
    assert_eq!(node(&app, placeholder(&app, field)).display, Display::Flex);
}

#[test]
fn the_placeholder_sits_at_the_padding_and_lets_the_pointer_through() {
    let mut app = test_app();
    let padding = probe_theme().text_field_metrics.padding;
    let field = spawn_field(&mut app);

    app.update();

    let placeholder = placeholder(&app, field);
    let node = node(&app, placeholder);
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!((node.left, node.top), (padding.left, padding.top));
    assert!(passes_through(
        app.world().get::<Pickable>(placeholder).unwrap()
    ));
}
