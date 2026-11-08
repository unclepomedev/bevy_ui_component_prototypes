use bevy::prelude::Entity;

/// What the tracker asks for in one step.
#[derive(Debug, Default, PartialEq)]
pub(super) struct Step {
    /// The tooltip on screen goes away.
    pub(super) hide: bool,
    /// A tooltip for this widget appears.
    pub(super) show: Option<Entity>,
}

/// When the delay of a widget and how long it was shown.
#[derive(Clone, Copy, Debug)]
pub(super) struct Timing {
    pub(super) delay: f32,
    pub(super) warmup: f32,
}

/// Decides when a tooltip shows and hides. It knows nothing of the world: it is told which widget wants
/// a tooltip, and answers with what to do.
#[derive(Debug)]
pub(super) struct Tracker {
    /// The widget whose tooltip is shown, or waits to be.
    target: Option<Entity>,
    shown: bool,
    /// When the widget of `target` was first seen.
    seen_at: f64,
    /// The widget whose tooltip was dismissed, and stays so until something else is wanted.
    dismissed: Option<Entity>,
    /// When a tooltip was last on screen.
    last_shown_at: f64,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            target: None,
            shown: false,
            seen_at: 0.0,
            dismissed: None,
            last_shown_at: f64::NEG_INFINITY,
        }
    }
}

impl Tracker {
    /// `wants` is the widget the pointer or the focus is on. `dismiss` is set on the frame that asks to hide
    /// the tooltip (a press, Escape). `now` is the seconds on the clock of this step.
    pub(super) fn step(
        &mut self,
        wants: Option<Entity>,
        dismiss: bool,
        now: f64,
        timing: impl Fn(Entity) -> Timing,
    ) -> Step {
        let wants = self.without_dismissed(wants, dismiss);

        let mut step = Step::default();
        if wants != self.target {
            step.hide = self.shown;
            self.shown = false;
            self.target = wants;
            self.seen_at = now;
        }

        if let (Some(target), false) = (self.target, self.shown) {
            let timing = timing(target);
            let warm = now - self.last_shown_at < f64::from(timing.warmup);
            let delay = if warm { 0.0 } else { timing.delay };
            if (now - self.seen_at) as f32 >= delay {
                self.shown = true;
                step.show = Some(target);
            }
        }

        if self.shown {
            self.last_shown_at = now;
        }
        step
    }

    /// The widget that wants a tooltip, unless it is the one dismissed and not yet left.
    fn without_dismissed(&mut self, wants: Option<Entity>, dismiss: bool) -> Option<Entity> {
        if dismiss {
            self.dismissed = wants.or(self.target);
        } else if self.dismissed != wants {
            self.dismissed = None;
        }
        wants.filter(|wants| Some(*wants) != self.dismissed)
    }

    /// The tooltip on screen is gone without the tracker asking (its widget was despawned).
    pub(super) fn lost(&mut self) {
        self.shown = false;
        self.target = None;
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    const A: Entity = Entity::from_raw_u32(1).unwrap();
    const B: Entity = Entity::from_raw_u32(2).unwrap();

    fn timing(_: Entity) -> Timing {
        Timing {
            delay: 0.5,
            warmup: 0.3,
        }
    }

    fn shows(tracker: &mut Tracker, wants: Option<Entity>, now: f64) -> Step {
        tracker.step(wants, false, now, timing)
    }

    /// The wait is counted from the step that brought the widget in.
    #[test]
    fn the_wait_starts_when_the_widget_is_first_wanted() {
        let mut tracker = Tracker::default();
        assert_eq!(shows(&mut tracker, Some(A), 10.0), Step::default());
        assert_eq!(shows(&mut tracker, Some(A), 10.4), Step::default());
        assert_eq!(shows(&mut tracker, Some(A), 10.5).show, Some(A));
    }

    #[test]
    fn a_tooltip_waits_for_the_delay() {
        let mut tracker = Tracker::default();
        assert_eq!(shows(&mut tracker, Some(A), 0.0), Step::default());
        assert_eq!(shows(&mut tracker, Some(A), 0.2), Step::default());
        assert_eq!(shows(&mut tracker, Some(A), 0.4), Step::default());
        let step = shows(&mut tracker, Some(A), 0.5);
        assert_eq!(step.show, Some(A));
        assert!(!step.hide);
    }

    #[test]
    fn a_tooltip_without_a_delay_shows_at_once() {
        let mut tracker = Tracker::default();
        let timing = |_| Timing {
            delay: 0.0,
            warmup: 0.3,
        };
        assert_eq!(tracker.step(Some(A), false, 0.1, timing).show, Some(A));
    }

    #[test]
    fn it_shows_once() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        assert_eq!(shows(&mut tracker, Some(A), 0.6).show, Some(A));
        assert_eq!(shows(&mut tracker, Some(A), 0.7), Step::default());
    }

    #[test]
    fn leaving_before_the_delay_shows_nothing() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        shows(&mut tracker, Some(A), 0.4);
        assert_eq!(shows(&mut tracker, None, 0.8), Step::default());
        assert_eq!(shows(&mut tracker, None, 1.8), Step::default());
    }

    #[test]
    fn leaving_hides_what_is_shown() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        shows(&mut tracker, Some(A), 0.6);
        assert_eq!(
            shows(&mut tracker, None, 0.7),
            Step {
                hide: true,
                show: None
            }
        );
    }

    #[test]
    fn the_next_widget_shows_at_once_inside_the_warmup() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        shows(&mut tracker, Some(A), 0.6);
        let step = shows(&mut tracker, Some(B), 0.616);
        assert!(step.hide);
        assert_eq!(step.show, Some(B));
    }

    #[test]
    fn the_next_widget_waits_again_after_the_warmup() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        shows(&mut tracker, Some(A), 0.6);
        shows(&mut tracker, None, 0.7);
        shows(&mut tracker, None, 1.2);
        assert_eq!(shows(&mut tracker, Some(B), 1.4), Step::default());
        assert_eq!(shows(&mut tracker, Some(B), 1.8), Step::default());
        assert_eq!(shows(&mut tracker, Some(B), 1.9).show, Some(B));
    }

    /// A step long enough to cross the warmup is already past it, so the next widget waits.
    #[test]
    fn a_step_that_crosses_the_warmup_waits_again() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        shows(&mut tracker, Some(A), 0.6);
        assert_eq!(
            shows(&mut tracker, Some(B), 1.2),
            Step {
                hide: true,
                show: None
            }
        );
        assert_eq!(shows(&mut tracker, Some(B), 1.8).show, Some(B));
    }

    #[test]
    fn a_dismissed_tooltip_stays_away_until_the_widget_is_left() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        shows(&mut tracker, Some(A), 0.6);
        let step = tracker.step(Some(A), true, 0.7, timing);
        assert!(step.hide);
        assert_eq!(shows(&mut tracker, Some(A), 2.7), Step::default());

        shows(&mut tracker, None, 2.8);
        shows(&mut tracker, None, 3.8);
        shows(&mut tracker, Some(A), 3.9);
        assert_eq!(shows(&mut tracker, Some(A), 4.5).show, Some(A));
    }

    #[test]
    fn dismissing_while_waiting_cancels_the_wait() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        shows(&mut tracker, Some(A), 0.2);
        tracker.step(Some(A), true, 0.3, timing);
        assert_eq!(shows(&mut tracker, Some(A), 2.3), Step::default());
    }

    #[test]
    fn a_lost_tooltip_can_show_again() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.0);
        shows(&mut tracker, Some(A), 0.6);
        tracker.lost();
        assert_eq!(shows(&mut tracker, Some(A), 0.7).show, Some(A));
    }
}
