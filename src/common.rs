pub(crate) mod bar;
pub(crate) mod bar_paint;
pub(crate) mod choice;
pub(crate) mod color;
pub(crate) mod focus;
pub(crate) mod font_size;
pub(crate) mod layout;
pub(crate) mod picking;
pub(crate) mod spacing;
pub(crate) mod state;
pub(crate) mod style_targets;
pub(crate) mod tab_order;
pub(crate) mod theme_lookup;

pub use self::{
    bar::BarSegments,
    choice::Choice,
    focus::{FocusIndicator, FocusRing, ShowsFocus},
    state::{CheckedColors, StateColors},
};

use self::{focus::ShowsFocusPlugin, tab_order::TabOrderPlugin};
use bevy::prelude::*;

pub(crate) struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TabOrderPlugin, ShowsFocusPlugin));
    }
}
