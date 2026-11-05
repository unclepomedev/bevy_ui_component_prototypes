use super::{
    components::{
        TextField, TextFieldInput, TextFieldPlaceholder, default_field_node, placeholder_node,
        sized_input_node,
    },
    style::TextFieldMetrics,
    value::{ReportedText, fit_text},
};
use crate::{common::picking::pass_through, scroll_indicator::scroll_indicator};
use accesskit::{Node as AccessKitNode, Role};
use bevy::{
    a11y::AccessibilityNode,
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
    text::{EditableText, TextReadWriteMode},
};

type Extra = Box<dyn FnOnce(&mut EntityWorldMut) + Send + Sync>;

/// Starts an empty single-line [`TextField`] as wide as its parent.
pub fn text_field() -> TextFieldBuilder {
    TextFieldBuilder {
        initial: String::new(),
        max_characters: None,
        visible_width: None,
        lines: None,
        read_only: false,
        placeholder_extras: Vec::new(),
    }
}

pub struct TextFieldBuilder {
    initial: String,
    max_characters: Option<usize>,
    visible_width: Option<f32>,
    lines: Option<f32>,
    read_only: bool,
    placeholder_extras: Vec<Extra>,
}

impl TextFieldBuilder {
    /// The text the field starts with, cut at `max_characters` and without line breaks unless `lines` is set.
    pub fn initial(mut self, text: impl Into<String>) -> Self {
        self.initial = text.into();
        self
    }

    /// Adds `bundle` to the placeholder entity, which is styled as text: give it a `Text`.
    /// Can be called more than once.
    pub fn placeholder(mut self, bundle: impl Bundle) -> Self {
        self.placeholder_extras.push(Box::new(move |placeholder| {
            placeholder.insert(bundle);
        }));
        self
    }

    /// The most characters the field takes in.
    pub fn max_characters(mut self, count: usize) -> Self {
        self.max_characters = Some(count);
        self
    }

    /// The width in `glyphs` instead of the width of the parent.
    pub fn visible_width(mut self, glyphs: f32) -> Self {
        self.visible_width = Some(glyphs);
        self
    }

    /// Makes it a text area: Enter starts a new line, long lines wrap, and the field is `lines` lines high
    /// (the text scrolls inside it).
    pub fn lines(mut self, lines: f32) -> Self {
        self.lines = Some(lines.max(1.0));
        self
    }

    /// The text can be selected and copied, not changed.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn build(self) -> impl Bundle {
        let (text, reported) = self.input_text();
        let node = self.frame_node();
        let Self {
            visible_width,
            lines,
            read_only,
            placeholder_extras,
            ..
        } = self;
        let mode = if read_only {
            TextReadWriteMode::ReadOnly
        } else {
            TextReadWriteMode::Editable
        };
        (
            TextField,
            node,
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                let mut input = parent.spawn((TextFieldInput, reported, text, mode));
                add_input_options(&mut input, visible_width, lines.is_some());
                let mut placeholder = parent.spawn((
                    TextFieldPlaceholder,
                    placeholder_node(&TextFieldMetrics::default()),
                    pass_through(),
                ));
                for extra in placeholder_extras {
                    extra(&mut placeholder);
                }
                if lines.is_some() {
                    parent.spawn(scroll_indicator());
                }
            })),
        )
    }

    /// The frame: as wide as the parent, or as wide as its input when `visible_width` is set.
    fn frame_node(&self) -> Node {
        let mut node = default_field_node();
        if self.visible_width.is_none() {
            node.width = percent(100);
        } else {
            // Not stretched across the parent: the input gives it its width.
            node.align_self = AlignSelf::FlexStart;
        }
        node
    }

    /// The editable text with its limits, and the text the app was last told about (the initial one).
    fn input_text(&self) -> (EditableText, ReportedText) {
        let initial = fit_text(&self.initial, self.lines.is_some(), self.max_characters);
        let mut text = EditableText::new(&initial);
        text.max_characters = self.max_characters;
        text.visible_width = self.visible_width;
        if let Some(lines) = self.lines {
            text.visible_lines = Some(lines);
            text.allow_newlines = true;
        }
        (text, ReportedText(initial))
    }
}

/// What the input has when it is sized by `visible_width` or is a text area.
fn add_input_options(input: &mut EntityWorldMut, visible_width: Option<f32>, area: bool) {
    if visible_width.is_some() {
        input.insert(sized_input_node());
    }
    if area {
        input.insert((
            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
            AccessibilityNode::from(AccessKitNode::new(Role::MultilineTextInput)),
        ));
    }
}
