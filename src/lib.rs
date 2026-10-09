pub mod behavior;
mod style;
pub mod view;

use self::behavior::PressablePlugin;
use bevy::prelude::*;

pub struct UiComponentPrototypesPlugin;
impl Plugin for UiComponentPrototypesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PressablePlugin);
    }
}
