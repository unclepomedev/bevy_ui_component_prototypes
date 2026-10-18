use bevy::{color::Color, picking::hover::PickingInteraction};

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

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use PickingInteraction::{Hovered, None as Idle, Pressed};

    fn resolve(interaction: PickingInteraction, focused: bool, disabled: bool) -> VisualState {
        VisualState::resolve(&interaction, focused, disabled)
    }

    #[test]
    fn disabled_beats_everything() {
        assert_eq!(resolve(Pressed, true, true), VisualState::Disabled);
    }

    #[test]
    fn pressed_beats_hovered_and_focused() {
        assert_eq!(resolve(Pressed, true, false), VisualState::Pressed);
    }

    #[test]
    fn hovered_beats_focused() {
        assert_eq!(resolve(Hovered, true, false), VisualState::Hovered);
    }

    #[test]
    fn focus_shows_only_while_idle() {
        assert_eq!(resolve(Idle, true, false), VisualState::Focused);
        assert_eq!(resolve(Idle, false, false), VisualState::Normal);
    }

    #[test]
    fn pick_returns_the_color_of_each_state() {
        let colors = StateColors {
            normal: Color::srgb(0.1, 0.0, 0.0),
            hovered: Color::srgb(0.2, 0.0, 0.0),
            pressed: Color::srgb(0.3, 0.0, 0.0),
            disabled: Color::srgb(0.4, 0.0, 0.0),
            focused: Color::srgb(0.5, 0.0, 0.0),
        };
        assert_eq!(colors.pick(VisualState::Normal), colors.normal);
        assert_eq!(colors.pick(VisualState::Hovered), colors.hovered);
        assert_eq!(colors.pick(VisualState::Pressed), colors.pressed);
        assert_eq!(colors.pick(VisualState::Focused), colors.focused);
        assert_eq!(colors.pick(VisualState::Disabled), colors.disabled);
    }
}
