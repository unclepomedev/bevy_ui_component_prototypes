use super::color::*;
use bevy::{picking::hover::PickingInteraction, prelude::*};

#[derive(Clone, Copy, Debug)]
pub struct StateColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub disabled: Color,
    pub focused: Color,
}

impl StateColors {
    pub fn uniform(c: Color) -> Self {
        Self {
            normal: c,
            hovered: c,
            pressed: c,
            disabled: c,
            focused: c,
        }
    }

    pub(crate) fn pick(&self, state: VisualState) -> Color {
        match state {
            VisualState::Normal => self.normal,
            VisualState::Hovered => self.hovered,
            VisualState::Pressed => self.pressed,
            VisualState::Focused => self.focused,
            VisualState::Disabled => self.disabled,
        }
    }
}

/// Outline drawn around a button while its focus indicator is visible.
#[derive(Clone, Copy, Debug)]
pub struct FocusRing {
    pub color: Color,
    pub width: Val,
    pub offset: Val,
}

impl FocusRing {
    /// No ring; focus is conveyed by the `focused` colors alone.
    pub const NONE: Self = Self {
        color: Color::NONE,
        width: Val::Px(0.0),
        offset: Val::Px(0.0),
    };
}

#[derive(Clone, Debug, Copy)]
pub struct ButtonStyle {
    pub bg: StateColors,
    pub fg: StateColors,
    pub border: StateColors,
    pub focus_ring: FocusRing,
}

#[derive(Clone, Debug)]
pub struct ButtonStyles {
    pub neutral: ButtonStyle,
    pub primary: ButtonStyle,
    pub affirmative: ButtonStyle,
    pub danger: ButtonStyle,
}

#[derive(Component, Clone, Debug)]
pub struct Theme {
    pub buttons: ButtonStyles,
    pub font: FontSource,
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

/// Which set of colors a button currently shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VisualState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

impl VisualState {
    /// Precedence: disabled > pressed > hovered > focused > normal.
    pub(crate) fn resolve(interaction: &PickingInteraction, focused: bool, disabled: bool) -> Self {
        if disabled {
            return Self::Disabled;
        }
        match interaction {
            PickingInteraction::Pressed => Self::Pressed,
            PickingInteraction::Hovered => Self::Hovered,
            PickingInteraction::None if focused => Self::Focused,
            PickingInteraction::None => Self::Normal,
        }
    }
}
