use super::{
    ToggleSwitch,
    components::{SwitchThumb, thumb_node, track_node},
};
use crate::view::picking::pass_through;
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};

pub struct ToggleSwitchBuilder;

/// A toggle switch that starts off. Add `bevy::ui::Checked` to the spawned bundle to start it on.
pub fn toggle_switch() -> ToggleSwitchBuilder {
    ToggleSwitchBuilder
}

impl ToggleSwitchBuilder {
    pub fn build(self) -> impl Bundle {
        (
            ToggleSwitch,
            track_node(),
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                parent.spawn((SwitchThumb, thumb_node(), pass_through()));
            })),
        )
    }
}
