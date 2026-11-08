mod components;
mod style;
mod timing;

pub use self::{
    components::Tooltip,
    style::{TooltipMetrics, TooltipShape, TooltipStyle},
};

use self::{
    components::{
        TooltipLabel, TooltipPopup, placement, popup_accessibility, popup_node, spawn_popup,
    },
    timing::{Timing, Tracker},
};
use crate::common::{focus::FocusIndicator, theme_lookup::ThemeLookup};
use bevy::{
    a11y::AccessibilityNode,
    ecs::query::QueryData,
    input_focus::{InputFocus, InputFocusVisible},
    picking::{hover::HoverMap, pointer::PointerId},
    prelude::*,
    ui::UiSystems,
};
use std::iter::once as iter_once;

pub(crate) struct TooltipPlugin;
impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TooltipState>()
            .add_systems(
                Update,
                (
                    forget_lost_popup,
                    ask_for_tooltip,
                    watch_dismiss,
                    drive_tooltips,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_popup_boxes,
                    sync_popup_layers,
                    sync_popup_labels,
                    describe_targets,
                )
                    .before(UiSystems::Prepare),
            )
            .add_observer(clear_description);
    }
}

/// The tooltip on screen, and the clock that decides when the next one shows.
#[derive(Resource, Default)]
struct TooltipState {
    tracker: Tracker,
    popup: Option<Entity>,
    /// The widget the pointer or the focus asks a tooltip for, as of this frame.
    wanted: Option<Entity>,
    /// This frame asks the tooltip on screen to go away (a press, Escape).
    dismiss: bool,
}

/// The nearest entity up from `entity` (itself included) that has a [`Tooltip`].
fn owner(entity: Entity, tooltips: &Query<&Tooltip>, parents: &Query<&ChildOf>) -> Option<Entity> {
    iter_once(entity)
        .chain(parents.iter_ancestors(entity))
        .find(|entity| tooltips.contains(*entity))
}

/// Forgets the tooltip on screen when it is gone without the tracker asking (its widget was despawned).
fn forget_lost_popup(popups: Query<(), With<TooltipPopup>>, mut state: ResMut<TooltipState>) {
    if state.popup.is_some_and(|popup| !popups.contains(popup)) {
        state.popup = None;
        state.tracker.lost();
    }
}

/// Finds the widget that wants a tooltip, and whether this frame asks the tooltip on screen to go away.
/// The pointer wins over the focus: while the pointer rests on a widget with a tooltip, the focus is not asked.
fn ask_for_tooltip(
    hover: Option<Res<HoverMap>>,
    focus: Option<Res<InputFocus>>,
    focus_visible: Option<Res<InputFocusVisible>>,
    indicator: Res<FocusIndicator>,
    tooltips: Query<&Tooltip>,
    parents: Query<&ChildOf>,
    mut state: ResMut<TooltipState>,
) {
    // The nearest hit counts: a smaller depth is closer to the camera.
    let hovered = hover
        .as_deref()
        .and_then(|map| map.get(&PointerId::Mouse))
        .and_then(|hits| {
            hits.iter()
                .filter_map(|(entity, hit)| {
                    owner(*entity, &tooltips, &parents).map(|owner| (hit.depth, owner))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
        })
        .map(|(_, owner)| owner);
    let focused = focus
        .as_deref()
        .and_then(InputFocus::get)
        .filter(|_| indicator.is_visible(focus_visible.as_deref()))
        .and_then(|entity| owner(entity, &tooltips, &parents));

    state.wanted = hovered.or(focused);
}

/// Tells whether this frame asks the tooltip on screen to go away: a pointer press, or Escape.
fn watch_dismiss(
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<TooltipState>,
) {
    state.dismiss = mouse
        .as_deref()
        .is_some_and(|mouse| mouse.get_just_pressed().next().is_some())
        || keys
            .as_deref()
            .is_some_and(|keys| keys.just_pressed(KeyCode::Escape));
}

/// Shows and hides the tooltip as the tracker says, from what [`ask_for_tooltip`] and [`watch_dismiss`] found.
fn drive_tooltips(
    time: Res<Time<Real>>,
    lookup: ThemeLookup,
    tooltips: Query<&Tooltip>,
    mut state: ResMut<TooltipState>,
    mut commands: Commands,
) {
    let (wanted, dismiss) = (state.wanted, state.dismiss);
    let step = state
        .tracker
        .step(wanted, dismiss, time.delta_secs(), |entity| {
            let metrics = &lookup.for_entity(entity).tooltip_metrics;
            Timing {
                delay: tooltips
                    .get(entity)
                    .ok()
                    .and_then(|tooltip| tooltip.delay)
                    .unwrap_or(metrics.delay),
                warmup: metrics.warmup,
            }
        });

    if step.hide
        && let Some(popup) = state.popup.take()
    {
        commands.entity(popup).try_despawn();
    }
    if let Some(target) = step.show {
        state.popup = Some(spawn_popup(&mut commands, target));
    }
}

/// Gives the box of each tooltip on screen its colors, its shape and its place.
fn sync_popup_boxes(lookup: ThemeLookup, tooltips: Query<&Tooltip>, mut popups: Query<PopupBox>) {
    for mut row in &mut popups {
        let Ok(tooltip) = tooltips.get(row.popup.target) else {
            continue;
        };
        let theme = lookup.for_entity(row.popup.target);
        let style = theme.tooltip;
        let metrics = theme.tooltip_metrics;

        row.popover.set_if_neq(placement(tooltip.side, &metrics));
        row.bg.set_if_neq(BackgroundColor(style.bg));
        row.border.set_if_neq(BorderColor::all(style.border));
        write_layout(&mut row.node, &popup_node(&metrics, &style.shape));

        if row.accessibility.0.label() != Some(tooltip.text.as_str()) {
            *row.accessibility = popup_accessibility(&tooltip.text);
        }
    }
}

/// Puts each tooltip on screen in the layer of its theme, or above the widget it is for.
fn sync_popup_layers(
    lookup: ThemeLookup,
    layers: Query<&GlobalZIndex, Without<TooltipPopup>>,
    parents: Query<&ChildOf>,
    mut popups: Query<(Entity, &TooltipPopup, &mut GlobalZIndex)>,
) {
    for (entity, popup, mut z) in &mut popups {
        let metrics = lookup.for_entity(popup.target).tooltip_metrics;
        let above = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| layers.get(ancestor).ok())
            .map_or(i32::MIN, |layer| layer.0.saturating_add(1));
        z.set_if_neq(GlobalZIndex(above.max(metrics.z_index)));
    }
}

/// Gives the text inside each tooltip on screen its words and its font.
fn sync_popup_labels(
    lookup: ThemeLookup,
    tooltips: Query<&Tooltip>,
    popups: Query<&TooltipPopup>,
    mut labels: Query<(&ChildOf, &mut Text, &mut TextColor, &mut TextFont), With<TooltipLabel>>,
) {
    for (child_of, mut text, mut color, mut font) in &mut labels {
        let Ok(popup) = popups.get(child_of.parent()) else {
            continue;
        };
        let Ok(tooltip) = tooltips.get(popup.target) else {
            continue;
        };
        let theme = lookup.for_entity(popup.target);

        if text.0 != tooltip.text {
            text.0.clone_from(&tooltip.text);
        }
        color.set_if_neq(TextColor(theme.tooltip.fg));
        if font.font != theme.font {
            font.font = theme.font.clone();
        }
        if font.font_smoothing != theme.font_smoothing {
            font.font_smoothing = theme.font_smoothing;
        }
        if font.font_size != theme.tooltip_metrics.font_size {
            font.font_size = theme.tooltip_metrics.font_size;
        }
    }
}

/// Writes the size, the border, and the corners, only where they changed.
fn write_layout(node: &mut Mut<Node>, want: &Node) {
    if node.max_width != want.max_width {
        node.max_width = want.max_width;
    }
    if node.padding != want.padding {
        node.padding = want.padding;
    }
    if node.border != want.border {
        node.border = want.border;
    }
    if node.border_radius != want.border_radius {
        node.border_radius = want.border_radius;
    }
}

/// Makes the text of a tooltip the description of its widget, where the widget has an accessibility node.
fn describe_targets(mut targets: TooltipTargetChanged) {
    for (tooltip, mut node) in &mut targets {
        if node.0.description() != Some(tooltip.text.as_str()) {
            node.0.set_description(tooltip.text.as_str());
        }
    }
}

/// Takes the description back when the [`Tooltip`] goes, so that a screen reader stops reading it.
fn clear_description(
    removed: On<Remove<Tooltip>>,
    tooltips: Query<&Tooltip>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    let (Ok(tooltip), Ok(mut node)) = (tooltips.get(removed.entity), nodes.get_mut(removed.entity))
    else {
        return;
    };
    if node.0.description() == Some(tooltip.text.as_str()) {
        node.0.clear_description();
    }
}

/// What the sync reads from the box of a tooltip and writes to it.
#[derive(QueryData)]
#[query_data(mutable)]
struct PopupBox {
    popup: &'static TooltipPopup,
    popover: &'static mut bevy::ui_widgets::popover::Popover,
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    node: &'static mut Node,
    accessibility: &'static mut AccessibilityNode,
}

type TooltipTargetChanged<'w, 's> = Query<
    'w,
    's,
    (&'static Tooltip, &'static mut AccessibilityNode),
    Or<(Changed<Tooltip>, Changed<AccessibilityNode>)>,
>;
