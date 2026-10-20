use bevy::{
    input_focus::tab_navigation::TabIndex,
    prelude::*,
    ui::{InteractionDisabled, auto_directional_navigation::AutoDirectionalNavigation},
};

/// Marks a widget that must leave keyboard navigation while it is disabled.
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct KeyboardNavigable;

/// The `TabIndex` a [`KeyboardNavigable`] widget had before it was disabled.
#[derive(Component)]
struct SavedTabIndex(i32);

/// The [`AutoDirectionalNavigation`] a [`KeyboardNavigable`] widget had before it was disabled.
#[derive(Component)]
struct SavedAutoNavigation(AutoDirectionalNavigation);

/// Keeps a disabled [`KeyboardNavigable`] widget out of keyboard navigation until it is enabled again.
///
/// - Tab: `TabIndex` is -1 while disabled, and the previous value is restored.
/// - Arrows: [`AutoDirectionalNavigation`] is removed while disabled and put back if the widget had it.
///   One added to a disabled widget is removed the same way.
///
/// A widget that holds the focus when it becomes disabled keeps it, and the arrows cannot leave it
/// (Tab can). Moving the focus first is up to the app.
pub(crate) struct TabOrderPlugin;
impl Plugin for TabOrderPlugin {
    fn build(&self, app: &mut App) {
        // Neither navigation looks at `InteractionDisabled`, so the widgets have to be taken out by hand.
        app.add_observer(leave_tab_order)
            .add_observer(restore_tab_order)
            .add_observer(leave_directional_navigation)
            .add_observer(refuse_directional_navigation)
            .add_observer(restore_directional_navigation);
    }
}

fn leave_tab_order(
    added: On<Add<InteractionDisabled>>,
    mut widgets: Query<&mut TabIndex, With<KeyboardNavigable>>,
    mut commands: Commands,
) {
    let Ok(mut tab_index) = widgets.get_mut(added.entity) else {
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
    widgets: Query<&AutoDirectionalNavigation, With<KeyboardNavigable>>,
    mut commands: Commands,
) {
    if let Ok(navigation) = widgets.get(added.entity) {
        set_aside(&mut commands, added.entity, *navigation);
    }
}

fn refuse_directional_navigation(
    added: On<Add<AutoDirectionalNavigation>>,
    widgets: Query<
        &AutoDirectionalNavigation,
        (With<KeyboardNavigable>, With<InteractionDisabled>),
    >,
    mut commands: Commands,
) {
    if let Ok(navigation) = widgets.get(added.entity) {
        set_aside(&mut commands, added.entity, *navigation);
    }
}

fn set_aside(commands: &mut Commands, widget: Entity, navigation: AutoDirectionalNavigation) {
    commands
        .entity(widget)
        .insert(SavedAutoNavigation(navigation))
        .remove::<AutoDirectionalNavigation>();
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

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TabOrderPlugin));
        app
    }

    fn tab_index(app: &App, entity: Entity) -> i32 {
        app.world().get::<TabIndex>(entity).unwrap().0
    }

    /// The plugin serves any widget that carries the marker, not only buttons.
    #[test]
    fn a_marked_widget_leaves_and_rejoins_the_tab_order() {
        let mut app = app();
        let widget = app.world_mut().spawn((KeyboardNavigable, TabIndex(2))).id();
        app.update();

        app.world_mut()
            .entity_mut(widget)
            .insert(InteractionDisabled);
        app.update();
        assert_eq!(tab_index(&app, widget), -1);

        app.world_mut()
            .entity_mut(widget)
            .remove::<InteractionDisabled>();
        app.update();
        assert_eq!(tab_index(&app, widget), 2);
    }

    #[test]
    fn a_marked_widget_gives_up_directional_navigation_while_disabled() {
        let mut app = app();
        let widget = app
            .world_mut()
            .spawn((
                KeyboardNavigable,
                TabIndex(0),
                AutoDirectionalNavigation::default(),
            ))
            .id();
        app.update();

        app.world_mut()
            .entity_mut(widget)
            .insert(InteractionDisabled);
        app.update();
        assert!(
            app.world()
                .get::<AutoDirectionalNavigation>(widget)
                .is_none()
        );

        app.world_mut()
            .entity_mut(widget)
            .remove::<InteractionDisabled>();
        app.update();
        assert!(
            app.world()
                .get::<AutoDirectionalNavigation>(widget)
                .is_some()
        );
    }

    /// Without the marker the entity is none of this plugin's business.
    #[test]
    fn an_unmarked_entity_is_left_alone() {
        let mut app = app();
        let other = app.world_mut().spawn(TabIndex(0)).id();
        app.update();

        app.world_mut()
            .entity_mut(other)
            .insert(InteractionDisabled);
        app.update();
        assert_eq!(tab_index(&app, other), 0);
    }
}
