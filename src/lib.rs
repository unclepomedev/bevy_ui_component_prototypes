pub mod behavior;
mod style;
pub mod view;

pub use style::theme::{ColorSet, Theme};

use self::behavior::PressablePlugin;
use bevy::prelude::*;

pub struct UiComponentPrototypesPlugin;
impl Plugin for UiComponentPrototypesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>().add_plugins(PressablePlugin);
    }
}
