use crate::style::{
    button::{ButtonMetrics, ButtonShape, ButtonSizeId},
    layout,
    theme::Theme,
};
use crate::view::{KeyboardNavigable, button::ButtonVariant, focus::hidden_outline};
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
    Node = icon_button_node(&ButtonMetrics::default(), &ButtonShape::default())
)]
pub struct IconButton;

/// Which footprint of the theme a button takes.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum ButtonSize {
    /// The theme's base size.
    #[default]
    Default,
    /// The size the theme defines under this name. A name the theme lacks falls back to the base size.
    Named(ButtonSizeId),
}

impl From<ButtonSizeId> for ButtonSize {
    fn from(id: ButtonSizeId) -> Self {
        Self::Named(id)
    }
}

impl ButtonSize {
    pub(crate) fn resolve<'a>(&self, theme: &'a Theme) -> &'a ButtonMetrics {
        let sizes = &theme.button_sizes;
        match self {
            Self::Default => &sizes.base,
            Self::Named(id) => sizes.named.get(id).unwrap_or_else(|| {
                warn_once!(
                    "ButtonSize {id:?} has no entry in `Theme::button_sizes`: using the base size"
                );
                &sizes.base
            }),
        }
    }
}

/// The layout of a button: what the size and the shape decide.
pub(crate) fn icon_button_node(metrics: &ButtonMetrics, shape: &ButtonShape) -> Node {
    Node {
        height: metrics.height,
        padding: metrics.padding,
        column_gap: metrics.column_gap,
        border: UiRect::all(Val::Px(shape.border_width)),
        border_radius: BorderRadius::all(Val::Px(shape.radius)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..layout::row_center()
    }
}
