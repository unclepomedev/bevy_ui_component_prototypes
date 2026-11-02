use super::style::{ProgressStyle, ProgressVariantId};
use crate::{common::choice::Choice, theme::Theme};
use bevy::prelude::*;

/// Which look of the theme a progress bar takes.
pub type ProgressVariant = Choice<ProgressVariantId>;

impl ProgressVariant {
    pub(crate) fn resolve<'a>(&self, theme: &'a Theme) -> &'a ProgressStyle {
        let styles = &theme.progress;
        self.select(&styles.base, &styles.named, |id| {
            warn_once!(
                "ProgressVariant {id:?} has no entry in `Theme::progress`: using the base style"
            );
        })
    }
}
