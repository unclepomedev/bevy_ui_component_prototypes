use super::{
    button::{ButtonSizes, ButtonStyles},
    slider::SliderStyle,
    switch::SwitchStyle,
    toast::{ToastMetrics, ToastStyles},
};
use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default)]
pub struct Theme {
    pub buttons: ButtonStyles,
    pub button_sizes: ButtonSizes,
    pub switch: SwitchStyle,
    pub slider: SliderStyle,
    pub toasts: ToastStyles,
    pub toast: ToastMetrics,
    pub font: FontSource,
    pub font_smoothing: FontSmoothing,
}

#[derive(Resource, Deref, DerefMut, Default, Clone)]
pub struct GlobalTheme(pub Theme);
