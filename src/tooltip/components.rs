use super::style::{TooltipMetrics, TooltipShape};
use crate::common::picking::pass_through;
use bevy::{
    ecs::system::EntityCommands,
    picking::Pickable,
    prelude::*,
    ui::OverrideClip,
    ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide},
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    sync::Arc,
};

const WINDOW_MARGIN: f32 = 4.0;

/// A text that appears next to a widget while the pointer rests on it, or while it holds the visible
/// keyboard focus. Add it to any entity: a button, a slider, a text field, or one of your own.
///
/// - The tooltip shows after a delay (the theme's, or [`Tooltip::delay`]) and hides when the pointer leaves,
///   when the focus moves away, when a pointer button is pressed, or when Escape is pressed.
/// - The keyboard focus counts as long as Bevy's `InputFocusVisible` is set: a click hides it, a key shows it
///   again.
/// - Only the nearest `Tooltip` up from the hovered or focused entity shows, so a child can have its own.
/// - A disabled widget still shows its tooltip: that is the place to say why it is disabled.
/// - Changing the component changes the tooltip on screen.
///
/// Add the component by itself; the crate spawns the tooltip as a child of the widget when it shows,
/// and despawns it when it hides.
///
/// For a text that has to follow something the app owns (the language, say), give the label a bundle
/// with [`Tooltip::bundle`] instead of a string.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Tooltip {
    pub text: String,
    /// Where the tooltip goes. It goes to the opposite side when there is no room.
    pub side: PopoverSide,
    /// Seconds to wait before showing. `None` takes the theme's `delay`.
    pub delay: Option<f32>,
    label: Option<LabelBundle>,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            side: PopoverSide::default(),
            delay: None,
            label: None,
        }
    }

    /// A tooltip whose text is left to `bundle`, which is added to the label each time the tooltip shows.
    /// Give it a `Text`, or a component that makes one. The crate still sets the color and the font of the
    /// text, but not its words, and `text` stays empty.
    pub fn bundle(bundle: impl Bundle + Clone) -> Self {
        Self {
            label: Some(LabelBundle::new(bundle)),
            ..Self::new(String::new())
        }
    }

    /// Whether the words come from a bundle, so that `text` is not written over them.
    pub(crate) fn has_bundle(&self) -> bool {
        self.label.is_some()
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

/// What the label of a tooltip is given when it shows. Two are equal when one is the clone of the other.
#[derive(Clone)]
struct LabelBundle(Arc<dyn Fn(&mut EntityCommands) + Send + Sync>);

impl LabelBundle {
    fn new(bundle: impl Bundle + Clone) -> Self {
        Self(Arc::new(move |label| {
            label.insert(bundle.clone());
        }))
    }
}

impl Debug for LabelBundle {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("LabelBundle")
    }
}

impl PartialEq for LabelBundle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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
    Node = popup_node(&TooltipMetrics::default(), &TooltipShape::default())
)]
pub(crate) struct TooltipPopup {
    pub(crate) target: Entity,
}

/// The text inside a [`TooltipPopup`].
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct TooltipLabel;

/// Spawns the tooltip `tooltip` of `target` as its child, ready for the sync to give it its look and its text.
pub(crate) fn spawn_popup(commands: &mut Commands, target: Entity, tooltip: &Tooltip) -> Entity {
    let popup = commands
        .spawn((TooltipPopup { target }, ChildOf(target)))
        .id();
    let mut label = commands.spawn((TooltipLabel, pass_through(), ChildOf(popup)));
    if let Some(bundle) = &tooltip.label {
        bundle.0(&mut label);
    }
    label.insert_if_new(Text::new(String::new()));
    popup
}

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
    fn a_tooltip_of_a_bundle_has_no_text_of_its_own() {
        let tooltip = Tooltip::bundle(Text::new("Save"));
        assert_eq!(tooltip.text, "");
        assert!(tooltip.has_bundle());
        assert!(!Tooltip::new("Save").has_bundle());
    }

    #[test]
    fn a_clone_is_equal_to_its_tooltip_but_another_bundle_is_not() {
        let tooltip = Tooltip::bundle(Text::new("Save"));
        assert_eq!(tooltip, tooltip.clone());
        assert_ne!(tooltip, Tooltip::bundle(Text::new("Save")));
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
