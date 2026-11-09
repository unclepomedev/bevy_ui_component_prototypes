mod builder;
mod choose;
mod components;
mod disabled;
mod highlight;
mod layer;
mod open;
mod popup;
mod scroll;
mod style;
mod trigger;

pub use self::{
    builder::{SelectBuilder, select},
    components::{Select, SelectKey, SelectedIndex, SelectedOption},
    style::{
        OptionColors, SelectMetrics, SelectPopupStyle, SelectShape, SelectStyle, SelectionColors,
    },
};

use self::{
    choose::SelectChoosePlugin, disabled::SelectDisabledPlugin, highlight::SelectHighlightPlugin,
    layer::SelectLayerPlugin, open::SelectOpenPlugin, popup::SelectPopupStylePlugin,
    scroll::SelectScrollPlugin, trigger::SelectTriggerStylePlugin,
};
use bevy::prelude::*;

pub(crate) struct SelectStylePlugin;
impl Plugin for SelectStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SelectTriggerStylePlugin,
            SelectPopupStylePlugin,
            SelectOpenPlugin,
            SelectChoosePlugin,
            SelectHighlightPlugin,
            SelectDisabledPlugin,
            SelectScrollPlugin,
            SelectLayerPlugin,
        ));
    }
}
