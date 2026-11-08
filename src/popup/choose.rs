use super::{
    components::{PopupButton, PopupRoot},
    events::PopupChosen,
};
use bevy::{prelude::*, ui_widgets::Activate};

/// Observes a popup button: activating it reports the choice on the popup it is in.
pub(super) fn choose_on_activate(
    activate: On<Activate>,
    buttons: Query<&PopupButton>,
    roots: Query<(), With<PopupRoot>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    let Some(popup) = parents
        .iter_ancestors(activate.entity)
        .find(|entity| roots.contains(*entity))
    else {
        return;
    };
    commands.trigger(PopupChosen {
        popup,
        choice: button.choice,
    });
}
