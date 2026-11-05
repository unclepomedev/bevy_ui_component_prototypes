//! What the options of the builder set.

use super::{input, spawn_field};
use crate::support::{node, test_app};
use bevy::{
    prelude::*,
    text::{EditableText, TextReadWriteMode},
};
use bevy_ui_component_prototypes::text_field;

#[test]
fn the_builder_options_go_to_the_input() {
    let mut app = test_app();
    let field = app
        .world_mut()
        .spawn(
            text_field()
                .initial("abc")
                .max_characters(5)
                .visible_width(12.0)
                .read_only(true)
                .build(),
        )
        .id();

    app.update();

    let input = input(&app, field);
    let text = app.world().get::<EditableText>(input).unwrap();
    assert_eq!(text.value().to_string(), "abc");
    assert_eq!(text.max_characters, Some(5));
    assert_eq!(text.visible_width, Some(12.0));
    assert_eq!(
        app.world().get::<TextReadWriteMode>(input),
        Some(&TextReadWriteMode::ReadOnly)
    );
    // With a width of its own the field is not as wide as its parent.
    let node = node(&app, field);
    assert_ne!(node.width, percent(100));
    assert_eq!(node.align_self, AlignSelf::FlexStart);
}

#[test]
fn lines_make_a_text_area_that_takes_newlines_and_wraps() {
    let mut app = test_app();
    let area = app
        .world_mut()
        .spawn(text_field().lines(4.0).initial("a\nb").build())
        .id();
    let single = spawn_field(&mut app);

    app.update();

    let text = app.world().get::<EditableText>(input(&app, area)).unwrap();
    assert!(text.allow_newlines);
    assert_eq!(text.visible_lines, Some(4.0));
    assert_eq!(text.value().to_string(), "a\nb");
    let layout = app.world().get::<TextLayout>(input(&app, area)).unwrap();
    assert_eq!(layout.linebreak, LineBreak::WordBoundary);
    // A field without `lines` stays a single line.
    let text = app
        .world()
        .get::<EditableText>(input(&app, single))
        .unwrap();
    assert!(!text.allow_newlines);
    assert_eq!(text.visible_lines, Some(1.0));
    let layout = app.world().get::<TextLayout>(input(&app, single)).unwrap();
    assert_eq!(layout.linebreak, LineBreak::NoWrap);
}

#[test]
fn fewer_than_one_line_is_one_line() {
    let mut app = test_app();
    let area = app.world_mut().spawn(text_field().lines(0.0).build()).id();

    app.update();

    let text = app.world().get::<EditableText>(input(&app, area)).unwrap();
    assert_eq!(text.visible_lines, Some(1.0));
}

#[test]
fn the_initial_text_is_cut_at_the_limit_and_has_no_line_breaks_in_a_single_line() {
    let mut app = test_app();
    let cut = app
        .world_mut()
        .spawn(text_field().initial("あいうえお").max_characters(3).build())
        .id();
    let single = app
        .world_mut()
        .spawn(text_field().initial("a\nb\r\nc").build())
        .id();
    let area = app
        .world_mut()
        .spawn(text_field().lines(3.0).initial("a\nb").build())
        .id();

    app.update();

    let text = |field| {
        app.world()
            .get::<EditableText>(input(&app, field))
            .unwrap()
            .value()
            .to_string()
    };
    assert_eq!(text(cut), "あいう");
    assert_eq!(text(single), "abc");
    assert_eq!(text(area), "a\nb");
}
