use crate::style::theme::{ButtonStyle, Theme};
use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default)]
pub enum ButtonVariant {
    #[default]
    Neutral,
    Primary,
    Affirmative,
    Danger,
    Custom(Box<ButtonStyle>),
}

impl ButtonVariant {
    /// Returns the style this variant selects from `theme`.
    pub(crate) fn resolve<'a>(&'a self, theme: &'a Theme) -> &'a ButtonStyle {
        match self {
            Self::Neutral => &theme.buttons.neutral,
            Self::Primary => &theme.buttons.primary,
            Self::Affirmative => &theme.buttons.affirmative,
            Self::Danger => &theme.buttons.danger,
            Self::Custom(style) => style,
        }
    }
}
