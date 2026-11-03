use super::style::{SelectMetrics, SelectShape};
use crate::{
    button::{ButtonShape, button_node},
    common::{focus::hidden_outline, tab_order::KeyboardNavigable},
};
use accesskit::{HasPopup, Node as AccessKitNode, Role};
use bevy::{
    a11y::AccessibilityNode,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    input_focus::tab_navigation::TabIndex,
    picking::hover::PickingInteraction,
    prelude::*,
    ui::OverrideClip,
    ui_widgets::{
        ListBox, ListItem, MenuButton, MenuPopup, ScrollArea,
        popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide},
    },
};
use std::sync::Arc;

const ARROW_SIZE: f32 = 10.0;
const ARROW_STROKE: f32 = 2.0;
/// How close to the window edge the popup may go, in pixels.
const WINDOW_MARGIN: f32 = 8.0;

/// What the value of an option must be: `Clone + PartialEq + Send + Sync + 'static`.
/// Every such type is a `SelectKey` already; there is nothing to implement.
pub trait SelectKey: Clone + PartialEq + Send + Sync + 'static {}
impl<T: Clone + PartialEq + Send + Sync + 'static> SelectKey for T {}

/// Adds a copy of an option's label to an entity.
pub(crate) type LabelSpawner = Arc<dyn Fn(&mut EntityWorldMut) + Send + Sync>;

/// A themed select: a box that shows the chosen option, and a list that opens under it.
///
/// It is controlled, like the other widgets: the select never changes its own choice. Choosing an option in
/// the list emits `ValueChange<V>` (`V` is the type of the option values) with this entity as `source`,
/// and the app applies it by inserting [`SelectedOption`] (or [`SelectedIndex`]) on this entity.
///
/// Text placed next to the select is not associated with it: give the select a name for screen readers
/// with `AccessibleLabel` on this entity.
///
/// Insert `InteractionDisabled` on this entity to disable the select.
///
/// The width follows the parent layout (`100%`).
#[derive(Component)]
#[require(SelectedIndex, Node = select_node())]
pub struct Select;

/// Which option is chosen, by position. `None`, or a position past the last option, shows the placeholder.
/// Inserting [`SelectedOption`] sets this for you. Use one of the two, not both.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Deref)]
pub struct SelectedIndex {
    pub index: Option<usize>,
}

/// The chosen option, by value. Insert it on a [`Select`] built with the same `V`; removing it shows the placeholder.
/// A value that is not one of the options shows the placeholder too.
#[derive(Component, Clone, Debug, PartialEq)]
#[component(immutable, on_insert = index_from_value::<V>, on_remove = clear_index)]
pub struct SelectedOption<V: SelectKey> {
    pub value: V,
}

/// The values of the options, in order.
#[derive(Component)]
pub(crate) struct SelectOptions<V: SelectKey> {
    pub(crate) values: Vec<V>,
}

/// What the select can show: the label of each option, and the placeholder.
#[derive(Component)]
pub(crate) struct SelectLabels {
    pub(crate) options: Vec<LabelSpawner>,
    pub(crate) placeholder: Option<LabelSpawner>,
}

/// Triggers the choice of the option at a position, given the select and the position.
pub(crate) type EmitChoice = Arc<dyn Fn(&mut Commands, Entity, usize) + Send + Sync>;

/// Emits the choice of the option at a position: a `ValueChange<V>` from the select. The type of the
/// values is gone from this type, so nothing has to be registered for each of them.
#[derive(Component)]
pub(crate) struct SelectRequest {
    pub(crate) emit: EmitChoice,
}

/// What the builder was given as the initial choice. It becomes a [`SelectedOption`] when the select is
/// spawned, and goes away then.
#[derive(Component)]
#[component(on_add = choose_initial_option::<V>)]
pub(crate) struct InitialOption<V: SelectKey> {
    pub(crate) value: Option<V>,
}

fn choose_initial_option<V: SelectKey>(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;
    let value = world
        .get::<InitialOption<V>>(entity)
        .and_then(|initial| initial.value.clone())
        .filter(|value| {
            world
                .get::<SelectOptions<V>>(entity)
                .is_some_and(|select| select.values.contains(value))
        });
    let mut commands = world.commands();
    let mut select = commands.entity(entity);
    select.try_remove::<InitialOption<V>>();
    if let Some(value) = value {
        select.try_insert(SelectedOption { value });
    }
}

fn index_from_value<V: SelectKey>(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;
    let Some(selected) = world.get::<SelectedOption<V>>(entity) else {
        return;
    };
    let Some(select) = world.get::<SelectOptions<V>>(entity) else {
        warn_once!("SelectedOption was inserted on an entity that has no options of that type");
        return;
    };
    let index = select
        .values
        .iter()
        .position(|option| *option == selected.value);
    if index.is_none() {
        warn_once!(
            "SelectedOption is not one of the options of its Select: showing the placeholder"
        );
    }
    world
        .commands()
        .entity(entity)
        .try_insert(SelectedIndex { index });
}

fn clear_index(mut world: DeferredWorld, context: HookContext) {
    world
        .commands()
        .entity(context.entity)
        .try_insert(SelectedIndex { index: None });
}

/// The part of a select that takes the focus and the pointer. Its look is the theme's `SelectStyle::trigger`.
#[derive(Component, Clone, Copy, Default)]
#[require(
    MenuButton,
    // `MenuButton` asks for the button role; a select is a combo box.
    AccessibilityNode = trigger_accessibility(),
    PickingInteraction,
    TabIndex(0),
    KeyboardNavigable,
    Outline = hidden_outline(),
    Node = trigger_node(&SelectMetrics::default(), &ButtonShape::default())
)]
pub(crate) struct SelectTrigger;

/// Where the label of the chosen option (or the placeholder) is put.
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct SelectContent;

/// The chevron at the end of the trigger.
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct SelectArrow;

/// The list that opens under the trigger. Shown only while the select is open.
/// It ignores the clipping of its ancestors, so that it can open past the edge of a clipped container.
#[derive(Component, Clone, Copy, Default)]
#[require(
    MenuPopup,
    // `MenuPopup` asks for the menu role; this is only a wrapper around the list.
    AccessibilityNode = popup_accessibility(),
    Popover = popup_placement(&SelectMetrics::default()),
    GlobalZIndex = GlobalZIndex(SelectMetrics::default().popup_z_index),
    OverrideClip,
    Visibility = Visibility::Hidden,
    BackgroundColor = BackgroundColor(Color::NONE),
    BorderColor = BorderColor::all(Color::NONE),
    Node = popup_node(&SelectMetrics::default(), &SelectShape::default())
)]
pub(crate) struct SelectPopup;

/// The options inside the popup. It holds the focus while the popup is open, and scrolls when there are more
/// options than the theme says it shows.
#[derive(Component, Clone, Copy, Default)]
#[require(ListBox, ScrollArea, TabIndex(0), Node = list_node(&SelectMetrics::default()))]
pub(crate) struct SelectList;

/// One option of the list. `index` is its position among the options.
#[derive(Component, Clone, Copy)]
#[require(
    ListItem,
    PickingInteraction,
    BackgroundColor = BackgroundColor(Color::NONE),
    Node = row_node(&SelectMetrics::default())
)]
pub(crate) struct SelectRow {
    pub(crate) index: usize,
}

/// A combo box that opens a list, closed to start with.
pub(crate) fn trigger_accessibility() -> AccessibilityNode {
    let mut node = AccessKitNode::new(Role::ComboBox);
    node.set_has_popup(HasPopup::Listbox);
    node.set_expanded(false);
    AccessibilityNode(node)
}

/// A plain wrapper around the list, out of the way of a screen reader until the select opens.
pub(crate) fn popup_accessibility() -> AccessibilityNode {
    let mut node = AccessKitNode::new(Role::GenericContainer);
    node.set_hidden();
    AccessibilityNode(node)
}

pub(crate) fn select_node() -> Node {
    Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

/// The popup floats over what is below it; the size and the shape come from the theme.
/// It is as wide as the select at least.
pub(crate) fn popup_node(metrics: &SelectMetrics, shape: &SelectShape) -> Node {
    Node {
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Column,
        min_width: percent(100),
        padding: metrics.popup_padding,
        border: UiRect::all(px(shape.border_width)),
        border_radius: BorderRadius::all(px(shape.radius)),
        ..default()
    }
}

/// Below the select with the left edges together, or above it when there is no room below.
pub(crate) fn popup_placement(metrics: &SelectMetrics) -> Popover {
    let placement = |side| PopoverPlacement {
        side,
        align: PopoverAlign::Start,
        gap: metrics.popup_gap,
    };
    Popover {
        positions: vec![placement(PopoverSide::Bottom), placement(PopoverSide::Top)],
        window_margin: WINDOW_MARGIN,
    }
}

pub(crate) fn list_node(metrics: &SelectMetrics) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        overflow: Overflow::scroll_y(),
        max_height: px(metrics.option_height * metrics.max_visible_options.max(1) as f32),
        ..default()
    }
}

pub(crate) fn row_node(metrics: &SelectMetrics) -> Node {
    Node {
        height: px(metrics.option_height),
        padding: metrics.option_padding,
        align_items: AlignItems::Center,
        // The rows keep their height in a list that is shorter than all of them.
        flex_shrink: 0.0,
        ..default()
    }
}

/// The trigger is a button that fills the select, with the label at one end and the chevron at the other.
pub(crate) fn trigger_node(metrics: &SelectMetrics, shape: &ButtonShape) -> Node {
    Node {
        width: percent(100),
        justify_content: JustifyContent::SpaceBetween,
        ..button_node(&metrics.trigger, shape)
    }
}

pub(crate) fn content_node() -> Node {
    Node {
        flex_grow: 1.0,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// A square with only two sides drawn, turned by 45 degrees: a chevron that needs no glyph.
pub(crate) fn arrow_node() -> Node {
    Node {
        width: px(ARROW_SIZE),
        height: px(ARROW_SIZE),
        border: UiRect {
            right: px(ARROW_STROKE),
            bottom: px(ARROW_STROKE),
            ..default()
        },
        margin: UiRect::bottom(px(ARROW_SIZE / 2.0)),
        ..default()
    }
}

pub(crate) fn arrow_transform() -> UiTransform {
    UiTransform {
        rotation: Rot2::degrees(45.0),
        ..default()
    }
}
