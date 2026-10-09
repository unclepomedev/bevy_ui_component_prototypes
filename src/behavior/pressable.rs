mod components;
mod events;
mod helpers;
mod systems;

pub(crate) use components::{ButtonColors, Pressable};
pub(crate) use helpers::pass_through;

use self::systems::{on_pressable_click, sync_pressable_visuals};
use bevy::prelude::*;

pub(crate) struct PressablePlugin;
impl Plugin for PressablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_pressable_visuals)
            .add_observer(on_pressable_click);
    }
}
