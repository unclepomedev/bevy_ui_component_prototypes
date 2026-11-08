mod accessibility;
mod builder;
mod choose;
mod components;
mod events;
mod focus;
mod layer;
mod style;
mod sync;

pub use self::{
    builder::{PopupBuilder, popup},
    components::{PopupChoice, PopupRoot},
    events::PopupChosen,
    style::{PopupMetrics, PopupShape, PopupStyle},
};

use bevy::{prelude::*, ui::UiSystems};

pub(crate) struct PopupPlugin;
impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(focus::keep_focus_in_popup).add_systems(
            PostUpdate,
            (
                layer::sync_popup_layer,
                sync::sync_popup_style,
                sync::sync_popup_buttons,
                focus::focus_front_popup.after(layer::sync_popup_layer),
            )
                .before(UiSystems::Prepare),
        );
    }
}
