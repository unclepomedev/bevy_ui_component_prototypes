//! The text of a text field reaching the app as `ValueChange<String>`, and `SetFieldText` going the other way.

use crate::support::test_app;
use bevy::{input_focus::FocusLost, prelude::*, text::EditableText, ui_widgets::ValueChange};
use bevy_ui_component_prototypes::{SetFieldText, text_field};

#[derive(Resource, Default)]
struct Seen(Vec<(Entity, String, bool)>);

fn app_with_log() -> App {
    let mut app = test_app();
    app.init_resource::<Seen>();
    app.add_observer(|change: On<ValueChange<String>>, mut seen: ResMut<Seen>| {
        seen.0
            .push((change.source, change.value.clone(), change.is_final));
    });
    app
}

fn input(app: &App, field: Entity) -> Entity {
    app.world().get::<Children>(field).unwrap()[0]
}

fn text(app: &App, field: Entity) -> String {
    let input = input(app, field);
    app.world()
        .get::<EditableText>(input)
        .unwrap()
        .value()
        .to_string()
}

fn type_text(app: &mut App, field: Entity, new: &str) {
    let input = input(app, field);
    app.world_mut()
        .get_mut::<EditableText>(input)
        .unwrap()
        .editor_mut()
        .set_text(new);
    app.update();
}

fn seen(app: &App) -> Vec<(Entity, String, bool)> {
    app.world().resource::<Seen>().0.clone()
}

#[test]
fn the_text_it_starts_with_is_not_a_change() {
    let mut app = app_with_log();
    app.world_mut().spawn(text_field().initial("hello").build());

    app.update();
    app.update();

    assert!(seen(&app).is_empty());
}

#[test]
fn a_change_of_the_text_is_reported_with_the_field_as_source() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(text_field().initial("a").build())
        .id();
    app.update();

    type_text(&mut app, field, "ab");

    assert_eq!(seen(&app), vec![(field, "ab".to_string(), false)]);
}

#[test]
fn touching_the_editor_without_changing_the_text_is_silent() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(text_field().initial("a").build())
        .id();
    app.update();

    type_text(&mut app, field, "a");
    type_text(&mut app, field, "a");

    assert!(seen(&app).is_empty());
}

#[test]
fn losing_the_focus_reports_the_text_as_final() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(text_field().initial("a").build())
        .id();
    app.update();
    let input = input(&app, field);

    app.world_mut().trigger(FocusLost { entity: input });
    app.update();

    assert_eq!(seen(&app), vec![(field, "a".to_string(), true)]);
}

#[test]
fn set_field_text_replaces_the_text_without_reporting_it() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(text_field().initial("a").build())
        .id();
    app.update();

    app.world_mut().trigger(SetFieldText {
        field,
        text: "new".into(),
    });
    app.update();
    app.update();

    assert_eq!(text(&app, field), "new");
    assert!(seen(&app).is_empty());
}

#[test]
fn a_change_after_set_field_text_is_reported_again() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(text_field().initial("a").build())
        .id();
    app.update();
    app.world_mut().trigger(SetFieldText {
        field,
        text: "new".into(),
    });
    app.update();

    type_text(&mut app, field, "newer");

    assert_eq!(seen(&app), vec![(field, "newer".to_string(), false)]);
}

#[test]
fn set_field_text_cuts_the_text_at_the_limit() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(text_field().max_characters(3).build())
        .id();
    app.update();

    app.world_mut().trigger(SetFieldText {
        field,
        text: "あいうえお".into(),
    });
    app.update();

    assert_eq!(text(&app, field), "あいう");
}

#[test]
fn set_field_text_for_something_else_is_ignored() {
    let mut app = app_with_log();
    let other = app.world_mut().spawn_empty().id();
    app.update();

    app.world_mut().trigger(SetFieldText {
        field: other,
        text: "x".into(),
    });
    app.update();

    assert!(seen(&app).is_empty());
}
