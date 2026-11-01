use super::components::{Select, SelectPopup, SelectTrigger};
use bevy::{
    ecs::system::SystemParam,
    input_focus::{FocusCause, InputFocus, InputFocusVisible, tab_navigation::NavAction},
    prelude::*,
    ui_widgets::{MenuAction, MenuEvent, MenuFocusState},
};

/// Present on a [`Select`] while its popup is open.
#[derive(Component, Clone, Copy, Default)]
pub(super) struct SelectOpen;

/// An option of the list was chosen, or the chosen one was chosen again: the list has done its job.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(super) struct OptionCommitted {
    #[event_target]
    pub(super) select: Entity,
}

pub(super) struct SelectOpenPlugin;
impl Plugin for SelectOpenPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_menu_event)
            .add_observer(on_option_committed);
    }
}

/// Opens and closes the popup. The events come from the trigger and from the popup, and reach the select
/// as they travel up; the select is where they stop.
fn on_menu_event(
    mut event: On<MenuEvent>,
    selects: Query<Has<SelectOpen>, With<Select>>,
    children: Query<&Children>,
    popups: Query<(), With<SelectPopup>>,
    triggers: Query<(), With<SelectTrigger>>,
    mut focus: FocusReturn,
    mut commands: Commands,
) {
    let select = event.event_target();
    let Ok(open) = selects.get(select) else {
        return;
    };
    event.propagate(false);
    let Some(popup) = descendant_with(select, &children, &popups) else {
        return;
    };

    match event.action {
        MenuAction::Open(nav) if !open => open_popup(&mut commands, select, popup, nav),
        MenuAction::Toggle if !open => open_popup(&mut commands, select, popup, NavAction::First),
        MenuAction::Toggle | MenuAction::CloseAll if open => {
            close_popup(&mut commands, select, popup);
        }
        MenuAction::FocusRoot => {
            if let Some(trigger) = descendant_with(select, &children, &triggers) {
                focus.to(trigger);
            }
        }
        _ => {}
    }
}

/// Closes the list after a choice and gives the focus back to the trigger, leaving the focus indicator as it is:
/// it is shown if the keys were used, and hidden if the pointer was.
fn on_option_committed(
    commit: On<OptionCommitted>,
    selects: Query<Has<SelectOpen>, With<Select>>,
    children: Query<&Children>,
    popups: Query<(), With<SelectPopup>>,
    triggers: Query<(), With<SelectTrigger>>,
    mut focus: FocusReturn,
    mut commands: Commands,
) {
    let select = commit.select;
    // A choice made while the list is closed (by the app, for instance) has nothing to close, and the focus
    // is not ours to move.
    if !selects.get(select).is_ok_and(|open| open) {
        return;
    }
    let (Some(popup), Some(trigger)) = (
        descendant_with(select, &children, &popups),
        descendant_with(select, &children, &triggers),
    ) else {
        return;
    };
    close_popup(&mut commands, select, popup);
    focus.quietly_to(trigger);
}

fn descendant_with<T: Component>(
    select: Entity,
    children: &Query<&Children>,
    marker: &Query<(), With<T>>,
) -> Option<Entity> {
    children
        .iter_descendants(select)
        .find(|entity| marker.contains(*entity))
}

/// Gives the focus back to the trigger. Both resources are absent in an app without the input focus
/// plugins: there is no focus to give back then.
#[derive(SystemParam)]
struct FocusReturn<'w> {
    focus: Option<ResMut<'w, InputFocus>>,
    visible: Option<ResMut<'w, InputFocusVisible>>,
}

impl FocusReturn<'_> {
    /// Moves the focus and leaves the focus indicator as it is. Returns whether there was a focus to move.
    fn quietly_to(&mut self, trigger: Entity) -> bool {
        let Some(focus) = self.focus.as_mut() else {
            return false;
        };
        focus.set(trigger, FocusCause::Navigated);
        true
    }

    /// Moves the focus and shows the indicator. The popup asks for this when the key press closes it,
    /// as after any key.
    fn to(&mut self, trigger: Entity) {
        if self.quietly_to(trigger)
            && let Some(visible) = self.visible.as_mut()
        {
            visible.0 = true;
        }
    }
}

/// `nav` is the end of the list that takes the focus.
fn open_popup(commands: &mut Commands, select: Entity, popup: Entity, nav: NavAction) {
    commands.entity(select).insert(SelectOpen);
    commands
        .entity(popup)
        .insert((Visibility::Inherited, MenuFocusState::Opening(nav)));
}

fn close_popup(commands: &mut Commands, select: Entity, popup: Entity) {
    commands.entity(select).remove::<SelectOpen>();
    commands
        .entity(popup)
        .insert((Visibility::Hidden, MenuFocusState::Closed));
}
