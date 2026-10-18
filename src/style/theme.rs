use super::{
    button::{ButtonStyle, ButtonStyles, FocusRing},
    color::*,
    state::StateColors,
};
use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Theme {
    pub buttons: ButtonStyles,
    pub font: FontSource,
    pub font_smoothing: FontSmoothing,
}

#[derive(Resource, Deref, DerefMut, Default, Clone)]
pub struct GlobalTheme(pub Theme);

impl Default for Theme {
    fn default() -> Self {
        let solid = |n: Color, h: Color, p: Color| ButtonStyle {
            bg: StateColors {
                normal: n,
                hovered: h,
                pressed: p,
                disabled: DISABLED,
                focused: h,
            },
            fg: StateColors {
                normal: TEXT_MAIN,
                hovered: TEXT_MAIN,
                pressed: TEXT_MAIN,
                disabled: TEXT_MUTED,
                focused: TEXT_MAIN,
            },
            border: StateColors::uniform(BORDER),
            focus_ring: FocusRing {
                color: FOCUS_RING,
                width: Val::Px(2.0),
                offset: Val::Px(2.0),
            },
            surface: None,
        };
        Self {
            buttons: ButtonStyles {
                neutral: solid(BG_NORMAL, BG_HOVERED, BG_PRESSED),
                primary: solid(PRIMARY_NORMAL, PRIMARY_HOVERED, PRIMARY_PRESSED),
                affirmative: solid(AFFIRM_NORMAL, AFFIRM_HOVERED, AFFIRM_PRESSED),
                danger: solid(DANGER_NORMAL, DANGER_HOVERED, DANGER_PRESSED),
            },
            font: FontSource::default(),
            font_smoothing: FontSmoothing::default(),
        }
    }
}
