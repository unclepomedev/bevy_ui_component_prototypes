use super::components::{ScrollIndicator, ScrollIndicatorThumb, thumb_node, track_node};
use crate::common::theme_lookup::ThemeLookup;
use bevy::{
    ecs::query::QueryData,
    prelude::*,
    ui::{ScrollPosition, UiSystems},
    ui_widgets::ScrollArea,
};

/// The shortest the thumb gets, so that it can still be seen and found when there is a lot to scroll.
const MIN_THUMB_LENGTH: f32 = 12.0;

/// Where the thumb is along a track of the length `track`, as `(top, length)`, for an area of the length
/// `visible` whose content of the length `content` is scrolled by `offset`. `None` when there is nothing
/// to scroll or no room for a track.
pub(crate) fn thumb(track: f32, visible: f32, content: f32, offset: f32) -> Option<(f32, f32)> {
    if !track.is_finite() || track <= 0.0 {
        return None;
    }
    if !(visible.is_finite() && content.is_finite()) || visible <= 0.0 || content <= visible {
        return None;
    }
    let length = (track * visible / content).clamp(MIN_THUMB_LENGTH.min(track), track);
    let range = content - visible;
    let share = if offset.is_finite() {
        (offset / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Some((share * (track - length), length))
}

pub(crate) struct ScrollIndicatorStylePlugin;
impl Plugin for ScrollIndicatorStylePlugin {
    fn build(&self, app: &mut App) {
        // Before the layout: the offset is the current one, and the sizes are those of the last layout.
        app.add_systems(PostUpdate, sync_scroll_indicator.before(UiSystems::Prepare));
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct TrackRow {
    parent: &'static ChildOf,
    children: &'static Children,
    node: &'static mut Node,
    bg: &'static mut BackgroundColor,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ThumbRow {
    node: &'static mut Node,
    bg: &'static mut BackgroundColor,
}

type Tracks<'w, 's> =
    Query<'w, 's, TrackRow, (With<ScrollIndicator>, Without<ScrollIndicatorThumb>)>;
type Thumbs<'w, 's> =
    Query<'w, 's, ThumbRow, (With<ScrollIndicatorThumb>, Without<ScrollIndicator>)>;
type Areas<'w, 's> =
    Query<'w, 's, (&'static ComputedNode, Option<&'static ScrollPosition>), With<ScrollArea>>;

/// Writes the look of each indicator, and puts its thumb where the scroll area it is next to has scrolled to.
fn sync_scroll_indicator(
    lookup: ThemeLookup,
    mut tracks: Tracks,
    mut thumbs: Thumbs,
    areas: Areas,
    siblings: Query<&Children>,
    track_ids: Query<Entity, With<ScrollIndicator>>,
) {
    for track_entity in &track_ids {
        let Ok(mut track) = tracks.get_mut(track_entity) else {
            continue;
        };
        let Some(area) = siblings
            .get(track.parent.parent())
            .ok()
            .and_then(|children| children.iter().find(|child| areas.contains(*child)))
        else {
            continue;
        };
        let theme = lookup.for_entity(track_entity);
        let (style, metrics) = (&theme.scroll_indicator, &theme.scroll_indicator_metrics);

        let placed = areas.get(area).ok().and_then(|(computed, scroll)| {
            let scale = computed.inverse_scale_factor;
            let visible = computed.size().y * scale;
            thumb(
                visible - 2.0 * metrics.inset,
                visible,
                computed.content_size().y * scale,
                scroll.map_or(0.0, |scroll| scroll.y),
            )
        });
        let mut wanted = track_node(metrics);
        wanted.display = if placed.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        track.bg.set_if_neq(BackgroundColor(style.track));
        track.node.set_if_neq(wanted);

        let Some(thumb_entity) = track.children.iter().find(|child| thumbs.contains(*child)) else {
            continue;
        };
        let Ok(mut thumb_row) = thumbs.get_mut(thumb_entity) else {
            continue;
        };
        thumb_row.bg.set_if_neq(BackgroundColor(style.thumb));
        let mut wanted = thumb_node(metrics);
        if let Some((top, length)) = placed {
            wanted.top = px(top);
            wanted.height = px(length);
        } else {
            wanted.top = thumb_row.node.top;
            wanted.height = thumb_row.node.height;
        }
        thumb_row.node.set_if_neq(wanted);
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_to_scroll_gives_no_thumb() {
        assert_eq!(thumb(100.0, 100.0, 100.0, 0.0), None);
        assert_eq!(thumb(100.0, 100.0, 50.0, 0.0), None);
        assert_eq!(thumb(100.0, 0.0, 50.0, 0.0), None);
    }

    #[test]
    fn a_track_without_room_gives_no_thumb() {
        assert_eq!(thumb(0.0, 100.0, 400.0, 0.0), None);
        assert_eq!(thumb(-8.0, 100.0, 400.0, 0.0), None);
    }

    #[test]
    fn the_thumb_is_at_the_top_at_the_start_and_at_the_bottom_at_the_end() {
        let (top, _) = thumb(100.0, 100.0, 400.0, 0.0).unwrap();
        assert_eq!(top, 0.0);
        let (top, length) = thumb(100.0, 100.0, 400.0, 300.0).unwrap();
        assert!((top + length - 100.0).abs() < 1e-4);
    }

    #[test]
    fn the_thumb_stays_inside_a_track_shorter_than_the_area() {
        // An area of 100 with an inset of 4 on both sides has a track of 92.
        let (top, length) = thumb(92.0, 100.0, 400.0, 300.0).unwrap();
        assert!((top + length - 92.0).abs() < 1e-4);
        assert_eq!(length, 23.0);
    }

    #[test]
    fn the_length_is_the_share_that_is_visible() {
        let (_, length) = thumb(100.0, 100.0, 400.0, 0.0).unwrap();
        assert_eq!(length, 25.0);
    }

    #[test]
    fn the_thumb_does_not_get_shorter_than_the_minimum_nor_longer_than_the_track() {
        let (_, length) = thumb(100.0, 100.0, 1_000_000.0, 0.0).unwrap();
        assert_eq!(length, MIN_THUMB_LENGTH);
        let (_, length) = thumb(5.0, 100.0, 1_000_000.0, 0.0).unwrap();
        assert_eq!(length, 5.0);
    }

    #[test]
    fn an_offset_out_of_range_keeps_the_thumb_on_the_track() {
        for offset in [-50.0, 10_000.0] {
            let (top, length) = thumb(92.0, 100.0, 400.0, offset).unwrap();
            assert!(top >= 0.0);
            assert!(top + length <= 92.0 + 1e-4);
        }
    }

    #[test]
    fn non_finite_input_does_not_panic() {
        let values = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1.0];
        for track in values {
            for visible in values {
                for content in values {
                    for offset in values {
                        if let Some((top, length)) = thumb(track, visible, content, offset) {
                            assert!(top.is_finite() && length.is_finite());
                        }
                    }
                }
            }
        }
    }
}
