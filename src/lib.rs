mod behavior;
mod style;
mod view;

pub use behavior::{Activated, Pressable};
pub use style::theme::{ButtonStyle, ButtonStyles, GlobalTheme, StateColors, Theme};
pub use view::{ButtonSize, ButtonVariant, IconButton, IconButtonBuilder, icon_button};

use crate::{behavior::PressablePlugin, view::ButtonStylePlugin};
use bevy::prelude::*;

pub struct UiComponentPrototypesPlugin;
impl Plugin for UiComponentPrototypesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalTheme>()
            .add_plugins((PressablePlugin, ButtonStylePlugin));
    }
}
