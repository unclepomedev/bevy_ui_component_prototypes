use crate::style::{
    button::{ButtonMetrics, ButtonShape, ButtonSizes},
    layout,
    theme::Theme,
};
use crate::view::{KeyboardNavigable, button::ButtonVariant};
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
#[derive(Component, Clone, Debug, Default)]
pub enum ButtonSize {
    Small,
    Medium,
    #[default]
    Large,
    /// A footprint of this button alone, whatever the theme says.
    Custom(Box<ButtonMetrics>),
}

impl ButtonSize {
    /// Returns the footprint this size selects from `theme`.
    pub(crate) fn resolve<'a>(&'a self, theme: &'a Theme) -> &'a ButtonMetrics {
        let ButtonSizes {
            small,
            medium,
            large,
        } = &theme.button_sizes;
        match self {
            Self::Small => small,
            Self::Medium => medium,
            Self::Large => large,
            Self::Custom(metrics) => metrics,
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

/// `Outline::default()` is a visible white line, so the ring starts hidden.
/// It is toggled through its color, which avoids table moves.
fn hidden_outline() -> Outline {
    Outline::new(Val::Px(0.0), Val::Px(0.0), Color::NONE)
}
