use super::color::*;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct ColorSet {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

// TODO to Component
#[derive(Resource, Clone)]
pub struct Theme {
    pub neutral: ColorSet,
    pub primary: ColorSet,
    pub affirmative: ColorSet,
    pub danger: ColorSet,

    pub disabled_bg: Color,
    pub text_main: Color,
    pub text_muted: Color,
    pub border: Color,

    pub font: FontSource,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            neutral: ColorSet {
                normal: BG_NORMAL,
                hovered: BG_HOVERED,
                pressed: BG_PRESSED,
            },
            primary: ColorSet {
                normal: PRIMARY_NORMAL,
                hovered: PRIMARY_HOVERED,
                pressed: PRIMARY_PRESSED,
            },
            affirmative: ColorSet {
                normal: AFFIRM_NORMAL,
                hovered: AFFIRM_HOVERED,
                pressed: AFFIRM_PRESSED,
            },
            danger: ColorSet {
                normal: DANGER_NORMAL,
                hovered: DANGER_HOVERED,
                pressed: DANGER_PRESSED,
            },
            disabled_bg: DISABLED,
            text_main: TEXT_MAIN,
            text_muted: TEXT_MUTED,
            border: BORDER,
            font: FontSource::default(),
        }
    }
}
