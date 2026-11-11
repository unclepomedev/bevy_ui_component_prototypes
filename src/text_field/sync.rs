use super::{
    components::{TextField, TextFieldInput, field_node, input_in, placeholder_node},
    parts::{FieldParts, FieldRowItem, InputRow, Placeholders},
    style::TextFieldLook,
    targets::FieldTargets,
};
use crate::{
    Theme,
    common::{state::VisualState, theme_lookup::ThemeLookup},
};
use bevy::{
    input_focus::{AcquireFocus, FocusCause, InputFocus, InputFocusSystems},
    prelude::*,
    text::TextCursorStyle,
    ui::{InteractionDisabled, UiSystems},
};
use std::mem::replace as mem_replace;

pub(crate) struct TextFieldStylePlugin;
impl Plugin for TextFieldStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(focus_input_on_press)
            .add_systems(PostUpdate, sync_text_field.before(UiSystems::Prepare))
            .add_systems(
                PreUpdate,
                release_disabled_input.before(InputFocusSystems::Dispatch),
            );
    }
}

/// A press on the frame (the padding, the border) focuses the input inside it.
fn focus_input_on_press(
    mut acquire: On<AcquireFocus>,
    fields: Query<(&Children, Has<InteractionDisabled>), With<TextField>>,
    inputs: Query<(), With<TextFieldInput>>,
    mut focus: ResMut<InputFocus>,
) {
    let Ok((children, disabled)) = fields.get(acquire.focused_entity) else {
        return;
    };
    let Some(input) = input_in(children, &inputs) else {
        return;
    };
    if disabled {
        return;
    }
    acquire.propagate(false);
    if focus.get() != Some(input) {
        focus.set(input, FocusCause::Pressed);
    }
}

/// Takes the focus away from a disabled input.
fn release_disabled_input(
    focus: Option<ResMut<InputFocus>>,
    disabled: Query<(), (With<TextFieldInput>, With<InteractionDisabled>)>,
) {
    let Some(mut focus) = focus else {
        return;
    };
    // Bevy's input still takes the keys of a focused `Editable` input that is disabled.
    if focus.get().is_some_and(|entity| disabled.contains(entity)) {
        focus.clear();
    }
}

/// Rewrites the look of each text field that needs it.
fn sync_text_field(
    mut lookup: ThemeLookup,
    focus: Option<Res<InputFocus>>,
    mut last_focus: Local<Option<Entity>>,
    mut targets: FieldTargets,
    mut parts: FieldParts,
    mut commands: Commands,
) {
    let focused = focus.as_deref().and_then(InputFocus::get);
    let previous = mem_replace(&mut *last_focus, focused);
    let focus_moved: Vec<Entity> = if previous == focused {
        Vec::new()
    } else {
        previous.into_iter().chain(focused).collect()
    };

    for entity in targets.collect(lookup.needs_resync(), focus_moved) {
        let Ok(mut field) = parts.fields.get_mut(entity) else {
            continue;
        };
        let Ok(kids) = parts.children.get(entity) else {
            continue;
        };
        let theme = lookup.for_entity(entity);

        let inputs = sync_inputs(
            &mut parts.inputs,
            kids,
            &field,
            focused,
            theme,
            &mut commands,
        );
        write_frame(
            &mut field,
            &theme.text_field.look(inputs.state, inputs.ring_visible),
            theme,
        );
        sync_placeholders(
            &mut parts.placeholders,
            kids,
            inputs.empty && !field.disabled,
            theme,
        );
    }
}

/// What the inputs of a field decide about the rest of it.
struct InputsLook {
    state: VisualState,
    ring_visible: bool,
    empty: bool,
}

/// Writes the text, the cursor and the font of the input of a field, and makes the input disabled when the
/// field is. Returns the state the field is in.
fn sync_inputs(
    inputs: &mut Query<InputRow, (With<TextFieldInput>, Without<TextField>)>,
    kids: &Children,
    field: &FieldRowItem,
    focused: Option<Entity>,
    theme: &Theme,
    commands: &mut Commands,
) -> InputsLook {
    let style = &theme.text_field;
    let mut found = InputsLook {
        state: VisualState::from_flags(false, field.hovered.get(), false, field.disabled),
        ring_visible: false,
        empty: false,
    };
    for kid in kids {
        let Ok(mut input) = inputs.get_mut(*kid) else {
            continue;
        };
        found.state = VisualState::from_flags(
            false,
            field.hovered.get(),
            focused == Some(*kid),
            field.disabled,
        );
        found.ring_visible = input.shows_focus;
        found.empty = input.text.value().chars().next().is_none() && !input.text.is_composing();
        input
            .color
            .set_if_neq(TextColor(style.look(found.state, found.ring_visible).text));
        input.cursor.set_if_neq(TextCursorStyle {
            color: style.cursor,
            selection_color: style.selection,
            unfocused_selection_color: style.unfocused_selection,
            selected_text_color: None,
            selection_radius: 0.0,
        });
        write_font(&mut input.font, theme);
        // The input takes after its frame: both are disabled or neither. A disabled input is also out of
        // the picking, so a press lands on the frame, which does not focus it.
        if input.disabled != field.disabled {
            if field.disabled {
                commands
                    .entity(*kid)
                    .try_insert((InteractionDisabled, Pickable::IGNORE));
            } else {
                commands
                    .entity(*kid)
                    .try_remove::<(InteractionDisabled, Pickable)>();
            }
        }
    }
    found
}

/// Shows the placeholder while `shown`, and writes its color and font.
fn sync_placeholders(placeholders: &mut Placeholders, kids: &Children, shown: bool, theme: &Theme) {
    for kid in kids {
        let Ok(mut placeholder) = placeholders.get_mut(*kid) else {
            continue;
        };
        let mut node = placeholder_node(&theme.text_field_metrics);
        node.display = if shown { Display::Flex } else { Display::None };
        placeholder.node.set_if_neq(node);
        placeholder
            .color
            .set_if_neq(TextColor(theme.text_field.placeholder));
        write_font(&mut placeholder.font, theme);
    }
}

fn write_frame(field: &mut FieldRowItem, look: &TextFieldLook, theme: &Theme) {
    let (style, metrics) = (&theme.text_field, &theme.text_field_metrics);
    field.bg.set_if_neq(BackgroundColor(look.bg));
    field.border.set_if_neq(BorderColor::all(look.border));
    field.outline.set_if_neq(Outline::new(
        look.ring.width,
        look.ring.offset,
        look.ring.color,
    ));
    let frame = field_node(metrics, &style.shape);
    if field.node.border != frame.border {
        field.node.border = frame.border;
    }
    if field.node.border_radius != frame.border_radius {
        field.node.border_radius = frame.border_radius;
    }
    if field.node.padding != frame.padding {
        field.node.padding = frame.padding;
    }
}

fn write_font(font: &mut Mut<TextFont>, theme: &Theme) {
    if font.font != theme.font {
        font.font = theme.font.clone();
    }
    if font.font_smoothing != theme.font_smoothing {
        font.font_smoothing = theme.font_smoothing;
    }
    if font.font_size != theme.text_field_metrics.font_size {
        font.font_size = theme.text_field_metrics.font_size;
    }
}
