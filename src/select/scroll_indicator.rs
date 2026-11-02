use super::components::{
    SelectList, SelectScrollThumb, SelectScrollTrack, scroll_thumb_node, scroll_track_node,
};
use crate::common::theme_lookup::ThemeLookup;
use bevy::{
    ecs::query::QueryData,
    prelude::*,
    ui::{ScrollPosition, UiSystems},
};

/// The shortest the thumb gets, so that it can still be seen and found when there is a lot to scroll.
const MIN_THUMB_LENGTH: f32 = 12.0;

/// Where the thumb is along a track of the length `visible`, as `(top, length)`, for content of the length
/// `content` scrolled by `offset`. `None` when there is nothing to scroll.
pub(crate) fn thumb(visible: f32, content: f32, offset: f32) -> Option<(f32, f32)> {
    if !(visible.is_finite() && content.is_finite()) || visible <= 0.0 || content <= visible {
        return None;
    }
    let length = (visible * visible / content).clamp(MIN_THUMB_LENGTH.min(visible), visible);
    let range = content - visible;
    let share = if offset.is_finite() {
        (offset / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Some((share * (visible - length), length))
}

pub(super) struct SelectScrollIndicatorPlugin;
impl Plugin for SelectScrollIndicatorPlugin {
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
    Query<'w, 's, TrackRow, (With<SelectScrollTrack>, Without<SelectScrollThumb>)>;
type Thumbs<'w, 's> =
    Query<'w, 's, ThumbRow, (With<SelectScrollThumb>, Without<SelectScrollTrack>)>;
type Lists<'w, 's> =
    Query<'w, 's, (&'static ComputedNode, Option<&'static ScrollPosition>), With<SelectList>>;

fn sync_scroll_indicator(
    lookup: ThemeLookup,
    mut tracks: Tracks,
    mut thumbs: Thumbs,
    lists: Lists,
    siblings: Query<&Children>,
    track_ids: Query<Entity, With<SelectScrollTrack>>,
) {
    for track_entity in &track_ids {
        let Ok(mut track) = tracks.get_mut(track_entity) else {
            continue;
        };
        let Some(list) = siblings
            .get(track.parent.parent())
            .ok()
            .and_then(|children| children.iter().find(|child| lists.contains(*child)))
        else {
            continue;
        };
        let theme = lookup.for_entity(track_entity);
        let style = &theme.select;
        let wanted = scroll_track_node(&theme.select_metrics);
        track.bg.set_if_neq(BackgroundColor(style.scroll_track));
        if track.node.top != wanted.top {
            track.node.top = wanted.top;
        }
        if track.node.bottom != wanted.bottom {
            track.node.bottom = wanted.bottom;
        }
        if track.node.right != wanted.right {
            track.node.right = wanted.right;
        }
        if track.node.width != wanted.width {
            track.node.width = wanted.width;
        }
        if track.node.border_radius != wanted.border_radius {
            track.node.border_radius = wanted.border_radius;
        }

        let thumb_node = scroll_thumb_node(&theme.select_metrics);
        let thumb_entity = track.children.iter().find(|child| thumbs.contains(*child));
        let placed = lists.get(list).ok().and_then(|(computed, scroll)| {
            let scale = computed.inverse_scale_factor;
            thumb(
                computed.size().y * scale,
                computed.content_size().y * scale,
                scroll.map_or(0.0, |scroll| scroll.y),
            )
        });
        let display = if placed.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if track.node.display != display {
            track.node.display = display;
        }
        let Some(thumb_entity) = thumb_entity else {
            continue;
        };
        let Ok(mut thumb_row) = thumbs.get_mut(thumb_entity) else {
            continue;
        };
        thumb_row.bg.set_if_neq(BackgroundColor(style.scroll_thumb));
        if thumb_row.node.border_radius != thumb_node.border_radius {
            thumb_row.node.border_radius = thumb_node.border_radius;
        }
        if let Some((top, length)) = placed {
            if thumb_row.node.top != px(top) {
                thumb_row.node.top = px(top);
            }
            if thumb_row.node.height != px(length) {
                thumb_row.node.height = px(length);
            }
        }
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
        assert_eq!(thumb(100.0, 100.0, 0.0), None);
        assert_eq!(thumb(100.0, 50.0, 0.0), None);
        assert_eq!(thumb(0.0, 50.0, 0.0), None);
    }

    #[test]
    fn the_thumb_is_at_the_top_at_the_start_and_at_the_bottom_at_the_end() {
        let (top, _) = thumb(100.0, 400.0, 0.0).unwrap();
        assert_eq!(top, 0.0);
        let (top, length) = thumb(100.0, 400.0, 300.0).unwrap();
        assert!((top + length - 100.0).abs() < 1e-4);
    }

    #[test]
    fn the_length_is_the_share_that_is_visible() {
        let (_, length) = thumb(100.0, 400.0, 0.0).unwrap();
        assert_eq!(length, 25.0);
    }

    #[test]
    fn the_thumb_does_not_get_shorter_than_the_minimum_nor_longer_than_the_track() {
        let (_, length) = thumb(100.0, 1_000_000.0, 0.0).unwrap();
        assert_eq!(length, MIN_THUMB_LENGTH);
        let (_, length) = thumb(5.0, 1_000_000.0, 0.0).unwrap();
        assert_eq!(length, 5.0);
    }

    #[test]
    fn an_offset_out_of_range_keeps_the_thumb_on_the_track() {
        for offset in [-50.0, 10_000.0] {
            let (top, length) = thumb(100.0, 400.0, offset).unwrap();
            assert!(top >= 0.0);
            assert!(top + length <= 100.0 + 1e-4);
        }
    }

    #[test]
    fn non_finite_input_does_not_panic() {
        let values = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1.0];
        for visible in values {
            for content in values {
                for offset in values {
                    if let Some((top, length)) = thumb(visible, content, offset) {
                        assert!(top.is_finite() && length.is_finite());
                    }
                }
            }
        }
    }
}
