mod accessibility;
mod builder;
mod choose;
mod components;
mod events;
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
        app.add_systems(
            PostUpdate,
            (
                layer::sync_popup_layer,
                sync::sync_popup_style,
                sync::sync_popup_buttons,
            )
                .before(UiSystems::Prepare),
        );
    }
}
