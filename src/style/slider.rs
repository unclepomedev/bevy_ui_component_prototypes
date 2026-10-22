use super::{
    button::FocusRing,
    color::{
        AFFIRM_HOVERED, AFFIRM_NORMAL, AFFIRM_PRESSED, BG_HOVERED, BG_PRESSED, DISABLED, EDITING,
        FOCUS_RING, TEXT_MAIN, TEXT_MUTED,
    },
    state::{StateColors, VisualState},
};
use bevy::{prelude::*, ui::ColorStop};

/// A slider drawn as separate blocks instead of one continuous bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderSegments {
    /// How many blocks the bar is made of. Fewer than two draw a plain bar.
    pub count: u32,
    /// The empty space between two blocks, as a share of one block's pitches (`0.2` a fifth of each pitch is a gap).
    pub gap: f32,
}

impl Default for SliderSegments {
    fn default() -> Self {
        Self {
            count: 10,
            gap: 0.2,
        }
    }
}

/// The size and the corners of a slider, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderShape {
    /// Thickness of the rail and of the fill.
    pub rail_height: f32,
    /// Width and height of the thumb. `0.0` draws no thumb: the fill then ends exactly at the value.
    /// Ignored when `segments` is set: a segmented slider has no thumb.
    pub thumb_size: f32,
    /// Fully rounded ends (a pill and a circle) when `true`, square corners when `false`.
    pub rounded: bool,
    /// Draws the rail and the fill as blocks. A block is lit once the value has reached its end.
    pub segments: Option<SliderSegments>,
}

impl Default for SliderShape {
    fn default() -> Self {
        Self {
            rail_height: 6.0,
            thumb_size: 20.0,
            rounded: true,
            segments: None,
        }
    }
}

/// Where the parts of a slider go, resolved from a [`SliderShape`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SliderGeometry {
    pub(crate) height: f32,
    pub(crate) rail_height: f32,
    pub(crate) rail_top: f32,
    pub(crate) rail_radius: f32,
    pub(crate) thumb_size: f32,
    pub(crate) thumb_top: f32,
    pub(crate) thumb_radius: f32,
    /// How much shorter than the slider the travel area of the thumb is.
    pub(crate) travel_inset: f32,
    pub(crate) segments: Option<SliderSegments>,
}

impl SliderShape {
    pub(crate) fn geometry(&self) -> SliderGeometry {
        // Fewer than two blocks is just a plain bar.
        let segments = self.segments.filter(|segments| segments.count >= 2);
        let thumb_size = if segments.is_some() {
            0.0
        } else {
            self.thumb_size
        };
        let height = self.rail_height.max(thumb_size);
        let radius = |size: f32| if self.rounded { size / 2.0 } else { 0.0 };
        SliderGeometry {
            height,
            rail_height: self.rail_height,
            rail_top: (height - self.rail_height) / 2.0,
            rail_radius: radius(self.rail_height),
            thumb_size,
            thumb_top: (height - thumb_size) / 2.0,
            thumb_radius: radius(thumb_size),
            travel_inset: thumb_size,
            segments,
        }
    }
}

impl SliderSegments {
    /// How many blocks are lit at `share` (0 to 1) of the travel. A block is lit once the value has reached its end.
    /// A share worked out from a value and a range is not exact, so the small margin keeps such a block lit.
    pub(crate) fn lit(&self, share: f32) -> u32 {
        let lit = (share.clamp(0.0, 1.0) * self.count as f32 + 1e-4).floor();
        (lit as u32).min(self.count)
    }

    /// Color stops of a left-to-right gradient that paints the first `lit` blocks with `color` and
    /// leaves the other blocks and every gap transparent.
    pub(crate) fn stops(&self, lit: u32, color: Color) -> Vec<ColorStop> {
        // `count` blocks and `count - 1` gaps fill the bar from 0% to 100%.
        let gap = self.gap.clamp(0.0, 1.0);
        let pitch = 100.0 / (self.count as f32 - gap);
        let block = pitch * (1.0 - gap);
        // Each boundary is computed once, so a block's end never passes the next block's start.
        let at = |index: u32| pitch * index as f32;
        let mut stops = Vec::new();
        for index in 0..self.count {
            let start = at(index);
            let end = (start + block).min(at(index + 1));
            let paint = if index < lit { color } else { Color::NONE };
            stops.push(ColorStop::percent(paint, start));
            stops.push(ColorStop::percent(paint, end));
            if gap > 0.0 && index + 1 < self.count {
                stops.push(ColorStop::percent(Color::NONE, end));
                stops.push(ColorStop::percent(Color::NONE, at(index + 1)));
            }
        }
        stops
    }
}

/// Colors, focus ring and shape of a slider: the rail, the filled part of it, and the thumb.
#[derive(Clone, Copy, Debug)]
pub struct SliderStyle {
    pub shape: SliderShape,
    pub rail: StateColors,
    pub fill: StateColors,
    pub thumb: StateColors,
    pub focus_ring: FocusRing,
    pub editing_ring: Color,
    pub editing_thumb: Color,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            shape: SliderShape::default(),
            rail: StateColors {
                normal: BG_HOVERED,
                hovered: BG_HOVERED,
                pressed: BG_HOVERED,
                disabled: DISABLED,
                focused: BG_HOVERED,
            },
            fill: StateColors {
                normal: AFFIRM_NORMAL,
                hovered: AFFIRM_HOVERED,
                pressed: AFFIRM_PRESSED,
                disabled: BG_PRESSED,
                focused: AFFIRM_NORMAL,
            },
            thumb: StateColors {
                normal: TEXT_MAIN,
                hovered: TEXT_MAIN,
                pressed: TEXT_MAIN,
                disabled: TEXT_MUTED,
                focused: TEXT_MAIN,
            },
            focus_ring: FocusRing {
                color: FOCUS_RING,
                width: Val::Px(2.0),
                offset: Val::Px(2.0),
            },
            editing_ring: EDITING,
            editing_thumb: EDITING,
        }
    }
}

/// What a slider looks like in one state, resolved from a [`SliderStyle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SliderLook {
    pub(crate) rail: Color,
    pub(crate) fill: Color,
    pub(crate) thumb: Color,
    pub(crate) ring: FocusRing,
}

impl SliderStyle {
    /// `focused` is whether this slider owns the visible focus indicator. A disabled slider ignores `editing`.
    pub(crate) fn look(&self, state: VisualState, focused: bool, editing: bool) -> SliderLook {
        let editing = editing && state != VisualState::Disabled;
        SliderLook {
            rail: self.rail.pick(state),
            fill: self.fill.pick(state),
            thumb: if editing {
                self.editing_thumb
            } else {
                self.thumb.pick(state)
            },
            ring: if !focused || state == VisualState::Disabled {
                FocusRing::NONE
            } else if editing {
                FocusRing {
                    color: self.editing_ring,
                    ..self.focus_ring
                }
            } else {
                self.focus_ring
            },
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn percent(stop: &ColorStop) -> f32 {
        match stop.point {
            Val::Percent(value) => value,
            other => panic!("not a percent: {other:?}"),
        }
    }

    #[test]
    fn a_block_is_lit_once_the_value_reaches_its_end() {
        let segments = SliderSegments {
            count: 10,
            gap: 0.2,
        };
        assert_eq!(segments.lit(0.0), 0);
        assert_eq!(segments.lit(0.05), 0);
        assert_eq!(segments.lit(0.79), 7);
        assert_eq!(segments.lit(1.0), 10);
        assert_eq!(segments.lit(2.0), 10);
        assert_eq!(segments.lit(-1.0), 0);
    }

    // f32 rounding must not drop a block whose end the value has reached.
    #[test]
    fn rounding_error_does_not_drop_a_lit_block() {
        for count in 1..=40u32 {
            let segments = SliderSegments { count, gap: 0.0 };
            for lit in 0..=count {
                let value = lit as f32 * 100.0 / count as f32;
                assert_eq!(segments.lit(value / 100.0), lit, "{lit} of {count}");
            }
        }
    }

    #[test]
    fn the_stops_paint_lit_blocks_and_leave_the_gaps_clear() {
        let color = Color::srgb(0.2, 0.9, 0.3);
        // Three blocks of 20% and two gaps of 20%: 0-20, 40-60 and 80-100.
        let stops = SliderSegments { count: 3, gap: 0.5 }.stops(2, color);

        let seen: Vec<(Color, f32)> = stops.iter().map(|s| (s.color, percent(s))).collect();
        assert_eq!(
            seen,
            vec![
                (color, 0.0),
                (color, 20.0),
                (Color::NONE, 20.0),
                (Color::NONE, 40.0),
                (color, 40.0),
                (color, 60.0),
                (Color::NONE, 60.0),
                (Color::NONE, 80.0),
                (Color::NONE, 80.0),
                (Color::NONE, 100.0),
            ]
        );
    }

    // The bar has to end at the right edge, with no gap left over after the last block.
    #[test]
    fn the_last_block_ends_at_the_right_edge() {
        for count in [2, 3, 10] {
            for gap in [0.0, 0.2, 0.8] {
                let stops = SliderSegments { count, gap }.stops(count, Color::WHITE);
                let last = percent(stops.last().unwrap());
                assert!(
                    (last - 100.0).abs() < 1e-3,
                    "{count} blocks, gap {gap}: {last}"
                );
                assert_eq!(percent(&stops[0]), 0.0);
            }
        }
    }

    #[test]
    fn fewer_than_two_blocks_is_a_plain_bar() {
        for count in [0, 1] {
            let geometry = SliderShape {
                segments: Some(SliderSegments { count, gap: 0.2 }),
                ..default()
            }
            .geometry();
            assert_eq!(geometry.segments, None);
            // The thumb stays: this is the plain slider.
            assert_eq!(geometry.thumb_size, 20.0);
        }
    }

    // A gap of 1.0 or more must not divide by zero or produce a NaN.
    #[test]
    fn a_whole_gap_still_gives_finite_stops() {
        for gap in [1.0, 2.0] {
            let stops = SliderSegments { count: 2, gap }.stops(2, Color::WHITE);
            assert!(stops.iter().all(|stop| percent(stop).is_finite()));
        }
    }

    #[test]
    fn stop_positions_never_go_backwards() {
        // A backwards step is drawn as a blend across the block, not as a hard edge.
        for count in 2..=40 {
            for gap in [0.0, 0.1, 0.2, 0.5, 0.8, 1.0] {
                let stops = SliderSegments { count, gap }.stops(count, Color::WHITE);
                let mut previous = 0.0;
                for stop in &stops {
                    let at = percent(stop);
                    assert!(
                        at >= previous,
                        "count {count} gap {gap}: {at} after {previous}"
                    );
                    previous = at;
                }
            }
        }
    }

    #[test]
    fn without_a_gap_the_blocks_touch() {
        let color = Color::srgb(0.2, 0.9, 0.3);
        let stops = SliderSegments { count: 2, gap: 0.0 }.stops(2, color);

        let seen: Vec<(Color, f32)> = stops.iter().map(|s| (s.color, percent(s))).collect();
        assert_eq!(
            seen,
            vec![(color, 0.0), (color, 50.0), (color, 50.0), (color, 100.0)]
        );
    }

    // A thumb would shorten the travel and push the lit blocks out of line with the rail's.
    #[test]
    fn a_segmented_slider_has_no_thumb() {
        let geometry = SliderShape {
            segments: Some(SliderSegments::default()),
            ..default()
        }
        .geometry();
        assert_eq!(geometry.thumb_size, 0.0);
        assert_eq!(geometry.travel_inset, 0.0);
        assert_eq!(geometry.height, 6.0);
    }

    #[test]
    fn the_default_shape_is_a_thin_rail_under_a_round_thumb() {
        let geometry = SliderShape::default().geometry();
        assert_eq!(geometry.height, 20.0);
        assert_eq!(geometry.rail_top, 7.0);
        assert_eq!(geometry.rail_radius, 3.0);
        assert_eq!(geometry.thumb_top, 0.0);
        assert_eq!(geometry.thumb_radius, 10.0);
        assert_eq!(geometry.travel_inset, 20.0);
    }

    // The slider is as tall as its tallest part, and the smaller part is centered in it.
    #[test]
    fn the_slider_is_as_tall_as_its_tallest_part_and_the_other_is_centered() {
        let thick_rail = SliderShape {
            rail_height: 16.0,
            thumb_size: 8.0,
            rounded: true,
            ..default()
        }
        .geometry();
        assert_eq!(thick_rail.height, 16.0);
        assert_eq!(thick_rail.rail_top, 0.0);
        assert_eq!(thick_rail.thumb_top, 4.0);
    }

    #[test]
    fn square_corners_have_no_radius() {
        let geometry = SliderShape {
            rounded: false,
            ..default()
        }
        .geometry();
        assert_eq!(geometry.rail_radius, 0.0);
        assert_eq!(geometry.thumb_radius, 0.0);
    }

    #[test]
    fn a_zero_thumb_leaves_the_whole_width_to_travel() {
        let geometry = SliderShape {
            thumb_size: 0.0,
            ..default()
        }
        .geometry();
        assert_eq!(geometry.travel_inset, 0.0);
        assert_eq!(geometry.height, 6.0);
    }

    #[test]
    fn each_part_follows_the_visual_state() {
        let style = SliderStyle::default();
        let look = style.look(VisualState::Pressed, false, false);
        assert_eq!(look.rail, style.rail.pressed);
        assert_eq!(look.fill, style.fill.pressed);
        assert_eq!(look.thumb, style.thumb.pressed);
    }

    #[test]
    fn the_fill_is_told_apart_from_the_rail_by_default() {
        let style = SliderStyle::default();
        assert_ne!(style.fill.normal, style.rail.normal);
    }

    #[test]
    fn ring_follows_focus_but_never_shows_on_a_disabled_slider() {
        let style = SliderStyle::default();
        assert_eq!(
            style.look(VisualState::Normal, true, false).ring,
            style.focus_ring
        );
        assert_eq!(
            style.look(VisualState::Normal, false, false).ring,
            FocusRing::NONE
        );
        assert_eq!(
            style.look(VisualState::Disabled, true, false).ring,
            FocusRing::NONE
        );
    }

    #[test]
    fn editing_changes_the_thumb_and_the_ring_color_only() {
        let style = SliderStyle::default();
        let idle = style.look(VisualState::Focused, true, false);
        let editing = style.look(VisualState::Focused, true, true);
        assert_eq!(editing.thumb, style.editing_thumb);
        assert_eq!(editing.ring.color, style.editing_ring);
        assert_eq!(editing.ring.width, idle.ring.width);
        assert_eq!(editing.ring.offset, idle.ring.offset);
        assert_eq!(editing.rail, idle.rail);
        assert_eq!(editing.fill, idle.fill);
        assert_ne!(
            idle.thumb, editing.thumb,
            "the default palette must show editing"
        );
        assert_ne!(idle.ring.color, editing.ring.color);
    }

    #[test]
    fn editing_shows_no_ring_without_the_focus_indicator() {
        let style = SliderStyle::default();
        let look = style.look(VisualState::Normal, false, true);
        assert_eq!(look.ring, FocusRing::NONE);
        assert_eq!(look.thumb, style.editing_thumb);
    }

    #[test]
    fn a_disabled_slider_ignores_editing() {
        let style = SliderStyle::default();
        let look = style.look(VisualState::Disabled, true, true);
        assert_eq!(look.thumb, style.thumb.disabled);
        assert_eq!(look.ring, FocusRing::NONE);
    }
}
