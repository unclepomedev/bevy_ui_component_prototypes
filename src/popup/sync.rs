use super::components::{
    PopupButton, PopupButtons, PopupMessage, PopupPanel, PopupRoot, popup_panel_node,
};
use super::style::PopupStyle;
use crate::{
    Theme,
    button::{ButtonStyles, ButtonVariant},
    common::{layout::write_changed, theme_lookup::ThemeLookup},
};
use bevy::prelude::*;

// Disjoint, because both write `BackgroundColor`.
type Roots<'w, 's> =
    Query<'w, 's, (Entity, &'static mut BackgroundColor), (With<PopupRoot>, Without<PopupPanel>)>;
type Panels<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
        &'static mut Node,
    ),
    With<PopupPanel>,
>;

/// Writes the look of each popup from its theme: the scrim, the panel, and the message.
pub(super) fn sync_popup_style(
    lookup: ThemeLookup,
    mut roots: Roots,
    mut panels: Panels,
    mut messages: Query<(Entity, &mut TextColor, &mut TextFont), With<PopupMessage>>,
) {
    for (entity, mut bg) in &mut roots {
        warn_if_unstyled(lookup.for_entity(entity));
        bg.set_if_neq(BackgroundColor(lookup.for_entity(entity).popup.scrim));
    }
    for (entity, mut bg, mut border, mut node) in &mut panels {
        let theme = lookup.for_entity(entity);
        let style = &theme.popup;
        bg.set_if_neq(BackgroundColor(style.bg));
        border.set_if_neq(BorderColor::all(style.border));
        write_layout(
            &mut node,
            &popup_panel_node(&theme.popup_metrics, &style.shape),
        );
    }
    for (entity, mut color, mut font) in &mut messages {
        let theme = lookup.for_entity(entity);
        color.set_if_neq(TextColor(theme.popup.fg));
        write_font(&mut font, theme);
    }
}

fn warn_if_unstyled(theme: &Theme) {
    let library = ButtonStyles::default();
    let restyled =
        theme.buttons.base.bg != library.base.bg || theme.buttons.base.fg != library.base.fg;
    if restyled && theme.popup == PopupStyle::default() {
        // A theme that restyles the buttons but leaves `Theme::popup` at the library's look is most likely one that forgot the popup.
        warn_once!(
            "A popup is shown with the library's default `Theme::popup`, while the theme restyles its buttons: set `Theme::popup` to match"
        );
    }
}

/// Writes the size, the spacing, the border, and the corners, only where they changed.
fn write_layout(node: &mut Mut<Node>, want: &Node) {
    write_changed!(
        node,
        want,
        min_width,
        max_width,
        padding,
        row_gap,
        border,
        border_radius
    );
}

fn write_font(font: &mut Mut<TextFont>, theme: &Theme) {
    if font.font != theme.font {
        font.font = theme.font.clone();
    }
    if font.font_smoothing != theme.font_smoothing {
        font.font_smoothing = theme.font_smoothing;
    }
    if font.font_size != theme.popup_metrics.font_size {
        font.font_size = theme.popup_metrics.font_size;
    }
}

// Disjoint, because both write `Node`.
type ButtonNodes<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static PopupButton,
        &'static mut Node,
        &'static mut ButtonVariant,
    ),
    Without<PopupButtons>,
>;

/// Writes the gap between the buttons, the width of each button, and the variant of each button
/// that has none of its own.
pub(super) fn sync_popup_buttons(
    lookup: ThemeLookup,
    mut rows: Query<(Entity, &mut Node), With<PopupButtons>>,
    mut buttons: ButtonNodes,
) {
    for (entity, mut node) in &mut rows {
        let gap = lookup.for_entity(entity).popup_metrics.button_gap;
        if node.column_gap != gap {
            node.column_gap = gap;
        }
    }
    for (entity, button, mut node, mut variant) in &mut buttons {
        let theme = lookup.for_entity(entity);
        let width = theme.popup_metrics.button_width;
        if node.width != width {
            node.width = width;
        }
        variant.set_if_neq(button.variant.unwrap_or(theme.popup.button));
    }
}
