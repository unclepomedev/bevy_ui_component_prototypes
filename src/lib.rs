#![warn(unreachable_pub)]
//! Themed UI widgets for Bevy.
//!
//! The crate root only holds what every app touches: [`UiComponentPrototypesPlugin`],
//! [`GlobalTheme`] and [`Theme`]. Each widget lives in its own module with its builder,
//! components and style types:
//!
//! - [`button`], [`slider`], [`select`], [`text_field`], [`toggle_switch`]
//! - [`popup`], [`progress`], [`scroll_indicator`], [`toast`], [`window`]
//! - [`common`]: types shared by several widgets (focus ring, state colors, ...)
pub mod button;
pub mod common;
pub mod popup;
pub mod progress;
pub mod scroll_indicator;
pub mod select;
pub mod slider;
pub mod text_field;
mod theme;
pub mod toast;
pub mod toggle_switch;
pub mod window;

pub use crate::theme::{GlobalTheme, Theme};

use crate::{
    button::ButtonStylePlugin,
    common::{focus::ShowsFocusPlugin, tab_order::TabOrderPlugin},
    popup::PopupPlugin,
    progress::ProgressStylePlugin,
    scroll_indicator::ScrollIndicatorStylePlugin,
    select::SelectStylePlugin,
    slider::SliderStylePlugin,
    text_field::{
        TextAreaPlugin, TextFieldEntryPlugin, TextFieldStylePlugin, TextFieldValuePlugin,
    },
    toast::ToastPlugin,
    toggle_switch::SwitchStylePlugin,
    window::WindowStylePlugin,
};
use bevy::{
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
    prelude::*,
    ui_widgets::{
        ButtonPlugin, CheckboxPlugin, ListBoxPlugin, MenuPlugin, ScrollAreaPlugin, SliderPlugin,
        TextInputPlugin, popover::PopoverPlugin,
    },
};

/// Theming for the widgets in this crate.
///
/// This plugin does not add `UiWidgetsPlugins`. Add the plugins for the behavior you use yourself:
/// - [`ButtonPlugin`] (included in `DefaultPlugins`): button activation
/// - [`CheckboxPlugin`] (included in `DefaultPlugins`): toggle switch changes
/// - [`SliderPlugin`] (included in `DefaultPlugins`): slider changes
/// - [`ListBoxPlugin`] (included in `DefaultPlugins`): the list of a select
/// - [`MenuPlugin`] (included in `DefaultPlugins`): opening, closing and the focus of a select
/// - [`PopoverPlugin`] (included in `DefaultPlugins`): where the popup of a select goes
/// - [`ScrollAreaPlugin`] (included in `DefaultPlugins`): scrolling the list of a select
/// - [`TextInputPlugin`] (included in `DefaultPlugins`): typing in a text field
/// - [`InputDispatchPlugin`] (included in `DefaultPlugins`): keyboard input to the focused entity (Tab, Enter, Space)
/// - [`TabNavigationPlugin`]: Tab-key focus navigation
///
/// What a window asks for ([`RaiseWindow`](window::RaiseWindow), [`MoveWindow`](window::MoveWindow), [`ResizeWindow`](window::ResizeWindow), [`CloseWindow`](window::CloseWindow)) is applied by [`WindowDefaultsPlugin`](window::WindowDefaultsPlugin), which is not included.
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
            ProgressStylePlugin,
            SelectStylePlugin,
            ScrollIndicatorStylePlugin,
            TextFieldStylePlugin,
            TextFieldValuePlugin,
            TextFieldEntryPlugin,
            TextAreaPlugin,
            WindowStylePlugin,
            PopupPlugin,
        ));
    }

    fn finish(&self, app: &mut App) {
        let plugins = [
            ("ButtonPlugin", app.is_plugin_added::<ButtonPlugin>()),
            ("CheckboxPlugin", app.is_plugin_added::<CheckboxPlugin>()),
            ("SliderPlugin", app.is_plugin_added::<SliderPlugin>()),
            ("ListBoxPlugin", app.is_plugin_added::<ListBoxPlugin>()),
            ("TextInputPlugin", app.is_plugin_added::<TextInputPlugin>()),
            ("MenuPlugin", app.is_plugin_added::<MenuPlugin>()),
            ("PopoverPlugin", app.is_plugin_added::<PopoverPlugin>()),
            (
                "ScrollAreaPlugin",
                app.is_plugin_added::<ScrollAreaPlugin>(),
            ),
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
