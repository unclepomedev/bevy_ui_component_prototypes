use crate::style::button::{ButtonMetrics, ButtonShape};
use crate::view::{
    KeyboardNavigable,
    button::{ButtonSize, ButtonVariant, button_node},
    focus::hidden_outline,
};
use bevy::{
    input_focus::tab_navigation::TabIndex, picking::hover::PickingInteraction, prelude::*,
    ui_widgets::Button,
};

/// A themed button with an optional icon and a text label.
///
/// Behavior comes from [`Button`]: it emits [`bevy::ui_widgets::Activate`] on click and on Enter/Space while focused,
/// uses [`bevy::ui::InteractionDisabled`], and exposes a button role to accessibility tools.
/// Text placed next to the button is not associated with it: give the button a name for screen readers with `AccessibleLabel`.
/// To reach it with the Tab key, put it under a `TabGroup`. The focus ring is drawn with the entity's
/// own `Outline` while the focus indicator is visible, so do not set an `Outline` on it yourself.
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
