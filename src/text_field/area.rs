//! What a text area has over a single line: the mouse wheel scrolls it, and a scroll indicator shows where the
//! visible lines are among all of them.

use super::components::{TextField, TextFieldInput, input_in};
use crate::{
    common::theme_lookup::ThemeLookup,
    scroll_indicator::{ScrollIndicator, ScrollIndicatorThumb, thumb, thumb_node, track_node},
};
use bevy::{
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

/// The wheel over a text area scrolls its lines. A single line lets the wheel through.
fn scroll_on_wheel(
    mut scroll: On<PointerScroll>,
    fields: Query<&Children, With<TextField>>,
    mut inputs: Query<(&mut EditableText, &ComputedNode), With<TextFieldInput>>,
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
    let Ok((mut text, computed)) = inputs.get_mut(input) else {
        return;
    };
    if !text.allow_newlines {
        return;
    }
    scroll.propagate(false);
    let displacement =
        wheel_displacement(scroll.to_pixels(&ratio).y, computed.inverse_scale_factor);
    if displacement != 0.0 {
        text.queue_edit(TextEdit::ScrollBy(Vec2::new(0.0, displacement)));
    }
}

type Tracks<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static ChildOf, &'static Children),
    (With<ScrollIndicator>, Without<TextField>),
>;

/// Puts the thumb of each text area where its visible lines are.
fn sync_area_indicator(
    lookup: ThemeLookup,
    fields: Query<(&Children, &ComputedNode), With<TextField>>,
    inputs: Query<(&EditableText, &TextLayoutInfo, &ComputedNode), With<TextFieldInput>>,
    tracks: Tracks,
    mut nodes: Query<&mut Node>,
    mut colors: Query<&mut BackgroundColor>,
    thumbs: Query<(), With<ScrollIndicatorThumb>>,
) {
    for (track, parent, kids) in &tracks {
        let Ok((children, field_computed)) = fields.get(parent.parent()) else {
            continue;
        };
        let Some((text, info, input_computed)) =
            children.iter().find_map(|child| inputs.get(child).ok())
        else {
            continue;
        };
        let theme = lookup.for_entity(parent.parent());
        let (style, metrics) = (&theme.scroll_indicator, &theme.scroll_indicator_metrics);

        let input_scale = input_computed.inverse_scale_factor;
        let field_scale = field_computed.inverse_scale_factor;
        let border = theme.text_field.shape.border_width;
        let track_length = field_computed.size().y * field_scale - 2.0 * (border + metrics.inset);
        let placed = thumb(
            track_length,
            text.viewport.size.y * input_scale,
            info.size.y * input_scale,
            text.viewport.offset.y * input_scale,
        );

        let mut wanted = track_node(metrics);
        // The track is in the padding box, inside the border.
        wanted.top = px(metrics.inset);
        wanted.bottom = px(metrics.inset);
        wanted.display = if placed.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Ok(mut node) = nodes.get_mut(track) {
            node.set_if_neq(wanted);
        }
        if let Ok(mut bg) = colors.get_mut(track) {
            bg.set_if_neq(BackgroundColor(style.track));
        }
        let Some(thumb) = kids.iter().find(|kid| thumbs.contains(*kid)) else {
            continue;
        };
        if let Ok(mut bg) = colors.get_mut(thumb) {
            bg.set_if_neq(BackgroundColor(style.thumb));
        }
        if let Ok(mut node) = nodes.get_mut(thumb) {
            let mut wanted = thumb_node(metrics);
            if let Some((top, length)) = placed {
                wanted.top = px(top);
                wanted.height = px(length);
            } else {
                wanted.top = node.top;
                wanted.height = node.height;
            }
            node.set_if_neq(wanted);
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_moves_the_text_the_other_way() {
        assert_eq!(wheel_displacement(30.0, 1.0), -30.0);
        assert_eq!(wheel_displacement(-30.0, 0.5), 60.0);
        assert_eq!(wheel_displacement(f32::NAN, 1.0), 0.0);
        assert_eq!(wheel_displacement(30.0, 0.0), 0.0);
    }
}
