use super::{
    button::FocusRing,
    color::{
        AFFIRM_HOVERED, AFFIRM_NORMAL, AFFIRM_PRESSED, BG_HOVERED, BG_NORMAL, BG_PRESSED, BORDER,
        DISABLED, FOCUS_RING, TEXT_MAIN, TEXT_MUTED,
    },
    state::{CheckedColors, StateColors, VisualState},
};
use bevy::prelude::*;

/// Colors and focus ring of a toggle switch.
#[derive(Clone, Copy, Debug)]
pub struct SwitchStyle {
    pub track: CheckedColors,
    pub thumb: CheckedColors,
    pub border: StateColors,
    pub focus_ring: FocusRing,
}

impl Default for SwitchStyle {
    fn default() -> Self {
        let track = |n: Color, h: Color, p: Color| StateColors {
            normal: n,
            hovered: h,
            pressed: p,
            disabled: DISABLED,
            focused: h,
        };
        let thumb = StateColors {
            normal: TEXT_MAIN,
            hovered: TEXT_MAIN,
            pressed: TEXT_MAIN,
            disabled: TEXT_MUTED,
            focused: TEXT_MAIN,
        };
        Self {
            track: CheckedColors {
                unchecked: track(BG_NORMAL, BG_HOVERED, BG_PRESSED),
                checked: track(AFFIRM_NORMAL, AFFIRM_HOVERED, AFFIRM_PRESSED),
            },
            thumb: CheckedColors {
                unchecked: thumb,
                checked: thumb,
            },
            border: StateColors::uniform(BORDER),
            focus_ring: FocusRing {
                color: FOCUS_RING,
                width: Val::Px(2.0),
                offset: Val::Px(2.0),
            },
        }
    }
}

/// What a switch looks like in one state, resolved from a [`SwitchStyle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SwitchLook {
    pub(crate) track: Color,
    pub(crate) thumb: Color,
    pub(crate) border: Color,
    pub(crate) ring: FocusRing,
}

impl SwitchStyle {
    /// `focused` is whether this switch owns the visible focus indicator.
    pub(crate) fn look(&self, state: VisualState, checked: bool, focused: bool) -> SwitchLook {
        SwitchLook {
            track: self.track.pick(checked, state),
            thumb: self.thumb.pick(checked, state),
            border: self.border.pick(state),
            ring: if focused && state != VisualState::Disabled {
                self.focus_ring
            } else {
                FocusRing::NONE
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

    #[test]
    fn checked_and_unchecked_use_their_own_colors() {
        let style = SwitchStyle::default();
        assert_eq!(
            style.look(VisualState::Normal, false, false).track,
            style.track.unchecked.normal
        );
        assert_eq!(
            style.look(VisualState::Normal, true, false).track,
            style.track.checked.normal
        );
        assert_ne!(
            style.track.unchecked.normal, style.track.checked.normal,
            "the default palette must tell on from off"
        );
    }

    #[test]
    fn state_colors_follow_the_visual_state() {
        let style = SwitchStyle::default();
        let look = style.look(VisualState::Pressed, true, false);
        assert_eq!(look.track, style.track.checked.pressed);
        assert_eq!(look.thumb, style.thumb.checked.pressed);
        assert_eq!(look.border, style.border.pressed);
    }

    #[test]
    fn ring_follows_focus_but_never_shows_on_a_disabled_switch() {
        let style = SwitchStyle::default();
        assert_eq!(
            style.look(VisualState::Normal, false, true).ring,
            style.focus_ring
        );
        assert_eq!(
            style.look(VisualState::Normal, false, false).ring,
            FocusRing::NONE
        );
        assert_eq!(
            style.look(VisualState::Disabled, true, true).ring,
            FocusRing::NONE
        );
    }
}
