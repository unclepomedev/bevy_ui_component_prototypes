use super::style::{TooltipMetrics, TooltipShape};
use crate::common::picking::pass_through;
use accesskit::{Node as AccessKitNode, Role};
use bevy::{
    a11y::AccessibilityNode,
    picking::Pickable,
    prelude::*,
    ui::OverrideClip,
    ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide},
};

/// How close to the window edge a tooltip may go, in pixels.
const WINDOW_MARGIN: f32 = 4.0;

/// A text that appears next to a widget while the pointer rests on it, or while it holds the visible
/// keyboard focus. Add it to any entity: a button, a slider, a text field, or one of your own.
///
/// - The tooltip shows after a delay (the theme's, or [`Tooltip::delay`]) and hides when the pointer leaves,
///   when the focus moves away, when a pointer button is pressed, or when Escape is pressed.
/// - The keyboard focus counts as long as its indicator is visible (see `FocusIndicator`).
/// - Only the nearest `Tooltip` up from the hovered or focused entity shows, so a child can have its own.
/// - A disabled widget still shows its tooltip: that is the place to say why it is disabled.
/// - Changing the component changes the tooltip on screen. Where the entity has an `AccessibilityNode`, the text
///   is also its description, so that screen readers get it without hovering.
///
/// Add the component by itself; the crate spawns the tooltip as a child of the widget when it shows,
/// and despawns it when it hides.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Tooltip {
    pub text: String,
    /// Where the tooltip goes. It goes to the opposite side when there is no room.
    pub side: PopoverSide,
    /// Seconds to wait before showing. `None` takes the theme's `delay`.
    pub delay: Option<f32>,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            side: PopoverSide::default(),
            delay: None,
        }
    }

    pub fn side(mut self, side: PopoverSide) -> Self {
        self.side = side;
        self
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = Some(seconds);
        self
    }
}

/// The box that shows a tooltip. The child of the widget it is for.
#[derive(Component, Clone, Copy)]
#[require(
    Popover = placement(PopoverSide::default(), &TooltipMetrics::default()),
    GlobalZIndex = GlobalZIndex(TooltipMetrics::default().layer_gap),
    OverrideClip,
    Pickable = pass_through(),
    BackgroundColor = BackgroundColor(Color::NONE),
    BorderColor = BorderColor::all(Color::NONE),
    AccessibilityNode = popup_accessibility(""),
    Node = popup_node(&TooltipMetrics::default(), &TooltipShape::default())
)]
pub(crate) struct TooltipPopup {
    /// The widget the tooltip is for.
    pub(crate) target: Entity,
}

/// The text inside a [`TooltipPopup`].
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct TooltipLabel;

/// Spawns the tooltip of `target` as its child, ready for the sync to give it its look and its text.
pub(crate) fn spawn_popup(commands: &mut Commands, target: Entity) -> Entity {
    commands
        .spawn((
            TooltipPopup { target },
            ChildOf(target),
            children![(TooltipLabel, Text::new(String::new()), pass_through())],
        ))
        .id()
}

/// The popup floats over what is below it, shrunk to its text.
pub(crate) fn popup_node(metrics: &TooltipMetrics, shape: &TooltipShape) -> Node {
    Node {
        position_type: PositionType::Absolute,
        max_width: metrics.max_width,
        padding: metrics.padding,
        border: UiRect::all(Val::Px(shape.border_width)),
        border_radius: BorderRadius::all(Val::Px(shape.radius)),
        ..default()
    }
}

/// On `side` of the widget and centered on it, or on the opposite side when there is no room.
pub(crate) fn placement(side: PopoverSide, metrics: &TooltipMetrics) -> Popover {
    let at = |side, align| PopoverPlacement {
        side,
        align,
        gap: metrics.gap,
    };
    let aligns = [PopoverAlign::Center, PopoverAlign::Start, PopoverAlign::End];
    Popover {
        positions: [side, side.mirror()]
            .into_iter()
            .flat_map(|side| aligns.map(|align| at(side, align)))
            .collect(),
        window_margin: WINDOW_MARGIN,
    }
}

/// A tooltip for a screen reader, with the text it shows.
pub(crate) fn popup_accessibility(text: &str) -> AccessibilityNode {
    let mut node = AccessKitNode::new(Role::Tooltip);
    node.set_label(text);
    AccessibilityNode(node)
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tooltip_starts_below_with_the_theme_delay() {
        let tooltip = Tooltip::new("Save");
        assert_eq!(tooltip.text, "Save");
        assert_eq!(tooltip.side, PopoverSide::Bottom);
        assert_eq!(tooltip.delay, None);
    }

    #[test]
    fn the_options_chain() {
        let tooltip = Tooltip::new("Save").side(PopoverSide::Right).delay(0.1);
        assert_eq!(tooltip.side, PopoverSide::Right);
        assert_eq!(tooltip.delay, Some(0.1));
    }

    #[test]
    fn the_placement_tries_every_align_then_the_opposite_side() {
        let metrics = TooltipMetrics::default();
        let popover = placement(PopoverSide::Top, &metrics);
        let tried: Vec<_> = popover
            .positions
            .iter()
            .map(|placement| (placement.side, placement.align))
            .collect();
        assert_eq!(
            tried,
            [
                (PopoverSide::Top, PopoverAlign::Center),
                (PopoverSide::Top, PopoverAlign::Start),
                (PopoverSide::Top, PopoverAlign::End),
                (PopoverSide::Bottom, PopoverAlign::Center),
                (PopoverSide::Bottom, PopoverAlign::Start),
                (PopoverSide::Bottom, PopoverAlign::End),
            ]
        );
        assert!(
            popover
                .positions
                .iter()
                .all(|placement| placement.gap == metrics.gap)
        );
    }
}
