mod components;
mod events;
mod helpers;
mod systems;

pub use components::Pressable;
pub use events::Activated;
pub(crate) use helpers::pass_through;

use self::systems::on_pressable_click;
use bevy::prelude::*;

// TODO: keyboard/gamepad activation (focus + focus ring)
// see: https://github.com/unclepomedev/bevy_ui_component_prototypes/pull/1#discussion_r4235936618
pub(crate) struct PressablePlugin;
impl Plugin for PressablePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_pressable_click);
    }
}
