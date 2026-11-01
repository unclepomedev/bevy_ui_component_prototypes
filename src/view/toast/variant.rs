use crate::{
    style::{
        theme::Theme,
        toast::{ToastStyle, ToastVariantId},
    },
    view::choice::Choice,
};
use bevy::prelude::*;

/// Which look of the theme a toast takes.
pub type ToastVariant = Choice<ToastVariantId>;

impl ToastVariant {
    /// Returns the style this variant selects from `theme`.
    pub(crate) fn resolve<'a>(&self, theme: &'a Theme) -> &'a ToastStyle {
        let styles = &theme.toasts;
        self.select(&styles.base, &styles.named, |id| {
            warn_once!("ToastVariant {id:?} has no entry in `Theme::toasts`: using the base style");
        })
    }
}
