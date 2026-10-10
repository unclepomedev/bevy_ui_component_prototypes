use super::{components::Pressable, events::Pressed};
use bevy::{prelude::*, ui::InteractionDisabled};

pub fn on_pressable_click(
    trigger: On<PointerClick>,
    pressables: Query<(), (With<Pressable>, Without<InteractionDisabled>)>,
    mut commands: Commands,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }
    if pressables.get(trigger.entity).is_ok() {
        commands.trigger(Pressed {
            entity: trigger.entity,
        });
    }
}
