use crate::style::{
    button::{ButtonMetrics, ButtonSizeId},
    theme::Theme,
};
use bevy::prelude::*;

/// Which footprint of the theme a button takes.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum ButtonSize {
    /// The theme's base size.
    #[default]
    Default,
    /// The size the theme defines under this name. A name the theme lacks falls back to the base size.
    Named(ButtonSizeId),
}

impl From<ButtonSizeId> for ButtonSize {
    fn from(id: ButtonSizeId) -> Self {
        Self::Named(id)
    }
}

impl ButtonSize {
    /// Returns the footprint this size selects from `theme`.
    pub(crate) fn resolve<'a>(&self, theme: &'a Theme) -> &'a ButtonMetrics {
        let sizes = &theme.button_sizes;
        match self {
            Self::Default => &sizes.base,
            Self::Named(id) => sizes.named.get(id).unwrap_or_else(|| {
                warn_once!(
                    "ButtonSize {id:?} has no entry in `Theme::button_sizes`: using the base size"
                );
                &sizes.base
            }),
        }
    }
}
