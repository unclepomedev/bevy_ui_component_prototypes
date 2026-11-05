use super::components::{TextField, TextFieldInput, input_in};
use bevy::{
    input_focus::FocusLost,
    prelude::*,
    text::{EditableText, EditableTextSystems, TextEdit},
    ui_widgets::ValueChange,
};

/// Replaces the text of a [`TextField`]. The text is cut at the `max_characters` of the field, and the line
/// breaks of a text are dropped in a field of a single line. The cursor goes to the end of the text when it
/// changes. The field does not report the change it was asked for.
#[derive(EntityEvent, Clone, Debug, PartialEq, Eq)]
pub struct SetFieldText {
    #[event_target]
    pub field: Entity,
    pub text: String,
}

type ChangedInput = (
    &'static EditableText,
    &'static ChildOf,
    &'static mut ReportedText,
);

/// The text the app was last told about. The builder starts it with the initial text, which is not a change.
#[derive(Component, Default, Deref, DerefMut)]
pub(super) struct ReportedText(pub(super) String);

pub(crate) struct TextFieldValuePlugin;
impl Plugin for TextFieldValuePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_set_field_text)
            .add_observer(report_blur)
            .add_systems(PostUpdate, report_changes.after(EditableTextSystems));
    }
}

/// The first `max` characters of `text`; all of it when there is no limit.
fn limited(text: &str, max: Option<usize>) -> &str {
    match max {
        Some(max) => text
            .char_indices()
            .nth(max)
            .map_or(text, |(end, _)| &text[..end]),
        None => text,
    }
}

/// `text` as a field takes it: without line breaks unless `newlines` are allowed, and cut at `max`.
pub(super) fn fit_text(text: &str, newlines: bool, max: Option<usize>) -> String {
    if newlines {
        limited(text, max).to_string()
    } else {
        let single: String = text.chars().filter(|c| !matches!(c, '\r' | '\n')).collect();
        limited(&single, max).to_string()
    }
}

/// Tells the app the text of a field when it is not the one it was told last, as a
/// [`ValueChange<String>`] that is not final.
fn report_changes(
    mut inputs: Query<ChangedInput, (With<TextFieldInput>, Changed<EditableText>)>,
    fields: Query<(), With<TextField>>,
    mut commands: Commands,
) {
    for (text, parent, mut reported) in &mut inputs {
        if text.value() == reported.as_str() {
            continue;
        }
        let value = text.value().to_string();
        reported.clone_from(&value);
        if fields.contains(parent.parent()) {
            commands.trigger(ValueChange {
                source: parent.parent(),
                value,
                is_final: false,
            });
        }
    }
}

/// Tells the app the text of a field when the input loses the focus, as a final [`ValueChange<String>`].
fn report_blur(
    lost: On<FocusLost>,
    inputs: Query<(&EditableText, &ChildOf), With<TextFieldInput>>,
    fields: Query<(), With<TextField>>,
    mut commands: Commands,
) {
    if let Ok((text, parent)) = inputs.get(lost.entity)
        && fields.contains(parent.parent())
    {
        commands.trigger(ValueChange {
            source: parent.parent(),
            value: text.value().to_string(),
            is_final: true,
        });
    }
}

fn on_set_field_text(
    set: On<SetFieldText>,
    fields: Query<&Children, With<TextField>>,
    mut inputs: Query<&mut EditableText, With<TextFieldInput>>,
    mut commands: Commands,
) {
    let Ok(children) = fields.get(set.field) else {
        return;
    };
    let Some(input) = input_in(children, &inputs) else {
        return;
    };
    let Ok(mut text) = inputs.get_mut(input) else {
        return;
    };
    let new = fit_text(&set.text, text.allow_newlines, text.max_characters);
    if text.value() != new.as_str() {
        // The edits that were queued were made for the text that is replaced.
        text.pending_edits.clear();
        text.editor_mut().set_text(&new);
        text.queue_edit(TextEdit::TextEnd(false));
    }
    commands.entity(input).try_insert(ReportedText(new));
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_is_cut_at_the_limit_in_characters_not_bytes() {
        assert_eq!(limited("abcdef", Some(3)), "abc");
        assert_eq!(limited("あいうえお", Some(2)), "あい");
        assert_eq!(limited("abc", Some(3)), "abc");
        assert_eq!(limited("abc", Some(10)), "abc");
        assert_eq!(limited("abc", Some(0)), "");
        assert_eq!(limited("abc", None), "abc");
    }

    #[test]
    fn line_breaks_are_dropped_unless_allowed_and_then_the_limit_applies() {
        assert_eq!(fit_text("a\r\nb\nc", false, None), "abc");
        assert_eq!(fit_text("a\nb\nc", false, Some(2)), "ab");
        assert_eq!(fit_text("a\nb\nc", true, None), "a\nb\nc");
        assert_eq!(fit_text("a\nb\nc", true, Some(3)), "a\nb");
    }
}
