//! The theme every test starts from. No two of its values agree and none is a default, so a test
//! that reads a value from it cannot pass by chance, and a change of `Theme::default()` breaks nothing here.

use bevy::prelude::*;
use bevy_ui_component_prototypes::{
    ButtonMetrics, ButtonShape, ButtonSizes, ButtonStyle, ButtonStyles, ButtonVariantId,
    CheckedColors, FocusRing, SliderShape, SliderStyle, StateColors, SwitchStyle, Theme,
    ToastMetrics, ToastShape, ToastStyle, ToastStyles, ToastVariantId,
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
        font: FontSource::monospace(),
        font_smoothing: FontSmoothing::AntiAliased,
    }
}

/// A button style with a color for every state of every part; `seed` tells one style from another.
fn button_style(seed: f32) -> ButtonStyle {
    ButtonStyle {
        bg: tones(seed, 0.1),
        fg: tones(seed, 0.5),
        border: tones(seed, 0.9),
        focus_ring: ring(seed),
        shape: ButtonShape {
            border_width: 3.0,
            radius: 5.0,
        },
        surface: None,
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
