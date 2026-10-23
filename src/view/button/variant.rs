use crate::style::{
    button::{ButtonStyle, ButtonVariantId},
    theme::Theme,
};
use bevy::prelude::*;

/// Which look of the theme a button takes.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Default,
    /// The style the theme defines under this name. A name the theme lacks falls back to the base style.
    Named(ButtonVariantId),
}

impl From<ButtonVariantId> for ButtonVariant {
    fn from(id: ButtonVariantId) -> Self {
        Self::Named(id)
    }
}

impl ButtonVariant {
    /// Returns the style this variant selects from `theme`.
    pub(crate) fn resolve<'a>(&self, theme: &'a Theme) -> &'a ButtonStyle {
        let styles = &theme.buttons;
        match self {
            Self::Default => &styles.base,
            Self::Named(id) => styles.named.get(id).unwrap_or_else(|| {
                warn_once!(
                    "ButtonVariant {id:?} has no entry in `Theme::buttons`: using the base style"
                );
                &styles.base
            }),
        }
    }
}
