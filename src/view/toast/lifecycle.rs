// TODO: rm
#![cfg_attr(not(test), expect(dead_code))]

//! The life of a toast over time, as pure functions: no ECS, no theme lookup, no units.

use crate::style::toast::ToastMetrics;

/// The three phases of a toast, in seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ToastTiming {
    fade_in: f32,
    hold: f32,
    fade_out: f32,
}

/// What a toast looks like at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ToastFrame {
    /// 0 to 1. Multiplied into the alpha of every color of the toast.
    pub(crate) opacity: f32,
    /// 1 to 0: how much of the way in is still to go. 1 is the start position the host gives,
    /// 0 is the resting place. The view turns it into a translation.
    pub(crate) slide: f32,
    /// The toast is over and can be despawned.
    pub(crate) finished: bool,
}

/// A negative or NaN time is no time at all.
fn seconds(value: f32) -> f32 {
    if value > 0.0 { value } else { 0.0 }
}

impl ToastTiming {
    /// `duration` is the hold: how long the toast stays fully visible. The fades come on top of it.
    pub(crate) fn new(metrics: &ToastMetrics, duration: f32) -> Self {
        Self {
            fade_in: seconds(metrics.fade_in),
            hold: seconds(duration),
            fade_out: seconds(metrics.fade_out),
        }
    }

    /// The elapsed time at which the fade-out begins.
    pub(crate) fn fade_out_start(&self) -> f32 {
        self.fade_in + self.hold
    }

    /// The elapsed time at which the toast is over.
    pub(crate) fn total(&self) -> f32 {
        self.fade_out_start() + self.fade_out
    }

    pub(crate) fn frame(&self, elapsed: f32) -> ToastFrame {
        let elapsed = seconds(elapsed);
        if elapsed >= self.total() {
            return ToastFrame {
                opacity: 0.0,
                slide: 0.0,
                finished: true,
            };
        }
        // Every division below is by a positive number: the branch is only entered when the phase has length.
        if elapsed < self.fade_in {
            let progress = elapsed / self.fade_in;
            return ToastFrame {
                opacity: progress.clamp(0.0, 1.0),
                // Ease-out: fast at first, settling at the end.
                slide: (1.0 - progress).clamp(0.0, 1.0).powi(3),
                finished: false,
            };
        }
        let start = self.fade_out_start();
        let opacity = if elapsed < start {
            1.0
        } else {
            1.0 - (elapsed - start) / self.fade_out
        };
        ToastFrame {
            opacity: opacity.clamp(0.0, 1.0),
            slide: 0.0,
            finished: false,
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    /// fade in 0.5, hold 2.0, fade out 0.25: the fade-out starts at 2.5 and the toast ends at 2.75.
    fn sample() -> ToastTiming {
        ToastTiming {
            fade_in: 0.5,
            hold: 2.0,
            fade_out: 0.25,
        }
    }

    #[test]
    fn it_starts_invisible_and_a_whole_slide_away() {
        let start = sample().frame(0.0);
        assert_eq!(start.opacity, 0.0);
        assert_eq!(start.slide, 1.0);
        assert!(!start.finished);
    }

    #[test]
    fn the_fade_in_ends_fully_visible_and_in_place() {
        let done = sample().frame(0.5);
        assert_eq!(done.opacity, 1.0);
        assert_eq!(done.slide, 0.0);
    }

    #[test]
    fn the_fade_in_is_halfway_at_half_the_time() {
        assert_eq!(sample().frame(0.25).opacity, 0.5);
    }

    #[test]
    fn the_hold_is_fully_visible_up_to_the_start_of_the_fade_out() {
        for elapsed in [0.5, 1.5, 2.49, 2.5] {
            let held = sample().frame(elapsed);
            assert_eq!(held.opacity, 1.0, "{elapsed}");
            assert_eq!(held.slide, 0.0, "{elapsed}");
            assert!(!held.finished, "{elapsed}");
        }
    }

    #[test]
    fn the_fade_out_is_halfway_at_half_the_time_and_does_not_slide() {
        let half = sample().frame(2.625);
        assert_eq!(half.opacity, 0.5);
        assert_eq!(half.slide, 0.0);
        assert!(!half.finished);
    }

    #[test]
    fn it_finishes_exactly_at_the_total_and_stays_finished() {
        let timing = sample();
        assert_eq!(timing.total(), 2.75);
        assert!(!timing.frame(2.74).finished);
        for elapsed in [2.75, 3.0, 1000.0] {
            let over = timing.frame(elapsed);
            assert!(over.finished, "{elapsed}");
            assert_eq!(over.opacity, 0.0, "{elapsed}");
            assert_eq!(over.slide, 0.0, "{elapsed}");
        }
    }

    #[test]
    fn opacity_rises_through_the_fade_in_and_falls_through_the_fade_out() {
        let timing = sample();
        let mut previous = timing.frame(0.0).opacity;
        for i in 1..=100 {
            let opacity = timing.frame(0.5 * i as f32 / 100.0).opacity;
            assert!(opacity >= previous, "rising at {i}");
            previous = opacity;
        }
        let mut previous = 1.0;
        for i in 0..=100 {
            let opacity = timing.frame(2.5 + 0.25 * i as f32 / 100.0).opacity;
            assert!(opacity <= previous, "falling at {i}");
            previous = opacity;
        }
    }

    #[test]
    fn the_slide_only_shrinks_and_never_leaves_its_range() {
        let timing = sample();
        let mut previous = 1.0;
        for i in 0..=1000 {
            let slide = timing.frame(i as f32 * 0.003).slide;
            assert!((0.0..=1.0).contains(&slide), "{slide}");
            assert!(slide <= previous, "growing at {i}");
            previous = slide;
        }
    }

    #[test]
    fn the_slide_settles_early_rather_than_linearly() {
        // Ease-out: at half the fade-in, more than half of the way is done.
        assert!(sample().frame(0.25).slide < 0.5);
    }

    #[test]
    fn without_a_fade_in_it_starts_visible_and_in_place() {
        let timing = ToastTiming {
            fade_in: 0.0,
            ..sample()
        };
        let start = timing.frame(0.0);
        assert_eq!(start.opacity, 1.0);
        assert_eq!(start.slide, 0.0);
    }

    #[test]
    fn without_a_fade_out_it_ends_the_moment_the_hold_does() {
        let timing = ToastTiming {
            fade_out: 0.0,
            ..sample()
        };
        let last = timing.frame(2.49);
        assert_eq!(last.opacity, 1.0);
        assert!(!last.finished);
        assert!(timing.frame(2.5).finished);
    }

    #[test]
    fn a_toast_without_any_time_is_over_at_once() {
        let timing = ToastTiming {
            fade_in: 0.0,
            hold: 0.0,
            fade_out: 0.0,
        };
        assert!(timing.frame(0.0).finished);
    }

    // No division by zero, no NaN, whatever comes in.
    #[test]
    fn bad_input_gives_a_finite_frame_in_range() {
        let metrics = ToastMetrics {
            fade_in: f32::NAN,
            fade_out: -1.0,
            ..ToastMetrics::default()
        };
        let timings = [
            ToastTiming::new(&metrics, f32::NAN),
            ToastTiming::new(&metrics, -5.0),
            sample(),
        ];
        for timing in timings {
            for elapsed in [f32::NAN, -1.0, 0.0, 1.0, 1000.0] {
                let frame = timing.frame(elapsed);
                assert!((0.0..=1.0).contains(&frame.opacity), "{frame:?}");
                assert!((0.0..=1.0).contains(&frame.slide), "{frame:?}");
            }
        }
    }

    #[test]
    fn the_timing_comes_from_the_metrics_and_the_duration() {
        let metrics = ToastMetrics {
            fade_in: 0.5,
            fade_out: 0.25,
            ..ToastMetrics::default()
        };
        let timing = ToastTiming::new(&metrics, 2.0);
        assert_eq!(timing, sample());
        assert_eq!(timing.fade_out_start(), 2.5);
        assert_eq!(timing.total(), 2.75);
    }
}
