mod behavior;
mod style;
mod view;

pub use behavior::{Pressable, Pressed};
pub use style::theme::{ButtonStyle, ButtonStyles, StateColors, Theme};
pub use view::{ButtonSize, ButtonVariant, IconButton, IconButtonBuilder, icon_button};

use behavior::PressablePlugin;
use bevy::prelude::*;
use view::ButtonStylePlugin;

pub struct UiComponentPrototypesPlugin;
impl Plugin for UiComponentPrototypesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>()
            .add_plugins((PressablePlugin, ButtonStylePlugin));
    }
}
