use super::{components::Pressable, events::Activated};
use bevy::{prelude::*, ui::InteractionDisabled};

pub(super) fn on_pressable_click(
    trigger: On<PointerClick>,
    pressables: Query<(), (With<Pressable>, Without<InteractionDisabled>)>,
    mut commands: Commands,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }
    if pressables.get(trigger.entity).is_ok() {
        commands.trigger(Activated {
            entity: trigger.entity,
        });
    }
}
