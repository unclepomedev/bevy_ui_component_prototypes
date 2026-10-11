use super::IconButton;
use bevy::{input_focus::tab_navigation::TabIndex, prelude::*, ui::InteractionDisabled};

/// The `TabIndex` an [`IconButton`] had before it was disabled.
#[derive(Component)]
struct SavedTabIndex(i32);

/// Keeps disabled buttons out of sequential Tab navigation.
///
/// Bevy's Tab navigation does not look at [`InteractionDisabled`], so a disabled button would still take a Tab stop.
/// It leaves the tab order (`TabIndex(-1)`) and gets its previous index back when it is enabled again.
pub(crate) struct TabOrderPlugin;
impl Plugin for TabOrderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(leave_tab_order)
            .add_observer(restore_tab_order);
    }
}

fn leave_tab_order(
    added: On<Add<InteractionDisabled>>,
    mut buttons: Query<&mut TabIndex, With<IconButton>>,
    mut commands: Commands,
) {
    let Ok(mut tab_index) = buttons.get_mut(added.entity) else {
        return;
    };
    // A negative index is already outside the tab order; there is nothing to restore later.
    if tab_index.0 >= 0 {
        commands
            .entity(added.entity)
            .insert(SavedTabIndex(tab_index.0));
        tab_index.0 = -1;
    }
}

fn restore_tab_order(
    removed: On<Remove<InteractionDisabled>>,
    saved: Query<&SavedTabIndex>,
    mut commands: Commands,
) {
    if let Ok(saved) = saved.get(removed.entity) {
        commands
            .entity(removed.entity)
            .try_insert(TabIndex(saved.0))
            .try_remove::<SavedTabIndex>();
    }
}
