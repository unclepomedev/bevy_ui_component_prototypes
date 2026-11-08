mod builder;
mod components;
mod layer;
mod style;
mod sync;

pub use self::{
    builder::{PopupBuilder, popup},
    components::PopupRoot,
    style::{PopupMetrics, PopupShape, PopupStyle},
};

use bevy::{prelude::*, ui::UiSystems};

pub(crate) struct PopupPlugin;
impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (layer::sync_popup_layer, sync::sync_popup_style).before(UiSystems::Prepare),
        );
    }
}
