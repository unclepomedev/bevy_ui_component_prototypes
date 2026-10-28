#![warn(unreachable_pub)]
mod style;
mod view;

// Theme
pub use crate::style::{
    state::{CheckedColors, StateColors},
    theme::{GlobalTheme, Theme},
};

// Focus
pub use crate::{
    style::button::FocusRing,
    view::{FocusIndicator, ShowsFocus},
};

// Button
pub use crate::{
    style::button::{
        ButtonMetrics, ButtonShape, ButtonSizeId, ButtonSizes, ButtonStyle, ButtonStyles,
        ButtonVariantId, SlicedImage,
    },
    view::{ButtonSize, ButtonVariant, IconButton, IconButtonBuilder, icon_button},
};

// Toggle switch
pub use crate::{
    style::switch::SwitchStyle,
    view::{ToggleSwitch, ToggleSwitchBuilder, toggle_switch},
};

// Slider
pub use crate::{
    style::slider::{SliderSegments, SliderShape, SliderStyle},
    view::{SliderBuilder, SliderEditing, ValueSlider, slider},
};

// Toast
pub use crate::{
    style::toast::{ToastMetrics, ToastShape, ToastStyle, ToastStyles, ToastVariantId},
    view::{
        Toast, ToastBuilder, ToastHost, ToastHostBuilder, ToastStack, ToastVariant, toast,
        toast_host,
    },
};

use crate::view::{
    ButtonStylePlugin, ShowsFocusPlugin, SliderStylePlugin, SwitchStylePlugin, TabOrderPlugin,
    ToastPlugin,
};
use bevy::{
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
    prelude::*,
    ui_widgets::{ButtonPlugin, CheckboxPlugin, SliderPlugin},
};

/// Theming for the widgets in this crate.
///
/// This plugin does not add `UiWidgetsPlugins`. Add the plugins for the behavior you use yourself:
/// - [`ButtonPlugin`] (included in `DefaultPlugins`): button activation
/// - [`CheckboxPlugin`] (included in `DefaultPlugins`): toggle switch changes
/// - [`SliderPlugin`] (included in `DefaultPlugins`): slider changes
/// - [`InputDispatchPlugin`] (included in `DefaultPlugins`): keyboard input to the focused entity (Tab, Enter, Space)
/// - [`TabNavigationPlugin`]: Tab-key focus navigation
pub struct UiComponentPrototypesPlugin;
impl Plugin for UiComponentPrototypesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalTheme>().add_plugins((
            ButtonStylePlugin,
            SwitchStylePlugin,
            SliderStylePlugin,
            TabOrderPlugin,
            ShowsFocusPlugin,
            ToastPlugin,
        ));
    }

    fn finish(&self, app: &mut App) {
        let plugins = [
            ("ButtonPlugin", app.is_plugin_added::<ButtonPlugin>()),
            ("CheckboxPlugin", app.is_plugin_added::<CheckboxPlugin>()),
            ("SliderPlugin", app.is_plugin_added::<SliderPlugin>()),
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
