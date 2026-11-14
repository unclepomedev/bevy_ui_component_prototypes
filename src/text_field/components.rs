use super::style::{TextFieldMetrics, TextFieldShape};
use super::value::ReportedText;
use crate::common::{focus::hidden_outline, tab_order::KeyboardNavigable};
use bevy::{
    ecs::query::{QueryData, QueryFilter},
    input_focus::tab_navigation::TabIndex,
    picking::hover::Hovered,
    prelude::*,
    text::TextCursorStyle,
    ui_widgets::TextInput,
};

/// A themed single-line text field: a frame around the editable text of Bevy's [`TextInput`].
///
/// The text is in the child marked [`TextFieldInput`], as `EditableText`. Its entity is the one that has
/// the input focus. The frame itself is not focusable, but pressing on it focuses the input.
///
/// `InteractionDisabled` goes on the field: it is forwarded to the input. A disabled
/// field takes no focus, no selection and no copy; `read_only` is for a text that can be copied.
///
/// The field reports its text as [`bevy::ui_widgets::ValueChange<String>`] with the field as `source`: while
/// the text changes (`is_final` is `false`), and once more when the input loses the focus (`is_final` is
/// `true`). [`SetFieldText`](super::SetFieldText) replaces the text, and Enter in a single line sends
/// [`Submit`](super::Submit).
///
/// With `lines` on the builder it is a text area: Enter starts a new line, long lines wrap, the mouse wheel
/// scrolls it, and a scroll indicator at the right shows where the visible lines are.
///
/// A placeholder, given to the builder, shows while the field is empty and not disabled.
///
/// See the [Conventions](crate#conventions) for the Tab key and the focus ring.
#[derive(Component)]
#[require(
    Hovered,
    Outline = hidden_outline(),
    Node = default_field_node()
)]
pub struct TextField;

/// The entity inside a [`TextField`] that holds the text and the input focus.
#[derive(Component)]
#[require(
    TextInput,
    TextLayout = TextLayout::new(Justify::Left, LineBreak::NoWrap),
    TabIndex(0),
    KeyboardNavigable,
    TextCursorStyle,
    ReportedText,
    Node = input_node()
)]
pub struct TextFieldInput;

/// The text that shows in an empty [`TextField`]. The bundle given to `placeholder` is added to it.
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct TextFieldPlaceholder;

/// The first child of a field that `inputs` matches: its input.
pub(crate) fn input_in<D: QueryData, F: QueryFilter>(
    children: &Children,
    inputs: &Query<D, F>,
) -> Option<Entity> {
    children.iter().find(|child| inputs.contains(*child))
}

/// The frame with the metrics and the shape of the default theme. The width is the builder's.
pub(crate) fn default_field_node() -> Node {
    field_node(&TextFieldMetrics::default(), &TextFieldShape::default())
}

/// The frame.
pub(crate) fn field_node(metrics: &TextFieldMetrics, shape: &TextFieldShape) -> Node {
    Node {
        border: UiRect::all(px(shape.border_width)),
        border_radius: BorderRadius::all(px(shape.radius)),
        padding: metrics.padding,
        ..default()
    }
}

/// The input fills the frame inside the padding. Its height is that of the lines it shows. It has no
/// children: the placeholder is its sibling.
pub(crate) fn input_node() -> Node {
    Node {
        flex_grow: 1.0,
        min_width: px(0),
        ..default()
    }
}

/// The input of a field that has a width of its own: it is as wide as the glyphs it shows.
pub(crate) fn sized_input_node() -> Node {
    Node::default()
}

/// Over the text of the field, from the top left of the padding box. It is displayed while the field is
/// empty.
pub(crate) fn placeholder_node(metrics: &TextFieldMetrics) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: metrics.padding.left,
        top: metrics.padding.top,
        ..default()
    }
}
