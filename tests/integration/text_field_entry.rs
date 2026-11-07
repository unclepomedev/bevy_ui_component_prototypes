//! What goes into a text field: the characters it accepts, and the Enter that submits it.

use crate::support::test_app;
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyCode, KeyboardInput},
    },
    input_focus::FocusedInput,
    prelude::*,
    text::{EditableText, EditableTextSystems, TextEdit},
    ui_widgets::ValueChange,
};
use bevy_ui_component_prototypes::{SetFieldText, Submit, text_field};

#[derive(Resource, Default)]
struct Submitted(Vec<(Entity, String)>);

fn app_with_log() -> App {
    let mut app = test_app();
    app.init_resource::<Submitted>();
    app.add_observer(|submit: On<Submit>, mut seen: ResMut<Submitted>| {
        seen.0.push((submit.field, submit.value.clone()));
    });
    app
}

fn input(app: &App, field: Entity) -> Entity {
    app.world().get::<Children>(field).unwrap()[0]
}

fn text(app: &App, field: Entity) -> String {
    app.world()
        .get::<EditableText>(input(app, field))
        .unwrap()
        .value()
        .to_string()
}

fn press(app: &mut App, field: Entity, logical_key: Key, repeat: bool) {
    let window = app.world_mut().spawn_empty().id();
    let input = input(app, field);
    let key = KeyboardInput {
        key_code: KeyCode::Enter,
        logical_key,
        state: ButtonState::Pressed,
        text: None,
        repeat,
        window,
    };
    app.world_mut()
        .trigger(FocusedInput::new(input, key, window));
    app.update();
}

fn submitted(app: &App) -> Vec<(Entity, String)> {
    app.world().resource::<Submitted>().0.clone()
}

/// The edits an input has waiting, after the filter had its turn.
fn pending(app: &mut App, field: Entity, edits: Vec<TextEdit>) -> Vec<TextEdit> {
    let input = input(app, field);
    app.world_mut()
        .get_mut::<EditableText>(input)
        .unwrap()
        .pending_edits = edits;
    app.update();
    app.world()
        .get::<EditableText>(input)
        .unwrap()
        .pending_edits
        .clone()
}

fn passcode() -> impl Bundle {
    text_field()
        .max_characters(4)
        .char_filter(|c| c.is_ascii_alphanumeric().then(|| c.to_ascii_uppercase()))
        .build()
}

#[test]
fn enter_in_a_single_line_field_submits_its_text_once() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(text_field().initial("07KT").build())
        .id();
    app.update();

    press(&mut app, field, Key::Enter, false);

    assert_eq!(submitted(&app), vec![(field, "07KT".to_string())]);
}

#[test]
fn a_read_only_field_submits_too() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(text_field().initial("a").read_only(true).build())
        .id();
    app.update();

    press(&mut app, field, Key::Enter, false);

    assert_eq!(submitted(&app).len(), 1);
}

#[test]
fn a_text_area_takes_enter_as_a_new_line_and_does_not_submit() {
    let mut app = app_with_log();
    let field = app.world_mut().spawn(text_field().lines(3.0).build()).id();
    app.update();

    press(&mut app, field, Key::Enter, false);

    assert!(submitted(&app).is_empty());
}

#[test]
fn other_keys_and_a_held_enter_do_not_submit() {
    let mut app = app_with_log();
    let field = app.world_mut().spawn(text_field().build()).id();
    app.update();

    press(&mut app, field, Key::Character("a".into()), false);
    press(&mut app, field, Key::Enter, true);

    assert!(submitted(&app).is_empty());
}

#[test]
fn the_filter_changes_and_drops_typed_characters() {
    let mut app = app_with_log();
    let field = app.world_mut().spawn(passcode()).id();
    app.update();

    let edits = pending(
        &mut app,
        field,
        vec![
            TextEdit::Insert("07kt".into()),
            TextEdit::Insert("-".into()),
            TextEdit::Insert("a-b".into()),
        ],
    );

    assert_eq!(
        edits,
        vec![
            TextEdit::Insert("07KT".into()),
            TextEdit::Insert("AB".into())
        ]
    );
}

#[test]
fn the_filter_applies_to_what_an_input_method_commits_and_keeps_the_commit() {
    let mut app = app_with_log();
    let field = app.world_mut().spawn(passcode()).id();
    app.update();

    let edits = pending(
        &mut app,
        field,
        vec![
            TextEdit::ImeCommit {
                value: "あ1".into(),
            },
            TextEdit::ImeCommit {
                value: "あ".into()
            },
        ],
    );

    // An empty commit is kept: it ends the composition.
    assert_eq!(
        edits,
        vec![
            TextEdit::ImeCommit { value: "1".into() },
            TextEdit::ImeCommit { value: "".into() }
        ]
    );
}

#[test]
fn edits_that_do_not_take_text_are_left_alone() {
    let mut app = app_with_log();
    let field = app.world_mut().spawn(passcode()).id();
    app.update();

    let edits = pending(
        &mut app,
        field,
        vec![TextEdit::SelectAll, TextEdit::Backspace],
    );

    assert_eq!(edits, vec![TextEdit::SelectAll, TextEdit::Backspace]);
}

#[test]
fn a_field_without_a_filter_keeps_its_edits() {
    let mut app = app_with_log();
    let field = app.world_mut().spawn(text_field().build()).id();
    app.update();

    let edits = pending(&mut app, field, vec![TextEdit::Insert("a-b".into())]);

    assert_eq!(edits, vec![TextEdit::Insert("a-b".into())]);
}

#[test]
fn the_initial_text_and_a_replacement_go_through_the_filter_before_the_limit() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(
            text_field()
                .max_characters(4)
                .char_filter(|c| c.is_ascii_digit().then_some(c))
                .initial("1-2-3-4-5")
                .build(),
        )
        .id();
    app.update();
    assert_eq!(text(&app, field), "1234");

    app.world_mut().trigger(SetFieldText {
        field,
        text: "9a8b7".into(),
    });
    app.update();

    assert_eq!(text(&app, field), "987");
}

#[test]
fn a_text_area_asks_the_filter_about_line_breaks_too() {
    let mut app = app_with_log();
    let field = app
        .world_mut()
        .spawn(
            text_field()
                .lines(3.0)
                .char_filter(|c| (c != '\n').then_some(c))
                .initial("a\nb")
                .build(),
        )
        .id();
    app.update();

    assert_eq!(text(&app, field), "ab");
}

#[test]
fn the_submitted_text_has_the_edits_of_the_frame_of_the_enter() {
    #[derive(Resource, Default)]
    struct Order(Vec<String>);

    let mut app = app_with_log();
    app.init_resource::<Order>();
    app.add_observer(|submit: On<Submit>, mut order: ResMut<Order>| {
        order.0.push(format!("submit {}", submit.value));
    });
    app.add_observer(
        |change: On<ValueChange<String>>, mut order: ResMut<Order>| {
            order.0.push(format!("change {}", change.value));
        },
    );
    let field = app
        .world_mut()
        .spawn(text_field().initial("07K").build())
        .id();
    app.update();
    let input = input(&app, field);
    // Stands in for the system of Bevy that applies the edits that were queued in the frame.
    app.add_systems(
        PostUpdate,
        (move |mut texts: Query<&mut EditableText>| {
            if let Ok(mut text) = texts.get_mut(input)
                && text.value() == "07K"
            {
                text.editor_mut().set_text("07KT");
            }
        })
        .in_set(EditableTextSystems),
    );

    press(&mut app, field, Key::Enter, false);

    assert_eq!(submitted(&app), vec![(field, "07KT".to_string())]);
    let order = &app.world().resource::<Order>().0;
    assert_eq!(order, &["change 07KT", "submit 07KT"]);
}
