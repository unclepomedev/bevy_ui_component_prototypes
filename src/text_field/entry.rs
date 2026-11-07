//! What goes into a field: the characters it accepts, and the Enter that submits it.

use super::components::{TextField, TextFieldInput};
use bevy::{
    clipboard::Clipboard,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::FocusedInput,
    prelude::*,
    text::{EditableText, EditableTextSystems, TextEdit},
};
use std::{mem::take, sync::Arc};

/// Sent to a single-line [`TextField`] when Enter is pressed in it.
/// Not sent by a text area, nor while an input method is composing.
#[derive(EntityEvent, Clone, Debug, PartialEq, Eq)]
pub struct Submit {
    #[event_target]
    pub field: Entity,
    pub value: String,
}

/// Per-character input filter: keep, replace, or drop (`None`).
#[derive(Component, Clone)]
pub(super) struct CharFilter(Arc<dyn Fn(char) -> Option<char> + Send + Sync>);

impl CharFilter {
    pub(super) fn new(filter: impl Fn(char) -> Option<char> + Send + Sync + 'static) -> Self {
        Self(Arc::new(filter))
    }

    pub(super) fn apply(&self, text: &str) -> String {
        text.chars().filter_map(|c| self.0(c)).collect()
    }
}

pub(crate) struct TextFieldEntryPlugin;
impl Plugin for TextFieldEntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(submit_on_enter)
            .add_systems(PostUpdate, filter_edits.before(EditableTextSystems));
    }
}

fn submit_on_enter(
    key: On<FocusedInput<KeyboardInput>>,
    inputs: Query<(&EditableText, &ChildOf), With<TextFieldInput>>,
    fields: Query<(), With<TextField>>,
    mut commands: Commands,
) {
    // The event passes the input on its way up: act where it starts.
    if key.event_target() != key.focused_entity {
        return;
    }
    let input = &key.input;
    if input.state != ButtonState::Pressed || input.repeat || input.logical_key != Key::Enter {
        return;
    }
    let Ok((text, parent)) = inputs.get(key.focused_entity) else {
        return;
    };
    if text.allow_newlines || text.is_composing() || !fields.contains(parent.parent()) {
        return;
    }
    commands.trigger(Submit {
        field: parent.parent(),
        value: text.value().to_string(),
    });
}

/// What a field takes of `raw`: without line breaks unless it is a text area, through the filter.
fn clean(filter: &CharFilter, newlines: bool, raw: &str) -> String {
    if newlines {
        filter.apply(raw)
    } else {
        filter.apply(&raw.replace(['\r', '\n'], ""))
    }
}

/// The text of the clipboard, when it can be read at once.
fn clipboard_text(clipboard: &mut Clipboard) -> Option<String> {
    clipboard.fetch_text().poll_result()?.ok()
}

/// Filters inserted, IME-committed, and pasted text before insertion.
/// Character limits apply to the filtered text.
fn filter_edits(
    mut inputs: Query<(&mut EditableText, &CharFilter), With<TextFieldInput>>,
    mut clipboard: Option<ResMut<Clipboard>>,
) {
    for (mut text, filter) in &mut inputs {
        let takes_text = |edit: &TextEdit| {
            matches!(
                edit,
                TextEdit::Insert(_) | TextEdit::ImeCommit { .. } | TextEdit::Paste
            )
        };
        if !text.pending_edits.iter().any(takes_text) {
            continue;
        }
        let newlines = text.allow_newlines;
        let mut edits = Vec::with_capacity(text.pending_edits.len());
        for edit in take(&mut text.pending_edits) {
            match edit {
                TextEdit::Insert(raw) => {
                    let kept = clean(filter, newlines, &raw);
                    if !kept.is_empty() {
                        edits.push(TextEdit::Insert(kept.into()));
                    }
                }
                // An empty commit still ends the composition.
                TextEdit::ImeCommit { value } => edits.push(TextEdit::ImeCommit {
                    value: clean(filter, newlines, &value).into(),
                }),
                TextEdit::Paste => {
                    let pasted = clipboard.as_mut().and_then(|c| clipboard_text(c));
                    if let Some(kept) = pasted.map(|raw| clean(filter, newlines, &raw))
                        && !kept.is_empty()
                    {
                        edits.push(TextEdit::Insert(kept.into()));
                    }
                }
                other => edits.push(other),
            }
        }
        text.pending_edits = edits;
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn passcode() -> CharFilter {
        CharFilter::new(|c| c.is_ascii_alphanumeric().then(|| c.to_ascii_uppercase()))
    }

    #[test]
    fn characters_are_kept_replaced_or_dropped() {
        let filter = passcode();
        assert_eq!(filter.apply("07kt"), "07KT");
        assert_eq!(filter.apply("07-K"), "07K");
        assert_eq!(filter.apply("あ-"), "");
    }
}
