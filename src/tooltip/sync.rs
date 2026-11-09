use super::components::{Tooltip, TooltipLabel, TooltipPopup, placement, popup_node};
use crate::common::{layer::layer_above, theme_lookup::ThemeLookup};
use bevy::{ecs::query::QueryData, prelude::*, ui_widgets::popover::Popover};

/// Gives the box of each tooltip on screen its colors, its shape and its place.
pub(super) fn sync_popup_boxes(
    lookup: ThemeLookup,
    tooltips: Query<&Tooltip>,
    mut popups: Query<PopupBox>,
) {
    for mut row in &mut popups {
        let Ok(tooltip) = tooltips.get(row.popup.target) else {
            continue;
        };
        let theme = lookup.for_entity(row.popup.target);
        let style = theme.tooltip;
        let metrics = theme.tooltip_metrics;

        row.popover.set_if_neq(placement(tooltip.side, &metrics));
        row.bg.set_if_neq(BackgroundColor(style.bg));
        row.border.set_if_neq(BorderColor::all(style.border));
        write_layout(&mut row.node, &popup_node(&metrics, &style.shape));
    }
}

/// Puts each tooltip on screen `layer_gap` above the nearest layer that holds the widget it is for.
pub(super) fn sync_popup_layers(
    lookup: ThemeLookup,
    layers: Query<&GlobalZIndex, Without<TooltipPopup>>,
    parents: Query<&ChildOf>,
    mut popups: Query<(Entity, &TooltipPopup, &mut GlobalZIndex)>,
) {
    for (entity, popup, mut z) in &mut popups {
        let metrics = lookup.for_entity(popup.target).tooltip_metrics;
        let ancestor = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| layers.get(ancestor).ok())
            .map(|layer| layer.0);
        z.set_if_neq(GlobalZIndex(layer_above(ancestor, metrics.layer_gap)));
    }
}

/// Gives the text inside each tooltip on screen its words and its font.
pub(super) fn sync_popup_labels(
    lookup: ThemeLookup,
    tooltips: Query<&Tooltip>,
    popups: Query<&TooltipPopup>,
    mut labels: Query<(&ChildOf, &mut Text, &mut TextColor, &mut TextFont), With<TooltipLabel>>,
) {
    for (child_of, mut text, mut color, mut font) in &mut labels {
        let Ok(popup) = popups.get(child_of.parent()) else {
            continue;
        };
        let Ok(tooltip) = tooltips.get(popup.target) else {
            continue;
        };
        let theme = lookup.for_entity(popup.target);

        if !tooltip.has_bundle() && text.0 != tooltip.text {
            text.0.clone_from(&tooltip.text);
        }
        color.set_if_neq(TextColor(theme.tooltip.fg));
        if font.font != theme.font {
            font.font = theme.font.clone();
        }
        if font.font_smoothing != theme.font_smoothing {
            font.font_smoothing = theme.font_smoothing;
        }
        if font.font_size != theme.tooltip_metrics.font_size {
            font.font_size = theme.tooltip_metrics.font_size;
        }
    }
}

/// Writes the size, the border, and the corners, only where they changed.
fn write_layout(node: &mut Mut<Node>, want: &Node) {
    if node.max_width != want.max_width {
        node.max_width = want.max_width;
    }
    if node.padding != want.padding {
        node.padding = want.padding;
    }
    if node.border != want.border {
        node.border = want.border;
    }
    if node.border_radius != want.border_radius {
        node.border_radius = want.border_radius;
    }
}

/// What the sync reads from the box of a tooltip and writes to it.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct PopupBox {
    popup: &'static TooltipPopup,
    popover: &'static mut Popover,
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    node: &'static mut Node,
}
