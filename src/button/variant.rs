use super::style::{ButtonStyle, ButtonVariantId};
use crate::{common::choice::Choice, theme::Theme};
use bevy::prelude::*;

/// Which look of the theme a button takes.
pub type ButtonVariant = Choice<ButtonVariantId>;

impl ButtonVariant {
    pub(crate) fn resolve<'a>(&self, theme: &'a Theme) -> &'a ButtonStyle {
        let styles = &theme.buttons;
        self.select(&styles.base, &styles.named, |id| {
            warn_once!(
                "ButtonVariant {id:?} has no entry in `Theme::buttons`: using the base style"
            );
        })
    }
}
