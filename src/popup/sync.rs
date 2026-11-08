use super::components::{PopupMessage, PopupPanel, PopupRoot, popup_panel_node};
use crate::{Theme, common::theme_lookup::ThemeLookup};
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

/// Writes the size, the spacing, the border, and the corners, only where they changed.
fn write_layout(node: &mut Mut<Node>, want: &Node) {
    if node.min_width != want.min_width {
        node.min_width = want.min_width;
    }
    if node.max_width != want.max_width {
        node.max_width = want.max_width;
    }
    if node.padding != want.padding {
        node.padding = want.padding;
    }
    if node.row_gap != want.row_gap {
        node.row_gap = want.row_gap;
    }
    if node.border != want.border {
        node.border = want.border;
    }
    if node.border_radius != want.border_radius {
        node.border_radius = want.border_radius;
    }
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
