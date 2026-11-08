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
    waited: f32,
    /// The widget whose tooltip was dismissed, and stays so until something else is wanted.
    dismissed: Option<Entity>,
    /// Seconds since a tooltip was last on screen.
    since_shown: f32,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            target: None,
            shown: false,
            waited: 0.0,
            dismissed: None,
            since_shown: f32::INFINITY,
        }
    }
}

impl Tracker {
    /// `wants` is the widget the pointer or the focus is on. `dismiss` is set on the frame that asks to hide
    /// the tooltip (a press, Escape). `dt` is the seconds since the last step.
    pub(super) fn step(
        &mut self,
        wants: Option<Entity>,
        dismiss: bool,
        dt: f32,
        timing: impl Fn(Entity) -> Timing,
    ) -> Step {
        if dismiss {
            self.dismissed = wants.or(self.target);
        } else if self.dismissed != wants {
            self.dismissed = None;
        }
        let wants = wants.filter(|wants| Some(*wants) != self.dismissed);

        let mut step = Step::default();
        if wants != self.target {
            step.hide = self.shown;
            self.shown = false;
            self.target = wants;
            self.waited = 0.0;
        }

        if let (Some(target), false) = (self.target, self.shown) {
            self.waited += dt;
            let timing = timing(target);
            let delay = if self.since_shown < timing.warmup {
                0.0
            } else {
                timing.delay
            };
            if self.waited >= delay {
                self.shown = true;
                step.show = Some(target);
            }
        }

        if self.shown {
            self.since_shown = 0.0;
        } else {
            self.since_shown += dt;
        }
        step
    }

    /// The tooltip on screen is gone without the tracker asking (its widget was despawned).
    pub(super) fn lost(&mut self) {
        self.shown = false;
        self.target = None;
        self.waited = 0.0;
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

    fn shows(tracker: &mut Tracker, wants: Option<Entity>, dt: f32) -> Step {
        tracker.step(wants, false, dt, timing)
    }

    #[test]
    fn a_tooltip_waits_for_the_delay() {
        let mut tracker = Tracker::default();
        assert_eq!(shows(&mut tracker, Some(A), 0.2), Step::default());
        assert_eq!(shows(&mut tracker, Some(A), 0.2), Step::default());
        let step = shows(&mut tracker, Some(A), 0.2);
        assert_eq!(step.show, Some(A));
        assert!(!step.hide);
    }

    #[test]
    fn it_shows_once() {
        let mut tracker = Tracker::default();
        assert_eq!(shows(&mut tracker, Some(A), 0.6).show, Some(A));
        assert_eq!(shows(&mut tracker, Some(A), 0.1), Step::default());
    }

    #[test]
    fn leaving_before_the_delay_shows_nothing() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.4);
        assert_eq!(shows(&mut tracker, None, 0.4), Step::default());
        assert_eq!(shows(&mut tracker, None, 1.0), Step::default());
    }

    #[test]
    fn leaving_hides_what_is_shown() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.6);
        assert_eq!(
            shows(&mut tracker, None, 0.1),
            Step {
                hide: true,
                show: None
            }
        );
    }

    #[test]
    fn the_next_widget_shows_at_once_inside_the_warmup() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.6);
        let step = shows(&mut tracker, Some(B), 0.016);
        assert!(step.hide);
        assert_eq!(step.show, Some(B));
    }

    #[test]
    fn the_next_widget_waits_again_after_the_warmup() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.6);
        shows(&mut tracker, None, 0.1);
        shows(&mut tracker, None, 0.5);
        assert_eq!(shows(&mut tracker, Some(B), 0.2), Step::default());
        assert_eq!(shows(&mut tracker, Some(B), 0.4).show, Some(B));
    }

    #[test]
    fn a_dismissed_tooltip_stays_away_until_the_widget_is_left() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.6);
        let step = tracker.step(Some(A), true, 0.1, timing);
        assert!(step.hide);
        assert_eq!(shows(&mut tracker, Some(A), 2.0), Step::default());

        shows(&mut tracker, None, 0.1);
        shows(&mut tracker, None, 1.0);
        assert_eq!(shows(&mut tracker, Some(A), 0.6).show, Some(A));
    }

    #[test]
    fn dismissing_while_waiting_cancels_the_wait() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.2);
        tracker.step(Some(A), true, 0.1, timing);
        assert_eq!(shows(&mut tracker, Some(A), 2.0), Step::default());
    }

    #[test]
    fn a_lost_tooltip_can_show_again() {
        let mut tracker = Tracker::default();
        shows(&mut tracker, Some(A), 0.6);
        tracker.lost();
        assert_eq!(shows(&mut tracker, Some(A), 0.0).show, Some(A));
    }
}
