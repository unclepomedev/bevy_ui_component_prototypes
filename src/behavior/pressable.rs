mod components;
mod events;
mod helpers;
mod systems;

pub use components::Pressable;
pub use events::Pressed;
pub(crate) use helpers::pass_through;

use self::systems::on_pressable_click;
use bevy::prelude::*;

pub(crate) struct PressablePlugin;
impl Plugin for PressablePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_pressable_click);
    }
}
