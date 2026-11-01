use super::{
    ProgressBar, ProgressRange, ProgressValue,
    components::{ProgressFill, fill_node, progress_node},
    variant::ProgressVariant,
};
use crate::{style::progress::ProgressShape, view::pass_through};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};

pub struct ProgressBarBuilder {
    range: ProgressRange,
    value: Option<f32>,
    variant: ProgressVariant,
}

/// A progress bar over `0.0..=1.0`, at 0.
pub fn progress_bar() -> ProgressBarBuilder {
    ProgressBarBuilder {
        range: ProgressRange::default(),
        value: None,
        variant: ProgressVariant::default(),
    }
}

impl ProgressBarBuilder {
    pub fn range(mut self, start: f32, end: f32) -> Self {
        self.range = ProgressRange { start, end };
        self
    }

    /// The initial value; the range start when not given.
    pub fn value(mut self, value: f32) -> Self {
        self.value = Some(value);
        self
    }

    pub fn variant(mut self, variant: impl Into<ProgressVariant>) -> Self {
        self.variant = variant.into();
        self
    }

    pub fn build(self) -> impl Bundle {
        let geometry = ProgressShape::default().geometry();
        (
            ProgressBar,
            self.range,
            ProgressValue(self.value.unwrap_or(self.range.start)),
            self.variant,
            progress_node(&geometry),
            BackgroundGradient::default(),
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                parent.spawn((
                    ProgressFill,
                    fill_node(&geometry),
                    BackgroundGradient::default(),
                    pass_through(),
                ));
            })),
        )
    }
}
