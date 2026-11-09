use super::{
    components::{
        TextField, TextFieldInput, TextFieldPlaceholder, default_field_node, placeholder_node,
        sized_input_node,
    },
    entry::CharFilter,
    style::TextFieldMetrics,
    value::{ReportedText, fit_text},
};
use crate::{common::picking::pass_through, scroll_indicator::scroll_indicator};
use bevy::{
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
        align_self: None,
        lines: None,
        filter: None,
        read_only: false,
        placeholder_extras: Vec::new(),
    }
}

pub struct TextFieldBuilder {
    initial: String,
    max_characters: Option<usize>,
    visible_width: Option<f32>,
    align_self: Option<AlignSelf>,
    lines: Option<f32>,
    filter: Option<CharFilter>,
    read_only: bool,
    placeholder_extras: Vec<Extra>,
}

impl TextFieldBuilder {
    /// The text the field starts with, as the field takes it: through `char_filter`, cut at `max_characters`,
    /// and without line breaks unless `lines` is set.
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

    /// Keeps (`Some(c)`), replaces, or drops (`None`) each character that is typed, IME-committed,
    /// pasted, in `initial`, or set by `SetFieldText`. `max_characters` counts what is kept.
    pub fn char_filter(
        mut self,
        filter: impl Fn(char) -> Option<char> + Send + Sync + 'static,
    ) -> Self {
        self.filter = Some(CharFilter::new(filter));
        self
    }

    /// The width in `glyphs` instead of the width of the parent.
    pub fn visible_width(mut self, glyphs: f32) -> Self {
        self.visible_width = Some(glyphs);
        self
    }

    /// Sets the field's cross-axis alignment.
    /// Defaults to `FlexStart` with `visible_width`, otherwise inherits the parent's alignment.
    pub fn align_self(mut self, align: AlignSelf) -> Self {
        self.align_self = Some(align);
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
            filter,
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
                if let Some(filter) = filter {
                    input.insert(filter);
                }
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
        let align_self = self.align_self.filter(|align| *align != AlignSelf::Auto);
        if self.visible_width.is_none() {
            node.width = percent(100);
        } else if align_self.is_none() {
            // Not stretched across the parent: the input gives it its width.
            node.align_self = AlignSelf::FlexStart;
        }
        if let Some(align_self) = align_self {
            node.align_self = align_self;
        }
        node
    }

    /// The editable text with its limits, and the text the app was last told about (the initial one).
    fn input_text(&self) -> (EditableText, ReportedText) {
        let initial = fit_text(
            &self.initial,
            self.lines.is_some(),
            self.max_characters,
            self.filter.as_ref(),
        );
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
        input.insert(TextLayout::new(Justify::Left, LineBreak::WordBoundary));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_width_defaults_to_flex_start() {
        let node = text_field().visible_width(10.0).frame_node();

        assert_eq!(node.align_self, AlignSelf::FlexStart);
    }

    #[test]
    fn full_width_defaults_to_auto_alignment() {
        let node = text_field().frame_node();

        assert_eq!(node.align_self, AlignSelf::Auto);
        assert_eq!(node.width, percent(100));
    }

    #[test]
    fn explicit_alignment_overrides_visible_width_default() {
        let node = text_field()
            .visible_width(10.0)
            .align_self(AlignSelf::Center)
            .frame_node();

        assert_eq!(node.align_self, AlignSelf::Center);
    }

    #[test]
    fn auto_alignment_keeps_the_visible_width_default() {
        let node = text_field()
            .visible_width(10.0)
            .align_self(AlignSelf::Auto)
            .frame_node();

        assert_eq!(node.align_self, AlignSelf::FlexStart);
    }

    #[test]
    fn explicit_alignment_is_used_for_full_width_fields() {
        let node = text_field().align_self(AlignSelf::Center).frame_node();

        assert_eq!(node.align_self, AlignSelf::Center);
        assert_eq!(node.width, percent(100));
    }
}
