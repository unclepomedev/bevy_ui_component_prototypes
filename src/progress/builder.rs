use super::{
    ProgressBar, ProgressRange, ProgressSegments, ProgressValue,
    components::{ProgressFill, fill_node, progress_node},
    style::ProgressShape,
    variant::ProgressVariant,
};
use crate::common::{bar::BarSegments, picking::pass_through};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};

pub struct ProgressBarBuilder {
    range: ProgressRange,
    value: Option<f32>,
    variant: ProgressVariant,
    segments: Option<BarSegments>,
}

/// A progress bar over `0.0..=1.0`, at 0.
pub fn progress_bar() -> ProgressBarBuilder {
    ProgressBarBuilder {
        range: ProgressRange::default(),
        value: None,
        variant: ProgressVariant::default(),
        segments: None,
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

    /// Draws this bar as `count` blocks, instead of what the theme says. See [`ProgressSegments`].
    pub fn segments(mut self, count: u32, gap: f32) -> Self {
        self.segments = Some(BarSegments { count, gap });
        self
    }

    pub fn build(self) -> impl Bundle {
        let geometry = ProgressShape::default().geometry();
        (
            ProgressBar,
            self.range,
            ProgressValue(self.value.unwrap_or(self.range.start)),
            self.variant,
            ProgressSegments(self.segments),
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
