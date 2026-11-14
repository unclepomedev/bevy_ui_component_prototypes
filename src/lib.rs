#![warn(unreachable_pub)]
//! Themed UI widgets for Bevy.
//!
//! The crate root only holds what every app touches: [`UiComponentPrototypesPlugin`],
//! [`GlobalTheme`] and [`Theme`]. Each widget lives in its own module with its builder,
//! components and style types:
//!
//! - [`button`], [`slider`], [`stepper`], [`select`], [`text_field`], [`toggle_switch`]
//! - [`popup`], [`progress`], [`scroll_indicator`], [`toast`], [`tooltip`], [`window`]
//! - [`common`]: types shared by several widgets (focus ring, state colors, ...)
//!
//! # Conventions
//!
//! **Controlled values.** A widget never changes its own value. It emits a
//! [`ValueChange<T>`](bevy::ui_widgets::ValueChange) with the widget as `source`, carrying the
//! requested value, and the app applies it by inserting the new value on the widget. To let a widget
//! update itself instead, add one of the observers
//! [`slider_self_update`](bevy::ui_widgets::slider_self_update),
//! [`checkbox_self_update`](bevy::ui_widgets::checkbox_self_update) or
//! [`stepper_self_update`](stepper::stepper_self_update).
//!
//! **Keyboard focus.** To reach a widget with the Tab key, put it under a `TabGroup`. The focus ring
//! is drawn with the widget's own `Outline` while the focus indicator is visible, so do not set an
//! `Outline` on it yourself.
//!
//! **Width.** A slider, a stepper and a select follow the parent layout (`100%`).
pub mod button;
pub mod common;
pub mod popup;
pub mod progress;
pub mod scroll_indicator;
pub mod select;
pub mod slider;
pub mod stepper;
pub mod text_field;
mod theme;
pub mod toast;
pub mod toggle_switch;
pub mod tooltip;
pub mod window;

pub use crate::theme::{GlobalTheme, Theme};

use crate::{
    button::ButtonStylePlugin,
    common::CommonPlugin,
    popup::PopupPlugin,
    progress::ProgressStylePlugin,
    scroll_indicator::ScrollIndicatorStylePlugin,
    select::SelectStylePlugin,
    slider::SliderStylePlugin,
    stepper::{StepperInputPlugin, StepperStylePlugin},
    text_field::TextFieldPlugin,
    toast::ToastPlugin,
    toggle_switch::SwitchStylePlugin,
    tooltip::TooltipPlugin,
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
/// This plugin does not add `UiWidgetsPlugins`. Each widget needs the Bevy plugins for its behavior;
/// add only those of the widgets you use. All of them are in `DefaultPlugins` except where noted.
///
/// | Widget | Plugins |
/// |---|---|
/// | button | [`ButtonPlugin`] |
/// | toggle switch | [`CheckboxPlugin`] |
/// | slider | [`SliderPlugin`] |
/// | stepper | [`ButtonPlugin`] |
/// | select | [`ListBoxPlugin`], [`MenuPlugin`], [`PopoverPlugin`], [`ScrollAreaPlugin`] |
/// | text field | [`TextInputPlugin`] |
/// | tooltip | [`PopoverPlugin`], and Bevy's picking for the pointer |
///
/// Optional:
/// - [`InputDispatchPlugin`]: keyboard input (Enter, Space) to the focused widget
/// - [`TabNavigationPlugin`] (not in `DefaultPlugins`): Tab-key focus navigation.
///   Without it, widgets can still be focused by a click, and a tooltip still shows on hover.
///   It is only the keyboard that cannot reach them.
pub struct UiComponentPrototypesPlugin;
impl Plugin for UiComponentPrototypesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalTheme>().add_plugins((
            ButtonStylePlugin,
            SwitchStylePlugin,
            SliderStylePlugin,
            StepperStylePlugin,
            StepperInputPlugin,
            CommonPlugin,
            ToastPlugin,
            ProgressStylePlugin,
            SelectStylePlugin,
            ScrollIndicatorStylePlugin,
            TextFieldPlugin,
            WindowStylePlugin,
            PopupPlugin,
            TooltipPlugin,
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
