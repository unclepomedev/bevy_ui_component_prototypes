use crate::style::toast::ToastVariantId;
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
