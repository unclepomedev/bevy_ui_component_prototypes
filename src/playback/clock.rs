use bevy::prelude::*;

const DEFAULT_FRAME_RATE: f32 = 30.0;

/// A position that moves along a stretch of time: the clock of whatever plays (an animation, a replay).
///
/// It only keeps time. What is shown at the position is up to the app, which reads it each frame
/// (`Changed<PlaybackClock>` finds the clocks that moved). The position is in seconds, from 0 to `duration`.
///
/// `UiComponentPrototypesPlugin` advances the clocks that are playing, by the real time that passed
/// (not by the game's time, which may be paused or scaled), and triggers [`PlaybackEnded`](super::PlaybackEnded)
/// when one reaches its end. A clock that is paused does not change.
///
/// Every change goes through a method: a value that makes no sense (a negative or NaN time, a speed that is not
/// above zero) is ignored or clamped, so the position stays within `0..=duration`.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct PlaybackClock {
    position: f32,
    duration: f32,
    speed: f32,
    frame_rate: f32,
    playing: bool,
}

/// `value` if it is a finite number above zero.
fn positive(value: f32) -> Option<f32> {
    (value.is_finite() && value > 0.0).then_some(value)
}

impl PlaybackClock {
    /// A clock of `duration` seconds, paused at the start, at normal speed and 30 frames per second.
    /// A duration that is not a finite number above zero is taken as zero: the clock is over from the start.
    pub fn new(duration: f32) -> Self {
        Self {
            position: 0.0,
            duration: positive(duration).unwrap_or(0.0),
            speed: 1.0,
            frame_rate: DEFAULT_FRAME_RATE,
            playing: false,
        }
    }

    /// The speed the clock runs at: 2.0 is twice as fast. Not changed if `speed` is not a number above zero.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.set_speed(speed);
        self
    }

    /// The frames per second that [`step`](Self::step) counts in. Not changed if it is not a number above zero.
    pub fn with_frame_rate(mut self, frame_rate: f32) -> Self {
        if let Some(frame_rate) = positive(frame_rate) {
            self.frame_rate = frame_rate;
        }
        self
    }

    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn frame_rate(&self) -> f32 {
        self.frame_rate
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// The position has reached the end.
    pub fn is_finished(&self) -> bool {
        self.position >= self.duration
    }

    /// Where the position is between the start (0) and the end (1); 0 for a clock without duration.
    pub fn fraction(&self) -> f32 {
        if self.duration > 0.0 {
            self.position / self.duration
        } else {
            0.0
        }
    }

    /// Starts to run; from the start if the clock was at its end.
    pub fn play(&mut self) {
        if self.is_finished() {
            self.position = 0.0;
        }
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Pauses a clock that is playing, and plays one that is not.
    pub fn toggle(&mut self) {
        if self.playing {
            self.pause();
        } else {
            self.play();
        }
    }

    /// Moves to `seconds`, held within the clock. A clock that is playing keeps playing. NaN is ignored.
    pub fn seek(&mut self, seconds: f32) {
        if !seconds.is_nan() {
            self.position = seconds.clamp(0.0, self.duration);
        }
    }

    /// Moves to `fraction` of the way: 0 is the start and 1 the end. See [`seek`](Self::seek).
    pub fn seek_fraction(&mut self, fraction: f32) {
        self.seek(fraction.clamp(0.0, 1.0) * self.duration);
    }

    /// Pauses, and moves by `frames` frames (back if negative), held within the clock.
    pub fn step(&mut self, frames: i32) {
        self.pause();
        self.seek(self.position + frames as f32 / self.frame_rate);
    }

    /// Not changed if `speed` is not a number above zero.
    pub fn set_speed(&mut self, speed: f32) {
        if let Some(speed) = positive(speed) {
            self.speed = speed;
        }
    }

    /// Moves a clock that is playing by `delta` seconds of real time. At the end it stops there.
    /// Returns whether it ended just now: once for each time the end is reached.
    pub(crate) fn advance(&mut self, delta: f32) -> bool {
        if !self.playing {
            return false;
        }
        self.position = (self.position + delta * self.speed).min(self.duration);
        if self.position < self.duration {
            return false;
        }
        self.playing = false;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn playing(duration: f32) -> PlaybackClock {
        let mut clock = PlaybackClock::new(duration);
        clock.play();
        clock
    }

    #[test]
    fn a_new_clock_is_paused_at_the_start() {
        let clock = PlaybackClock::new(10.0);
        assert_eq!((clock.position(), clock.duration()), (0.0, 10.0));
        assert!(!clock.is_playing());
        assert!(!clock.is_finished());
        assert_eq!((clock.speed(), clock.frame_rate()), (1.0, 30.0));
    }

    #[test]
    fn a_duration_that_is_not_above_zero_is_no_duration() {
        for duration in [0.0, -3.0, f32::NAN, f32::INFINITY] {
            let clock = PlaybackClock::new(duration);
            assert_eq!(clock.duration(), 0.0);
            assert!(clock.is_finished());
        }
    }

    #[test]
    fn a_paused_clock_does_not_move() {
        let mut clock = PlaybackClock::new(10.0);
        assert!(!clock.advance(1.0));
        assert_eq!(clock.position(), 0.0);
    }

    #[test]
    fn a_playing_clock_moves_by_the_delta_times_the_speed() {
        let mut clock = playing(10.0).with_speed(2.0);
        clock.advance(0.5);
        assert_eq!(clock.position(), 1.0);
        clock.set_speed(0.5);
        clock.advance(1.0);
        assert_eq!(clock.position(), 1.5);
    }

    #[test]
    fn a_clock_stops_at_its_end_and_says_so_once() {
        let mut clock = playing(1.0);
        assert!(!clock.advance(0.5));
        assert!(clock.advance(0.7));
        assert_eq!(clock.position(), 1.0);
        assert!(!clock.is_playing());
        assert!(clock.is_finished());
        assert!(!clock.advance(1.0));
    }

    #[test]
    fn a_clock_that_is_sought_to_its_end_while_playing_ends_at_the_next_advance() {
        let mut clock = playing(2.0);
        clock.seek(2.0);
        assert!(clock.is_playing());
        assert!(clock.advance(0.1));
        assert!(!clock.is_playing());
    }

    #[test]
    fn playing_at_the_end_starts_from_the_beginning() {
        let mut clock = playing(1.0);
        clock.advance(2.0);
        clock.play();
        assert_eq!(clock.position(), 0.0);
        assert!(clock.is_playing());
    }

    #[test]
    fn a_clock_without_duration_ends_at_once_and_plays_again() {
        let mut clock = PlaybackClock::new(0.0);
        clock.play();
        assert!(clock.advance(0.1));
        clock.play();
        assert!(clock.advance(0.1));
    }

    #[test]
    fn toggle_swaps_between_playing_and_paused() {
        let mut clock = PlaybackClock::new(10.0);
        clock.toggle();
        assert!(clock.is_playing());
        clock.toggle();
        assert!(!clock.is_playing());
    }

    #[test]
    fn seeking_is_held_within_the_clock_and_keeps_the_state() {
        let mut clock = playing(10.0);
        clock.seek(4.0);
        assert_eq!(clock.position(), 4.0);
        assert!(clock.is_playing());
        clock.seek(-5.0);
        assert_eq!(clock.position(), 0.0);
        clock.seek(99.0);
        assert_eq!(clock.position(), 10.0);
        clock.seek(f32::NAN);
        assert_eq!(clock.position(), 10.0);
    }

    #[test]
    fn seeking_by_fraction_is_a_share_of_the_duration() {
        let mut clock = PlaybackClock::new(8.0);
        clock.seek_fraction(0.25);
        assert_eq!(clock.position(), 2.0);
        assert_eq!(clock.fraction(), 0.25);
        clock.seek_fraction(3.0);
        assert_eq!(clock.position(), 8.0);
    }

    #[test]
    fn the_fraction_of_a_clock_without_duration_is_zero() {
        assert_eq!(PlaybackClock::new(0.0).fraction(), 0.0);
    }

    #[test]
    fn a_step_pauses_and_moves_by_frames_within_the_clock() {
        let mut clock = playing(1.0).with_frame_rate(10.0);
        clock.step(3);
        assert!(!clock.is_playing());
        assert!((clock.position() - 0.3).abs() < 1e-6);
        clock.step(-5);
        assert_eq!(clock.position(), 0.0);
        clock.step(100);
        assert_eq!(clock.position(), 1.0);
    }

    #[test]
    fn a_speed_or_a_frame_rate_that_is_not_above_zero_is_ignored() {
        let mut clock = PlaybackClock::new(1.0)
            .with_speed(2.0)
            .with_frame_rate(24.0);
        for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            clock.set_speed(bad);
            clock = clock.with_frame_rate(bad);
        }
        assert_eq!((clock.speed(), clock.frame_rate()), (2.0, 24.0));
    }
}
