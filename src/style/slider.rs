use super::{
    button::FocusRing,
    color::{
        AFFIRM_HOVERED, AFFIRM_NORMAL, AFFIRM_PRESSED, BG_HOVERED, BG_PRESSED, DISABLED, EDITING,
        FOCUS_RING, TEXT_MAIN, TEXT_MUTED,
    },
    state::{StateColors, VisualState},
};
use bevy::prelude::*;

/// Colors and focus ring of a slider: the rail, the filled part of it and the thumb.
#[derive(Clone, Copy, Debug)]
pub struct SliderStyle {
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
