use super::color::*;
use crate::view::ButtonVariant;
use bevy::{picking::hover::PickingInteraction, prelude::*};

#[derive(Clone, Copy, Debug)]
pub struct StateColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub disabled: Color,
}

impl StateColors {
    pub fn uniform(c: Color) -> Self {
        Self {
            normal: c,
            hovered: c,
            pressed: c,
            disabled: c,
        }
    }

    pub fn pick(&self, interaction: &PickingInteraction, disabled: bool) -> Color {
        if disabled {
            return self.disabled;
        }
        match interaction {
            PickingInteraction::Pressed => self.pressed,
            PickingInteraction::Hovered => self.hovered,
            PickingInteraction::None => self.normal,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonStyle {
    pub bg: StateColors,
    pub fg: StateColors,
    pub border: StateColors,
}

#[derive(Clone, Debug)]
pub struct ButtonStyles {
    pub neutral: ButtonStyle,
    pub primary: ButtonStyle,
    pub affirmative: ButtonStyle,
    pub danger: ButtonStyle,
}

// TODO to Component
#[derive(Resource, Clone)]
pub struct Theme {
    pub buttons: ButtonStyles,
    pub font: FontSource,
}

impl Theme {
    pub fn resolve_button<'a>(&'a self, variant: &'a ButtonVariant) -> &'a ButtonStyle {
        match variant {
            ButtonVariant::Neutral => &self.buttons.neutral,
            ButtonVariant::Primary => &self.buttons.primary,
            ButtonVariant::Affirmative => &self.buttons.affirmative,
            ButtonVariant::Danger => &self.buttons.danger,
            ButtonVariant::Custom(style) => style,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        let solid = |n: Color, h: Color, p: Color| ButtonStyle {
            bg: StateColors {
                normal: n,
                hovered: h,
                pressed: p,
                disabled: DISABLED,
            },
            fg: StateColors {
                normal: TEXT_MAIN,
                hovered: TEXT_MAIN,
                pressed: TEXT_MAIN,
                disabled: TEXT_MUTED,
            },
            border: StateColors::uniform(BORDER),
        };
        Self {
            buttons: ButtonStyles {
                neutral: solid(BG_NORMAL, BG_HOVERED, BG_PRESSED),
                primary: solid(PRIMARY_NORMAL, PRIMARY_HOVERED, PRIMARY_PRESSED),
                affirmative: solid(AFFIRM_NORMAL, AFFIRM_HOVERED, AFFIRM_PRESSED),
                danger: solid(DANGER_NORMAL, DANGER_HOVERED, DANGER_PRESSED),
            },
            font: FontSource::default(),
        }
    }
}
