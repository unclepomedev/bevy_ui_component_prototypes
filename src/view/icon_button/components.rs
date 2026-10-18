use crate::style::{
    font_size::{FONT_SIZE_LG, FONT_SIZE_MD, FONT_SIZE_SM},
    layout,
    spacing::{SPACING_LG, SPACING_MD, SPACING_SM},
};
use crate::view::button::ButtonVariant;
use bevy::{
    input_focus::tab_navigation::TabIndex, picking::hover::PickingInteraction, prelude::*,
    ui_widgets::Button,
};

/// A themed button with an optional icon and a text label.
///
/// Behavior comes from [`Button`]: it emits [`bevy::ui_widgets::Activate`] on click and on Enter/Space while focused,
/// uses [`bevy::ui::InteractionDisabled`], and exposes a button role to accessibility tools.
/// `TabIndex(0)` makes it reachable with the Tab key when it is inside a `TabGroup`.
/// A focus ring is drawn through `Outline` while the focus indicator is visible.
#[derive(Component)]
#[require(
    Button,
    PickingInteraction,
    TabIndex(0),
    ButtonVariant,
    Outline = hidden_outline(),
    Node = default_icon_button_node()
)]
pub struct IconButton;

/// Coherent footprint of an icon button: outer height, inner padding/gap,
/// label font size, and icon box all scale together.
#[derive(Clone, Copy, Debug)]
pub struct ButtonSize {
    pub height: Val,
    pub padding: UiRect,
    pub column_gap: Val,
    pub font_size: FontSize,
    pub icon_size: f32,
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::large()
    }
}

impl ButtonSize {
    pub fn large() -> Self {
        Self {
            height: Val::Px(80.0),
            padding: UiRect::axes(SPACING_LG, SPACING_SM),
            column_gap: SPACING_SM,
            font_size: FONT_SIZE_LG,
            icon_size: 40.0,
        }
    }

    pub fn medium() -> Self {
        Self {
            height: Val::Px(56.0),
            padding: UiRect::axes(SPACING_MD, SPACING_SM),
            column_gap: SPACING_SM,
            font_size: FONT_SIZE_MD,
            icon_size: 28.0,
        }
    }

    pub fn small() -> Self {
        Self {
            height: Val::Px(40.0),
            padding: UiRect::all(SPACING_SM),
            column_gap: SPACING_SM,
            font_size: FONT_SIZE_SM,
            icon_size: 20.0,
        }
    }
}

/// Border and corner radius are intentionally constant (chrome, not scale).
pub(crate) fn icon_button_node(size: &ButtonSize) -> Node {
    Node {
        height: size.height,
        padding: size.padding,
        column_gap: size.column_gap,
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(SPACING_SM),
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

fn default_icon_button_node() -> Node {
    icon_button_node(&ButtonSize::default())
}
