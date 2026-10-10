use bevy::{picking::hover::PickingInteraction, prelude::*};

/// Presentation-agnostic interaction surface.
/// Fires `Pressed` on click unless `ButtonDisabled` is present.
#[derive(Component, Default)]
#[require(PickingInteraction)]
pub struct Pressable;
