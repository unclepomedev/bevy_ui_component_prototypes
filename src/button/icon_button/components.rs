use crate::{
    button::{ButtonMetrics, ButtonShape, ButtonSize, ButtonVariant, button_node},
    common::{focus::hidden_outline, tab_order::KeyboardNavigable},
};
use bevy::{
    input_focus::tab_navigation::TabIndex, picking::hover::PickingInteraction, prelude::*,
    ui_widgets::Button,
};

/// A themed button with an optional icon and a text label.
///
/// Behavior comes from [`Button`]: it emits [`bevy::ui_widgets::Activate`] on click and on Enter/Space while focused,
/// and uses [`bevy::ui::InteractionDisabled`].
///
/// See the [Conventions](crate#conventions) for the Tab key and the focus ring.
///
/// Its height, padding, border, and corners come from the theme (`Theme::button_sizes` for
/// [`ButtonSize`], `ButtonStyle::shape` for the variant), so they follow a theme change.
#[derive(Component)]
#[require(
    Button,
    PickingInteraction,
    TabIndex(0),
    KeyboardNavigable,
    ButtonVariant,
    ButtonSize,
    Outline = hidden_outline(),
    Node = button_node(&ButtonMetrics::default(), &ButtonShape::default())
)]
pub struct IconButton;
