use super::style::ScrollIndicatorMetrics;
use crate::common::picking::pass_through;
use bevy::{
    ecs::spawn::{Spawn, SpawnRelated},
    picking::Pickable,
    prelude::*,
};

/// The mark at the right edge of a scroll area that shows where the visible part is among the content: a track,
/// and a thumb that moves along it.
///
/// Spawn it with [`scroll_indicator`] next to the scroll area (the sibling that has `ScrollArea`), in a node that
/// is as large as the area. It is shown only while there is something to scroll, and lets the pointer through.
#[derive(Component, Clone, Copy, Default)]
#[require(
    Pickable = pass_through(),
    BackgroundColor = BackgroundColor(Color::NONE),
    Node = track_node(&ScrollIndicatorMetrics::default())
)]
pub struct ScrollIndicator;

/// The thumb; it moves along the track.
#[derive(Component, Clone, Copy, Default)]
#[require(
    Pickable = pass_through(),
    BackgroundColor = BackgroundColor(Color::NONE),
    Node = thumb_node(&ScrollIndicatorMetrics::default())
)]
pub(crate) struct ScrollIndicatorThumb;

/// A track with its thumb.
pub fn scroll_indicator() -> impl Bundle {
    (
        ScrollIndicator,
        Children::spawn(Spawn(ScrollIndicatorThumb)),
    )
}

pub(crate) fn track_node(metrics: &ScrollIndicatorMetrics) -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: px(metrics.inset),
        bottom: px(metrics.inset),
        right: px(metrics.inset),
        width: px(metrics.width),
        border_radius: BorderRadius::all(px(metrics.width / 2.0)),
        // Shown only while there is something to scroll.
        display: Display::None,
        ..default()
    }
}

pub(crate) fn thumb_node(metrics: &ScrollIndicatorMetrics) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        width: percent(100),
        border_radius: BorderRadius::all(px(metrics.width / 2.0)),
        ..default()
    }
}
