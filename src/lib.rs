#![warn(unreachable_pub)]
mod style;
mod view;

pub use style::theme::{
    ButtonStyle, ButtonStyles, FocusRing, GlobalTheme, SlicedImage, StateColors, Theme,
};
pub use view::{ButtonSize, ButtonVariant, IconButton, IconButtonBuilder, icon_button};

use crate::view::{ButtonStylePlugin, TabOrderPlugin};
use bevy::{
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
    prelude::*,
    ui_widgets::ButtonPlugin,
};

/// Theming for the widgets in this crate.
///
/// This plugin does not add `UiWidgetsPlugins`. Add the plugins for the behavior you use yourself:
/// - [`ButtonPlugin`] (included in `DefaultPlugins`): button activation
/// - [`InputDispatchPlugin`] (included in `DefaultPlugins`): keyboard input to the focused entity (Tab, Enter, Space)
/// - [`TabNavigationPlugin`]: Tab-key focus navigation
pub struct UiComponentPrototypesPlugin;
impl Plugin for UiComponentPrototypesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalTheme>()
            .add_plugins((ButtonStylePlugin, TabOrderPlugin));
    }

    fn finish(&self, app: &mut App) {
        let plugins = [
            ("ButtonPlugin", app.is_plugin_added::<ButtonPlugin>()),
            (
                "InputDispatchPlugin",
                app.is_plugin_added::<InputDispatchPlugin>(),
            ),
            (
                "TabNavigationPlugin",
                app.is_plugin_added::<TabNavigationPlugin>(),
            ),
        ];
        let list = |added: bool| {
            plugins
                .iter()
                .filter(|(_, is_added)| *is_added == added)
                .map(|(name, _)| format!("\n  - {name}"))
                .collect::<String>()
        };
        let missing = list(false);
        info!(
            "UiComponentPrototypesPlugin: behavior plugins\nadded:{}\nnot added:{missing}",
            list(true)
        );
    }
}
