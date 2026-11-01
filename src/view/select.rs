mod builder;
mod choose;
mod components;
mod highlight;
mod open;
mod popup;
mod trigger;

pub use self::{
    builder::{SelectBuilder, select},
    components::{Select, SelectKey, SelectedIndex, SelectedOption},
};

use self::{
    choose::SelectChoosePlugin, highlight::SelectHighlightPlugin, open::SelectOpenPlugin,
    popup::SelectPopupStylePlugin, trigger::SelectTriggerStylePlugin,
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
        ));
    }
}
