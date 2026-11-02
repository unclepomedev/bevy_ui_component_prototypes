use super::style::{ButtonMetrics, ButtonSizeId};
use crate::{common::choice::Choice, theme::Theme};
use bevy::prelude::*;

/// Which footprint of the theme a button takes.
pub type ButtonSize = Choice<ButtonSizeId>;

impl ButtonSize {
    /// Returns the footprint this size selects from `theme`.
    pub(crate) fn resolve<'a>(&self, theme: &'a Theme) -> &'a ButtonMetrics {
        let sizes = &theme.button_sizes;
        self.select(&sizes.base, &sizes.named, |id| {
            warn_once!(
                "ButtonSize {id:?} has no entry in `Theme::button_sizes`: using the base size"
            );
        })
    }
}
