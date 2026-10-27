use super::{
    components::{Toast, ToastLabel},
    variant::ToastVariant,
};
use crate::{style::toast::ToastMetrics, view::pass_through};
use bevy::prelude::*;

pub struct ToastBuilder {
    text: String,
    variant: ToastVariant,
    duration: Option<f32>,
}

/// A toast with this text. The text wraps at the theme's `max_width`.
pub fn toast(text: impl Into<String>) -> ToastBuilder {
    ToastBuilder {
        text: text.into(),
        variant: ToastVariant::default(),
        duration: None,
    }
}

impl ToastBuilder {
    pub fn variant(mut self, variant: impl Into<ToastVariant>) -> Self {
        self.variant = variant.into();
        self
    }

    /// Seconds the toast stays fully visible; the theme's `default_duration` when not given.
    pub fn duration(mut self, seconds: f32) -> Self {
        self.duration = Some(seconds);
        self
    }

    pub fn build(self) -> impl Bundle {
        let metrics = ToastMetrics::default();
        (
            Toast {
                elapsed: 0.0,
                duration: self.duration,
            },
            self.variant,
            children![(
                ToastLabel,
                Text::new(self.text),
                TextFont {
                    font_size: metrics.font_size,
                    ..default()
                },
                // Unseen until the first style sync.
                TextColor(Color::NONE),
                pass_through(),
            )],
        )
    }
}
