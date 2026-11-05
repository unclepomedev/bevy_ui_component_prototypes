use super::components::{TextField, TextFieldInput, TextFieldPlaceholder};
use crate::common::focus::ShowsFocus;
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    picking::hover::Hovered,
    prelude::*,
    text::{EditableText, TextCursorStyle},
    ui::InteractionDisabled,
};

/// The entities of the text fields that the sync reads and writes.
#[derive(SystemParam)]
pub(super) struct FieldParts<'w, 's> {
    pub(super) fields: Query<'w, 's, FieldRow, With<TextField>>,
    pub(super) inputs: Query<'w, 's, InputRow, (With<TextFieldInput>, Without<TextField>)>,
    pub(super) placeholders: Placeholders<'w, 's>,
    pub(super) children: Query<'w, 's, &'static Children>,
}

pub(super) type Placeholders<'w, 's> = Query<
    'w,
    's,
    PlaceholderRow,
    (
        With<TextFieldPlaceholder>,
        Without<TextField>,
        Without<TextFieldInput>,
    ),
>;

/// What the sync reads from a text field, and writes to it.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct FieldRow {
    pub(super) hovered: &'static Hovered,
    pub(super) disabled: Has<InteractionDisabled>,
    pub(super) bg: &'static mut BackgroundColor,
    pub(super) border: &'static mut BorderColor,
    pub(super) outline: &'static mut Outline,
    pub(super) node: &'static mut Node,
}

/// What the sync reads from the input of a text field, and writes to it.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct InputRow {
    pub(super) text: &'static EditableText,
    pub(super) disabled: Has<InteractionDisabled>,
    pub(super) shows_focus: Has<ShowsFocus>,
    pub(super) color: &'static mut TextColor,
    pub(super) font: &'static mut TextFont,
    pub(super) cursor: &'static mut TextCursorStyle,
}

/// What the sync writes to the placeholder.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct PlaceholderRow {
    pub(super) color: &'static mut TextColor,
    pub(super) font: &'static mut TextFont,
    pub(super) node: &'static mut Node,
}
