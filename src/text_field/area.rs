//! What a text area has over a single line: the mouse wheel scrolls it, and a scroll indicator shows where the
//! visible lines are among all of them.

use super::components::{TextField, TextFieldInput, input_in};
use crate::{
    Theme,
    common::theme_lookup::ThemeLookup,
    scroll_indicator::{ScrollIndicator, ScrollIndicatorThumb, thumb, thumb_node, track_node},
};
use bevy::{
    ecs::query::QueryData,
    input::mouse::MouseScrollPixelsPerLine,
    picking::events::PointerScroll,
    prelude::*,
    text::{EditableText, TextEdit, TextLayoutInfo},
    ui::UiSystems,
};

pub(crate) struct TextAreaPlugin;
impl Plugin for TextAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(scroll_on_wheel)
            .add_systems(PostUpdate, sync_area_indicator.before(UiSystems::Prepare));
    }
}

/// How far the text moves down for a wheel turn of `wheel` logical pixels (up is positive), in the
/// pixels of the layout.
fn wheel_displacement(wheel: f32, inverse_scale_factor: f32) -> f32 {
    if inverse_scale_factor > 0.0 && wheel.is_finite() {
        -wheel / inverse_scale_factor
    } else {
        0.0
    }
}

/// Whether the text can move by `displacement` (down is positive) from `offset`, when it can be scrolled by
/// `range` at the most.
fn can_scroll(displacement: f32, offset: f32, range: f32) -> bool {
    (displacement > 0.0 && offset < range) || (displacement < 0.0 && offset > 0.0)
}

/// The wheel over a text area scrolls its lines, as long as they can move that way. A single line, and
/// a wheel that has nowhere to scroll the text, let the wheel through.
fn scroll_on_wheel(
    mut scroll: On<PointerScroll>,
    fields: Query<&Children, With<TextField>>,
    mut inputs: Query<(&mut EditableText, &TextLayoutInfo, &ComputedNode), With<TextFieldInput>>,
    ratio: Res<MouseScrollPixelsPerLine>,
) {
    let target = scroll.entity;
    let input = match fields.get(target) {
        Ok(children) => input_in(children, &inputs),
        Err(_) => inputs.contains(target).then_some(target),
    };
    let Some(input) = input else {
        return;
    };
    let Ok((mut text, info, computed)) = inputs.get_mut(input) else {
        return;
    };
    if !text.allow_newlines {
        return;
    }
    let displacement =
        wheel_displacement(scroll.to_pixels(&ratio).y, computed.inverse_scale_factor);
    let range = (info.size.y - text.viewport.size.y).max(0.0);
    if can_scroll(displacement, text.viewport.offset.y, range) {
        scroll.propagate(false);
        text.queue_edit(TextEdit::ScrollBy(Vec2::new(0.0, displacement)));
    }
}

/// The track of a text area, with the scroll indicator's parts.
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

type Tracks<'w, 's> = Query<
    'w,
    's,
    TrackRow,
    (
        With<ScrollIndicator>,
        Without<TextField>,
        Without<ScrollIndicatorThumb>,
    ),
>;
type Thumbs<'w, 's> =
    Query<'w, 's, ThumbRow, (With<ScrollIndicatorThumb>, Without<ScrollIndicator>)>;
type Inputs<'w, 's> = Query<
    'w,
    's,
    (
        &'static EditableText,
        &'static TextLayoutInfo,
        &'static ComputedNode,
    ),
    With<TextFieldInput>,
>;

/// Puts the track and the thumb of each text area where its visible lines are.
fn sync_area_indicator(
    lookup: ThemeLookup,
    fields: Query<(&Children, &ComputedNode), With<TextField>>,
    inputs: Inputs,
    mut tracks: Tracks,
    mut thumbs: Thumbs,
) {
    for mut track in &mut tracks {
        let field = track.parent.parent();
        let Ok((children, field_size)) = fields.get(field) else {
            continue;
        };
        let Some(input) = input_in(children, &inputs).and_then(|input| inputs.get(input).ok())
        else {
            continue;
        };
        let theme = lookup.for_entity(field);

        let placed = thumb_of(input, field_size, theme);
        write_track(&mut track, placed.is_some(), theme);
        let thumb = track.children.iter().find(|kid| thumbs.contains(*kid));
        if let Some(mut thumb) = thumb.and_then(|kid| thumbs.get_mut(kid).ok()) {
            write_thumb(&mut thumb, placed, theme);
        }
    }
}

/// Where the thumb goes on the track of a field as `(top, length)` in logical pixels; `None` while
/// all the lines are shown. The track is inside the border and the inset, along the height of the field.
fn thumb_of(
    (text, info, input_size): (&EditableText, &TextLayoutInfo, &ComputedNode),
    field_size: &ComputedNode,
    theme: &Theme,
) -> Option<(f32, f32)> {
    let scale = input_size.inverse_scale_factor;
    let edge = theme.text_field.shape.border_width + theme.scroll_indicator_metrics.inset;
    let track = field_size.size().y * field_size.inverse_scale_factor - 2.0 * edge;
    thumb(
        track,
        text.viewport.size.y * scale,
        info.size.y * scale,
        text.viewport.offset.y * scale,
    )
}

/// The track is shown while the thumb has a place.
fn write_track(track: &mut TrackRowItem, shown: bool, theme: &Theme) {
    let mut wanted = track_node(&theme.scroll_indicator_metrics);
    wanted.display = if shown { Display::Flex } else { Display::None };
    track.node.set_if_neq(wanted);
    track
        .bg
        .set_if_neq(BackgroundColor(theme.scroll_indicator.track));
}

/// A thumb that has no place keeps the one it had; it is hidden with its track.
fn write_thumb(thumb: &mut ThumbRowItem, placed: Option<(f32, f32)>, theme: &Theme) {
    let mut wanted = thumb_node(&theme.scroll_indicator_metrics);
    match placed {
        Some((top, length)) => {
            wanted.top = px(top);
            wanted.height = px(length);
        }
        None => {
            wanted.top = thumb.node.top;
            wanted.height = thumb.node.height;
        }
    }
    thumb.node.set_if_neq(wanted);
    thumb
        .bg
        .set_if_neq(BackgroundColor(theme.scroll_indicator.thumb));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_scrolls_only_where_it_can_move() {
        assert!(can_scroll(10.0, 0.0, 50.0));
        assert!(can_scroll(-10.0, 20.0, 50.0));
        assert!(!can_scroll(10.0, 50.0, 50.0));
        assert!(!can_scroll(-10.0, 0.0, 50.0));
        assert!(!can_scroll(10.0, 0.0, 0.0));
        assert!(!can_scroll(0.0, 20.0, 50.0));
    }

    #[test]
    fn the_wheel_moves_the_text_the_other_way() {
        assert_eq!(wheel_displacement(30.0, 1.0), -30.0);
        assert_eq!(wheel_displacement(-30.0, 0.5), 60.0);
        assert_eq!(wheel_displacement(f32::NAN, 1.0), 0.0);
        assert_eq!(wheel_displacement(30.0, 0.0), 0.0);
    }
}
