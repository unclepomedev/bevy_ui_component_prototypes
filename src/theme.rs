use crate::button::{ButtonSizes, ButtonStyles};
use crate::progress::ProgressStyles;
use crate::select::{SelectMetrics, SelectStyle};
use crate::slider::SliderStyle;
use crate::toast::{ToastMetrics, ToastStyles};
use crate::toggle_switch::SwitchStyle;
use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default)]
pub struct Theme {
    pub buttons: ButtonStyles,
    pub button_sizes: ButtonSizes,
    pub switch: SwitchStyle,
    pub slider: SliderStyle,
    pub progress: ProgressStyles,
    pub select: SelectStyle,
    pub select_metrics: SelectMetrics,
    pub toasts: ToastStyles,
    pub toast: ToastMetrics,
    pub font: FontSource,
    pub font_smoothing: FontSmoothing,
}

#[derive(Resource, Deref, DerefMut, Default, Clone)]
pub struct GlobalTheme(pub Theme);
