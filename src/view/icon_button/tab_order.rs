use super::IconButton;
use bevy::{
    input_focus::tab_navigation::TabIndex,
    prelude::*,
    ui::{InteractionDisabled, auto_directional_navigation::AutoDirectionalNavigation},
};

/// The `TabIndex` an [`IconButton`] had before it was disabled.
#[derive(Component)]
struct SavedTabIndex(i32);

/// The [`AutoDirectionalNavigation`] an [`IconButton`] had before it was disabled.
#[derive(Component)]
struct SavedAutoNavigation(AutoDirectionalNavigation);

/// Keeps a disabled [`IconButton`] out of keyboard navigation until it is enabled again.
///
/// - Tab: `TabIndex` is -1 while disabled, and the previous value is restored.
/// - Arrows: [`AutoDirectionalNavigation`] is removed while disabled and put back if the button had it.
///   One added to a button that is already disabled stays.
pub(crate) struct TabOrderPlugin;
impl Plugin for TabOrderPlugin {
    fn build(&self, app: &mut App) {
        // Neither navigation looks at `InteractionDisabled`, so the buttons have to be taken out by hand.
        app.add_observer(leave_tab_order)
            .add_observer(restore_tab_order)
            .add_observer(leave_directional_navigation)
            .add_observer(restore_directional_navigation);
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

fn leave_directional_navigation(
    added: On<Add<InteractionDisabled>>,
    buttons: Query<&AutoDirectionalNavigation, With<IconButton>>,
    mut commands: Commands,
) {
    if let Ok(navigation) = buttons.get(added.entity) {
        commands
            .entity(added.entity)
            .insert(SavedAutoNavigation(*navigation))
            .remove::<AutoDirectionalNavigation>();
    }
}

fn restore_directional_navigation(
    removed: On<Remove<InteractionDisabled>>,
    saved: Query<&SavedAutoNavigation>,
    mut commands: Commands,
) {
    if let Ok(saved) = saved.get(removed.entity) {
        commands
            .entity(removed.entity)
            .try_insert(saved.0)
            .try_remove::<SavedAutoNavigation>();
    }
}
