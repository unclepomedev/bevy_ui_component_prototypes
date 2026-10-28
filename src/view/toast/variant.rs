use crate::style::{
    theme::Theme,
    toast::{ToastStyle, ToastVariantId},
};
use bevy::prelude::*;

/// Which look of the theme a toast takes.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum ToastVariant {
    #[default]
    Default,
    /// The style the theme defines under this name. A name the theme lacks falls back to the base style.
    Named(ToastVariantId),
}

impl From<ToastVariantId> for ToastVariant {
    fn from(id: ToastVariantId) -> Self {
        Self::Named(id)
    }
}

impl ToastVariant {
    /// Returns the style this variant selects from `theme`.
    pub(crate) fn resolve<'a>(&self, theme: &'a Theme) -> &'a ToastStyle {
        let styles = &theme.toasts;
        match self {
            Self::Default => &styles.base,
            Self::Named(id) => styles.named.get(id).unwrap_or_else(|| {
                warn_once!(
                    "ToastVariant {id:?} has no entry in `Theme::toasts`: using the base style"
                );
                &styles.base
            }),
        }
    }
}
