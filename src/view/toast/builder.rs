use super::{
    components::{Toast, ToastHost, ToastLabel, ToastStack},
    variant::ToastVariant,
};
use crate::{style::toast::ToastMetrics, view::pass_through};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};

type Extra = Box<dyn FnOnce(&mut EntityWorldMut) + Send + Sync>;

pub struct ToastBuilder {
    text: String,
    variant: ToastVariant,
    duration: Option<f32>,
    label_extras: Vec<Extra>,
}

/// A toast. The text wraps at the theme's `max_width`.
pub fn toast() -> ToastBuilder {
    ToastBuilder {
        text: String::new(),
        variant: ToastVariant::default(),
        duration: None,
        label_extras: Vec::new(),
    }
}

impl ToastBuilder {
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn variant(mut self, variant: impl Into<ToastVariant>) -> Self {
        self.variant = variant.into();
        self
    }

    /// Seconds the toast stays fully visible; the theme's `default_duration` when not given.
    pub fn duration(mut self, seconds: f32) -> Self {
        self.duration = Some(seconds);
        self
    }

    /// Adds `bundle` to the text entity. Can be called more than once.
    pub fn label_bundle(mut self, bundle: impl Bundle) -> Self {
        self.label_extras.push(Box::new(move |label| {
            label.insert(bundle);
        }));
        self
    }

    pub fn build(self) -> impl Bundle {
        let Self {
            text,
            variant,
            duration,
            label_extras,
        } = self;
        let font_size = ToastMetrics::default().font_size;

        (
            Toast {
                elapsed: 0.0,
                duration,
            },
            variant,
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                let mut label = parent.spawn((
                    ToastLabel,
                    Text::new(text),
                    TextFont {
                        font_size,
                        ..default()
                    },
                    // Unseen until the first style sync.
                    TextColor(Color::NONE),
                    pass_through(),
                ));
                for extra in label_extras {
                    extra(&mut label);
                }
            })),
        )
    }
}

pub struct ToastHostBuilder {
    host: ToastHost,
}

/// A host with the defaults of `ToastHost`. Add a `Node` to place it.
pub fn toast_host() -> ToastHostBuilder {
    ToastHostBuilder {
        host: ToastHost::default(),
    }
}

impl ToastHostBuilder {
    pub fn max(mut self, max: usize) -> Self {
        self.host.max = max;
        self
    }

    pub fn stack(mut self, stack: ToastStack) -> Self {
        self.host.stack = stack;
        self
    }

    pub fn slide(mut self, slide: Val2) -> Self {
        self.host.slide = slide;
        self
    }

    pub fn build(self) -> impl Bundle {
        self.host
    }
}
