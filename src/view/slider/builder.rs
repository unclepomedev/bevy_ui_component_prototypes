use super::{
    ValueSlider,
    components::{SliderPart, fill_node, rail_node, slider_node, thumb_node, travel_node},
};
use crate::{SliderShape, view::pass_through};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
    ui_widgets::{SliderRange, SliderStep, SliderThumb, SliderValue},
};

/// Starts a [`ValueSlider`] over `0.0..=1.0`, at 0.
pub fn slider() -> SliderBuilder {
    SliderBuilder {
        range: (0.0, 1.0),
        value: None,
        step: None,
    }
}

pub struct SliderBuilder {
    range: (f32, f32),
    value: Option<f32>,
    step: Option<f32>,
}

impl SliderBuilder {
    pub fn range(mut self, start: f32, end: f32) -> Self {
        self.range = (start, end);
        self
    }

    /// The initial value; the range start when not given. Later changes are the app's to apply.
    pub fn value(mut self, value: f32) -> Self {
        self.value = Some(value);
        self
    }

    /// What one key press moves by; 1% of the range when not given.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn build(self) -> impl Bundle {
        let (start, end) = self.range;
        let geometry = SliderShape::default().geometry();
        (
            ValueSlider,
            SliderRange::new(start, end),
            SliderValue(self.value.unwrap_or(start)),
            SliderStep(self.step.unwrap_or((end - start) / 100.0)),
            slider_node(&geometry),
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                parent.spawn((
                    SliderPart::Rail,
                    rail_node(&geometry),
                    BackgroundGradient::default(),
                    pass_through(),
                ));
                parent
                    .spawn((SliderPart::Travel, travel_node(&geometry), pass_through()))
                    .with_children(|travel| {
                        travel.spawn((
                            SliderPart::Fill,
                            fill_node(&geometry),
                            BackgroundGradient::default(),
                            pass_through(),
                        ));
                        travel.spawn((SliderPart::Thumb, SliderThumb, thumb_node(&geometry)));
                    });
            })),
        )
    }
}
