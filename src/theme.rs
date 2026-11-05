use crate::button::{ButtonSizes, ButtonStyles};
use crate::progress::ProgressStyles;
use crate::scroll_indicator::{ScrollIndicatorMetrics, ScrollIndicatorStyle};
use crate::select::{SelectMetrics, SelectStyle};
use crate::slider::SliderStyle;
use crate::text_field::{TextFieldMetrics, TextFieldStyle};
use crate::toast::{ToastMetrics, ToastStyles};
use crate::toggle_switch::SwitchStyle;
use crate::window::{WindowMetrics, WindowStyle};
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
    pub scroll_indicator: ScrollIndicatorStyle,
    pub scroll_indicator_metrics: ScrollIndicatorMetrics,
    pub text_field: TextFieldStyle,
    pub text_field_metrics: TextFieldMetrics,
    pub toasts: ToastStyles,
    pub toast: ToastMetrics,
    pub window: WindowStyle,
    pub window_metrics: WindowMetrics,
    pub font: FontSource,
    pub font_smoothing: FontSmoothing,
}

#[derive(Resource, Deref, DerefMut, Default, Clone)]
pub struct GlobalTheme(pub Theme);
