mod accessibility;
mod builder;
mod choose;
mod components;
mod events;
mod focus;
mod style;
mod sync;

pub use self::{
    builder::{PopupBuilder, popup},
    components::{PopupChoice, PopupRoot},
    events::PopupChosen,
    style::{PopupMetrics, PopupShape, PopupStyle},
};

use self::{
    focus::{focus_front_popup, keep_focus_in_popup},
    sync::{sync_popup_buttons, sync_popup_style},
};
use bevy::{prelude::*, ui::UiSystems};

pub(crate) struct PopupPlugin;
impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(keep_focus_in_popup).add_systems(
            PostUpdate,
            (sync_popup_style, sync_popup_buttons, focus_front_popup).before(UiSystems::Prepare),
        );
    }
}
