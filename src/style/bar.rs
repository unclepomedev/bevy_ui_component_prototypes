use bevy::{prelude::*, ui::ColorStop};

/// A bar drawn as separate blocks instead of one continuous bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarSegments {
    /// How many blocks the bar is made of. Fewer than two draw a plain bar.
    pub count: u32,
    /// The empty space between two blocks, as a share of one pitch (a block plus the gap after it):
    /// `0.2` leaves a fifth of each pitch empty.
    pub gap: f32,
}

impl Default for BarSegments {
    fn default() -> Self {
        Self {
            count: 10,
            gap: 0.2,
        }
    }
}

impl BarSegments {
    /// `segments` unless it draws fewer than two blocks, which is a plain bar.
    pub(crate) fn usable(segments: Option<Self>) -> Option<Self> {
        segments.filter(|segments| segments.count >= 2)
    }

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

/// Fully rounded ends (a pill, a circle) when `rounded`, square corners otherwise.
pub(crate) fn corner_radius(size: f32, rounded: bool) -> f32 {
    if rounded { size / 2.0 } else { 0.0 }
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
    fn fewer_than_two_blocks_is_a_plain_bar() {
        for count in [0, 1] {
            assert_eq!(
                BarSegments::usable(Some(BarSegments { count, gap: 0.2 })),
                None
            );
        }
        assert!(BarSegments::usable(Some(BarSegments::default())).is_some());
        assert_eq!(BarSegments::usable(None), None);
    }

    #[test]
    fn a_block_is_lit_once_the_value_reaches_its_end() {
        let segments = BarSegments {
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
            let segments = BarSegments { count, gap: 0.0 };
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
        let stops = BarSegments { count: 3, gap: 0.5 }.stops(2, color);

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
                let stops = BarSegments { count, gap }.stops(count, Color::WHITE);
                let last = percent(stops.last().unwrap());
                assert!(
                    (last - 100.0).abs() < 1e-3,
                    "{count} blocks, gap {gap}: {last}"
                );
                assert_eq!(percent(&stops[0]), 0.0);
            }
        }
    }

    // A gap of 1.0 or more must not divide by zero or produce a NaN.
    #[test]
    fn a_whole_gap_still_gives_finite_stops() {
        for gap in [1.0, 2.0] {
            let stops = BarSegments { count: 2, gap }.stops(2, Color::WHITE);
            assert!(stops.iter().all(|stop| percent(stop).is_finite()));
        }
    }

    #[test]
    fn stop_positions_never_go_backwards() {
        // A backwards step is drawn as a blend across the block, not as a hard edge.
        for count in 2..=40 {
            for gap in [0.0, 0.1, 0.2, 0.5, 0.8, 1.0] {
                let stops = BarSegments { count, gap }.stops(count, Color::WHITE);
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
        let stops = BarSegments { count: 2, gap: 0.0 }.stops(2, color);

        let seen: Vec<(Color, f32)> = stops.iter().map(|s| (s.color, percent(s))).collect();
        assert_eq!(
            seen,
            vec![(color, 0.0), (color, 50.0), (color, 50.0), (color, 100.0)]
        );
    }
}
