use super::components::{
    Select, SelectList, SelectPopup, SelectRow, SelectedIndex, list_node, popup_node,
    popup_placement, row_node,
};
use crate::{
    style::state::VisualState,
    view::{
        button::{ButtonContent, ButtonIcon},
        style_targets::{drain_removed, pick_targets},
        theme_lookup::ThemeLookup,
    },
};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{InteractionDisabled, Selected, UiSystems},
    ui_widgets::{ActiveDescendant, popover::Popover},
};

pub(super) struct SelectPopupStylePlugin;
impl Plugin for SelectPopupStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                sync_select_popup,
                // `Selected` is put on the rows first, so that they are painted with it in the same frame.
                (sync_selected_row, sync_select_rows).chain(),
            )
                .before(UiSystems::Prepare),
        );
    }
}

type ChangedSelects<'w, 's> =
    Query<'w, 's, (Entity, &'static SelectedIndex), (With<Select>, Changed<SelectedIndex>)>;

/// Marks the row of the chosen option as `Selected`, and no other row.
fn sync_selected_row(
    selects: ChangedSelects,
    children: Query<&Children>,
    rows: Query<(&SelectRow, Has<Selected>)>,
    mut commands: Commands,
) {
    for (select, chosen) in &selects {
        for entity in children.iter_descendants(select) {
            let Ok((row, selected)) = rows.get(entity) else {
                continue;
            };
            let wanted = chosen.index == Some(row.index);
            if wanted && !selected {
                commands.entity(entity).insert(Selected);
            } else if !wanted && selected {
                commands.entity(entity).remove::<Selected>();
            }
        }
    }
}

/// Paints the frame of each popup: its background, border, corners and inner padding, and sizes its list.
fn sync_select_popup(
    mut lookup: ThemeLookup,
    added: Query<Entity, Added<SelectPopup>>,
    all: Query<Entity, With<SelectPopup>>,
    mut popups: Query<PopupRow, With<SelectPopup>>,
    children: Query<&Children>,
    mut lists: Query<&mut Node, (With<SelectList>, Without<SelectPopup>)>,
) {
    let resync = lookup.needs_resync();
    let targets = pick_targets(resync, all.iter(), added.iter(), [], []);

    for entity in targets {
        let Ok(mut popup) = popups.get_mut(entity) else {
            continue;
        };
        let theme = lookup.for_entity(entity);
        let style = &theme.select.popup;

        popup.bg.set_if_neq(BackgroundColor(style.bg));
        popup.border.set_if_neq(BorderColor::all(style.border));
        write_popup_layout(
            &mut popup.node,
            &popup_node(&theme.select_metrics, &style.shape),
        );
        let placement = popup_placement(&theme.select_metrics);
        if *popup.popover != placement {
            *popup.popover = placement;
        }
        let z_index = theme.select_metrics.popup_z_index;
        if popup.z_index.0 != z_index {
            popup.z_index.0 = z_index;
        }
        for child in children.get(entity).into_iter().flat_map(|c| c.iter()) {
            if let Ok(mut list) = lists.get_mut(child) {
                let want = list_node(&theme.select_metrics);
                if list.max_height != want.max_height {
                    list.max_height = want.max_height;
                }
            }
        }
    }
}

/// Writes the border, the corners and the padding, only where they changed.
fn write_popup_layout(node: &mut Mut<Node>, want: &Node) {
    if node.border != want.border {
        node.border = want.border;
    }
    if node.border_radius != want.border_radius {
        node.border_radius = want.border_radius;
    }
    if node.padding != want.padding {
        node.padding = want.padding;
    }
}

/// What the sync writes to a popup.
#[derive(QueryData)]
#[query_data(mutable)]
struct PopupRow {
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    node: &'static mut Node,
    popover: &'static mut Popover,
    z_index: &'static mut GlobalZIndex,
}

/// Paints each row of the list: its background, its text and its size.
fn sync_select_rows(
    mut lookup: ThemeLookup,
    mut targets: RowTargets,
    lists: Query<&ActiveDescendant, With<SelectList>>,
    // Without the icon, which `ButtonContent` writes a `Node` of its own to.
    mut rows: Query<OptionRow, (With<SelectRow>, Without<ButtonIcon>)>,
    mut content: ButtonContent,
) {
    let resync = lookup.needs_resync();
    let targets = targets.collect(resync);

    for entity in targets {
        let Ok(mut row) = rows.get_mut(entity) else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let metrics = &theme.select_metrics;
        let highlighted = lists
            .get(row.parent.parent())
            .is_ok_and(|active| active.0 == Some(entity));
        // The row under the pointer is the highlighted one already, so the pointer adds only the press.
        let state = VisualState::from_flags(
            *row.interaction == PickingInteraction::Pressed,
            false,
            highlighted,
            row.disabled,
        );
        let look = theme.select.option_look(state, row.selected);

        row.bg.set_if_neq(BackgroundColor(look.bg));
        write_row_layout(&mut row.node, &row_node(metrics));
        content.apply(entity, look.fg, Some(&metrics.trigger), theme);
    }
}

/// Writes the height and the padding, only where they changed.
fn write_row_layout(node: &mut Mut<Node>, want: &Node) {
    if node.height != want.height {
        node.height = want.height;
    }
    if node.padding != want.padding {
        node.padding = want.padding;
    }
}

/// What the sync reads from a row and writes to it.
#[derive(QueryData)]
#[query_data(mutable)]
struct OptionRow {
    parent: &'static ChildOf,
    interaction: &'static PickingInteraction,
    selected: Has<Selected>,
    disabled: Has<InteractionDisabled>,
    bg: &'static mut BackgroundColor,
    node: &'static mut Node,
}

/// Finds the rows whose look may have changed from the last run.
#[derive(SystemParam)]
struct RowTargets<'w, 's> {
    changed: ChangedRows<'w, 's>,
    all: Query<'w, 's, Entity, With<SelectRow>>,
    moved: Query<'w, 's, &'static Children, (With<SelectList>, Changed<ActiveDescendant>)>,
    deselected: RemovedComponents<'w, 's, Selected>,
    enabled: RemovedComponents<'w, 's, InteractionDisabled>,
}

impl RowTargets<'_, '_> {
    /// Every row after a theme change, otherwise the ones that changed, and all the rows of a list
    /// whose highlight moved (the old row and the new one).
    fn collect(&mut self, resync: bool) -> Vec<Entity> {
        let mut removed = drain_removed(&mut self.deselected);
        removed.extend(drain_removed(&mut self.enabled));
        removed.extend(self.moved.iter().flat_map(|children| children.iter()));
        pick_targets(resync, self.all.iter(), self.changed.iter(), removed, [])
    }
}

type ChangedRows<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<SelectRow>,
        Or<(
            Added<SelectRow>,
            Changed<PickingInteraction>,
            Added<Selected>,
            Added<InteractionDisabled>,
        )>,
    ),
>;
