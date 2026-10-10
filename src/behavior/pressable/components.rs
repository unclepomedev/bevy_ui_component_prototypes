use bevy::{picking::hover::PickingInteraction, prelude::*};

/// Presentation-agnostic interaction surface.
/// Fires [super::Activated] on click unless [`bevy::ui::InteractionDisabled`] is present.
#[derive(Component, Default)]
#[require(PickingInteraction)]
pub struct Pressable;
