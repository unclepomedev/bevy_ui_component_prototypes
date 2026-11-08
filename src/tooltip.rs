mod components;
mod describe;
mod drive;
mod style;
mod sync;
mod timing;
mod wanted;

pub use self::{
    components::Tooltip,
    style::{TooltipMetrics, TooltipShape, TooltipStyle},
};

use self::{
    describe::{clear_description, describe_targets},
    drive::{TooltipState, drive_tooltips, forget_lost_popup},
    sync::{sync_popup_boxes, sync_popup_labels, sync_popup_layers},
    wanted::{Wanted, ask_for_tooltip, watch_dismiss},
};
use bevy::{prelude::*, ui::UiSystems};

pub(crate) struct TooltipPlugin;
impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Wanted>()
            .init_resource::<TooltipState>()
            .add_systems(
                Update,
                (
                    forget_lost_popup,
                    ask_for_tooltip,
                    watch_dismiss,
                    drive_tooltips,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_popup_boxes,
                    sync_popup_layers,
                    sync_popup_labels,
                    describe_targets,
                )
                    .before(UiSystems::Prepare),
            )
            .add_observer(clear_description);
    }
}
