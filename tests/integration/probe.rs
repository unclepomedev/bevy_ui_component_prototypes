//! The theme every test starts from. No two of its values agree and none is a default, so a test
//! that reads a value from it cannot pass by chance, and a change of `Theme::default()` breaks nothing here.

use bevy::prelude::*;
use bevy_ui_component_prototypes::{
    Theme,
    button::{ButtonMetrics, ButtonShape, ButtonSizes, ButtonStyle, ButtonStyles, ButtonVariantId},
    common::{CheckedColors, FocusRing, StateColors},
    popup::{PopupMetrics, PopupShape, PopupStyle},
    progress::{ProgressShape, ProgressStyle, ProgressStyles, ProgressVariantId},
    scroll_indicator::{ScrollIndicatorMetrics, ScrollIndicatorStyle},
    select::{
        OptionColors, SelectMetrics, SelectPopupStyle, SelectShape, SelectStyle, SelectionColors,
    },
    slider::{SliderShape, SliderStyle},
    stepper::{StepperFill, StepperShape, StepperStyle},
    text_field::{TextFieldMetrics, TextFieldShape, TextFieldStyle},
    toast::{ToastMetrics, ToastShape, ToastStyle, ToastStyles, ToastVariantId},
    toggle_switch::SwitchStyle,
    tooltip::{TooltipMetrics, TooltipShape, TooltipStyle},
    window::{
        CloseShape, CloseStyle, TitleAlign, TitleBarStyle, WindowMetrics, WindowShape, WindowSide,
        WindowStyle,
    },
};
use std::collections::HashMap;

pub fn probe_theme() -> Theme {
    Theme {
        buttons: ButtonStyles {
            base: button_style(0.1),
            named: HashMap::from([
                (ButtonVariantId::PRIMARY, button_style(0.2)),
                (ButtonVariantId::AFFIRMATIVE, button_style(0.3)),
            ]),
        },
        button_sizes: ButtonSizes {
            base: ButtonMetrics {
                height: Val::Px(70.0),
                padding: UiRect::all(Val::Px(5.0)),
                column_gap: Val::Px(6.0),
                font_size: FontSize::Px(17.0),
                icon_size: 18.0,
            },
            named: HashMap::new(),
        },
        switch: SwitchStyle {
            track: CheckedColors {
                unchecked: tones(0.4, 0.1),
                checked: tones(0.4, 0.2),
            },
            thumb: CheckedColors {
                unchecked: tones(0.4, 0.3),
                checked: tones(0.4, 0.4),
            },
            border: tones(0.4, 0.5),
            focus_ring: ring(0.4),
        },
        slider: SliderStyle {
            shape: SliderShape {
                rail_height: 14.0,
                thumb_size: 22.0,
                rounded: true,
                segments: None,
            },
            rail: tones(0.5, 0.1),
            fill: tones(0.5, 0.2),
            thumb: tones(0.5, 0.3),
            focus_ring: ring(0.5),
            editing_ring: Color::srgb(0.5, 0.4, 0.4),
            editing_thumb: Color::srgb(0.5, 0.4, 0.5),
        },
        stepper: StepperStyle {
            shape: StepperShape {
                cell_height: 34.0,
                min_cell_width: 13.0,
                gap: 7.0,
                radius: 9.0,
                border_width: 2.0,
                mark_size: 6.0,
                fill: StepperFill::Current,
            },
            passed: tones(0.93, 0.1),
            current: tones(0.93, 0.2),
            upcoming: tones(0.93, 0.3),
            cell_border: tones(0.93, 0.35),
            button: tones(0.93, 0.4),
            arrow: tones(0.93, 0.5),
            label: tones(0.93, 0.6),
            mark: tones(0.93, 0.7),
            focus_ring: ring(0.93),
        },
        progress: ProgressStyles {
            base: progress_style(0.1),
            named: HashMap::from([
                (ProgressVariantId::PRIMARY, progress_style(0.2)),
                (ProgressVariantId::DANGER, progress_style(0.3)),
            ]),
        },
        select: SelectStyle {
            trigger: button_style(0.6),
            popup: SelectPopupStyle {
                bg: Color::srgb(0.6, 0.1, 0.1),
                border: Color::srgb(0.6, 0.9, 0.9),
                shape: SelectShape {
                    border_width: 4.0,
                    radius: 6.0,
                },
            },
            option_bg: SelectionColors {
                unselected: option_tones(0.6, 0.1),
                selected: option_tones(0.6, 0.2),
            },
            option_fg: SelectionColors {
                unselected: option_tones(0.6, 0.3),
                selected: option_tones(0.6, 0.4),
            },
        },
        select_metrics: SelectMetrics {
            trigger: ButtonMetrics {
                height: Val::Px(61.0),
                padding: UiRect::all(Val::Px(4.0)),
                column_gap: Val::Px(5.0),
                font_size: FontSize::Px(15.0),
                icon_size: 16.0,
            },
            option_height: 37.0,
            option_padding: UiRect::all(Val::Px(6.0)),
            popup_padding: UiRect::all(Val::Px(3.0)),
            popup_gap: 7.0,
            layer_gap: 42,
            max_visible_options: 2,
        },
        scroll_indicator: ScrollIndicatorStyle {
            track: Color::srgb(0.6, 0.5, 0.1),
            thumb: Color::srgb(0.6, 0.5, 0.2),
        },
        scroll_indicator_metrics: ScrollIndicatorMetrics {
            width: 5.0,
            inset: 3.0,
        },
        text_field: TextFieldStyle {
            bg: tones(0.7, 0.4),
            border: tones(0.7, 0.5),
            text: tones(0.7, 0.6),
            placeholder: Color::srgb(0.7, 0.7, 0.1),
            cursor: Color::srgb(0.7, 0.7, 0.2),
            selection: Color::srgb(0.7, 0.7, 0.3),
            unfocused_selection: Color::srgb(0.7, 0.7, 0.4),
            focus_ring: ring(0.7),
            shape: TextFieldShape {
                border_width: 2.0,
                radius: 7.0,
            },
        },
        text_field_metrics: TextFieldMetrics {
            padding: UiRect::new(Val::Px(11.0), Val::Px(12.0), Val::Px(13.0), Val::Px(14.0)),
            font_size: FontSize::Px(19.0),
        },
        toasts: ToastStyles {
            base: toast_style(0.1),
            named: HashMap::from([
                (ToastVariantId::INFO, toast_style(0.2)),
                (ToastVariantId::SUCCESS, toast_style(0.3)),
                (ToastVariantId::DANGER, toast_style(0.4)),
            ]),
        },
        toast: ToastMetrics {
            min_width: Val::Px(111.0),
            max_width: Val::Px(222.0),
            padding: UiRect::all(Val::Px(7.0)),
            font_size: FontSize::Px(13.0),
            row_gap: Val::Px(9.0),
            fade_in: 0.5,
            fade_out: 0.25,
            default_duration: 2.0,
        },
        tooltip: TooltipStyle {
            bg: Color::srgb(0.61, 0.11, 0.12),
            fg: Color::srgb(0.62, 0.52, 0.53),
            border: Color::srgb(0.63, 0.93, 0.94),
            shape: TooltipShape {
                border_width: 2.0,
                radius: 3.0,
            },
        },
        tooltip_metrics: TooltipMetrics {
            max_width: Val::Px(171.0),
            padding: UiRect::axes(Val::Px(5.0), Val::Px(3.0)),
            font_size: FontSize::Px(11.0),
            gap: 13.0,
            delay: 0.4,
            warmup: 0.25,
            layer_gap: 77,
        },
        window: WindowStyle {
            bg: Color::srgb(0.7, 0.1, 0.1),
            border: Color::srgb(0.7, 0.9, 0.9),
            shape: WindowShape {
                border_width: 3.0,
                radius: 6.0,
            },
            title_bar: TitleBarStyle {
                bg: Color::srgb(0.7, 0.2, 0.2),
                fg: Color::srgb(0.7, 0.2, 0.3),
                inactive_bg: Color::srgb(0.2, 0.7, 0.2),
                inactive_fg: Color::srgb(0.3, 0.7, 0.2),
                align: TitleAlign::Center,
            },
            close: CloseStyle {
                side: WindowSide::Left,
                shape: CloseShape::Circle,
                bg: tones(0.7, 0.1),
                glyph: tones(0.7, 0.2),
                focus_ring: FocusRing {
                    color: Color::srgb(0.9, 0.3, 0.6),
                    width: Val::Px(3.0),
                    offset: Val::Px(5.0),
                },
            },
        },
        window_metrics: WindowMetrics {
            title_height: 31.0,
            title_padding: UiRect::all(Val::Px(5.0)),
            title_gap: 9.0,
            title_font_size: FontSize::Px(19.0),
            close_width: 23.0,
            close_height: 21.0,
            glyph_size: 11.0,
            glyph_stroke: 3.0,
            resize_grip: 11.0,
        },
        popup: PopupStyle {
            scrim: Color::srgba(0.1, 0.9, 0.5, 0.4),
            bg: Color::srgb(0.1, 0.7, 0.3),
            border: Color::srgb(0.1, 0.7, 0.9),
            fg: Color::srgb(0.1, 0.5, 0.7),
            shape: PopupShape {
                border_width: 5.0,
                radius: 13.0,
            },
        },
        popup_metrics: PopupMetrics {
            min_width: Val::Px(333.0),
            max_width: Val::Px(444.0),
            padding: UiRect::all(Val::Px(17.0)),
            row_gap: Val::Px(15.0),
            button_gap: Val::Px(21.0),
            button_width: Val::Px(123.0),
            font_size: FontSize::Px(18.0),
        },
        font: FontSource::monospace(),
        font_smoothing: FontSmoothing::AntiAliased,
    }
}

/// A button style with a color for every state of every part; `seed` tells one style from another.
fn button_style(seed: f32) -> ButtonStyle {
    ButtonStyle {
        bg: tones(seed, 0.1),
        fg: tones(seed, 0.5),
        accent: Some(tones(seed, 0.7)),
        border: tones(seed, 0.9),
        focus_ring: ring(seed),
        shape: ButtonShape {
            border_width: 3.0,
            radius: 5.0,
        },
        surface: None,
        fill: None,
    }
}

/// A color for each state, none of them equal to another.
fn tones(seed: f32, part: f32) -> StateColors {
    StateColors {
        normal: Color::srgb(seed, part, 0.1),
        hovered: Color::srgb(seed, part, 0.2),
        pressed: Color::srgb(seed, part, 0.3),
        disabled: Color::srgb(seed, part, 0.4),
        focused: Color::srgb(seed, part, 0.5),
    }
}

/// The colors of an option row, none of them equal to another; `seed` and `part` tell one set from another.
fn option_tones(seed: f32, part: f32) -> OptionColors {
    OptionColors {
        normal: Color::srgb(seed, part, 0.1),
        pressed: Color::srgb(seed, part, 0.3),
        disabled: Color::srgb(seed, part, 0.4),
        highlighted: Color::srgb(seed, part, 0.5),
    }
}

fn ring(seed: f32) -> FocusRing {
    FocusRing {
        color: Color::srgb(seed, 0.7, 0.7),
        width: Val::Px(3.0),
        offset: Val::Px(4.0),
    }
}

/// A toast style with a distinct color for every part; `seed` tells one style from another.
fn toast_style(seed: f32) -> ToastStyle {
    ToastStyle {
        bg: Color::srgb(seed, 0.1, 0.1),
        fg: Color::srgb(seed, 0.5, 0.5),
        border: Color::srgb(seed, 0.9, 0.9),
        shape: ToastShape {
            border_width: 3.0,
            radius: 5.0,
        },
    }
}

/// A progress style with its own colors and a height that is not the default (12); `seed` tells one style from another.
fn progress_style(seed: f32) -> ProgressStyle {
    ProgressStyle {
        rail: Color::srgb(seed, 0.1, 0.1),
        fill: Color::srgb(seed, 0.5, 0.5),
        shape: ProgressShape {
            height: 9.0,
            rounded: true,
            segments: None,
        },
    }
}
