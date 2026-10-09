use crate::ColorSet;
use crate::style::color::*;
use bevy::{picking::hover::PickingInteraction, prelude::*};
// TODO: replace button-related components

/// Presentation-agnostic interaction surface. Any pressable thing
/// (flat button, icon button, header gear...) carries this.
/// Fires `Pressed` on click unless `ButtonDisabled` is present.
#[derive(Component, Default)]
#[require(PickingInteraction, ButtonColors)]
pub struct Pressable;

/// Opt out of `sync_pressable_visuals` background/content coloring.
#[derive(Component)]
pub struct SelfStyled;

#[derive(Component)]
pub struct ButtonDisabled;

#[derive(Component, Clone, Copy, Debug)]
pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self::normal_variant()
    }
}

impl ButtonColors {
    pub fn normal_variant() -> Self {
        Self {
            normal: BG_NORMAL,
            hovered: BG_HOVERED,
            pressed: BG_PRESSED,
        }
    }
    pub fn primary_variant() -> Self {
        Self {
            normal: PRIMARY_NORMAL,
            hovered: PRIMARY_HOVERED,
            pressed: PRIMARY_PRESSED,
        }
    }

    pub fn affirmative_variant() -> Self {
        Self {
            normal: AFFIRM_NORMAL,
            hovered: AFFIRM_HOVERED,
            pressed: AFFIRM_PRESSED,
        }
    }

    pub fn danger_variant() -> Self {
        Self {
            normal: DANGER_NORMAL,
            hovered: DANGER_HOVERED,
            pressed: DANGER_PRESSED,
        }
    }
}

impl From<ColorSet> for ButtonColors {
    fn from(c: ColorSet) -> Self {
        Self {
            normal: c.normal,
            hovered: c.hovered,
            pressed: c.pressed,
        }
    }
}
