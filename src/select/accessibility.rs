use super::{
    components::{
        Select, SelectContent, SelectLabels, SelectList, SelectPopup, SelectRow, SelectTrigger,
        SelectedIndex,
    },
    open::SelectOpen,
    trigger::refresh_select_label,
};
use crate::{button::ButtonLabel, common::style_targets::drain_removed};
use accesskit::NodeId;
use bevy::{a11y::AccessibilityNode, prelude::*};

pub(super) struct SelectAccessibilityPlugin;
impl Plugin for SelectAccessibilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                forward_label,
                link_trigger_to_list,
                sync_open_state,
                sync_names.after(refresh_select_label),
            ),
        );
    }
}

type NamedSelects<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static AccessibleLabel, &'static Children),
    (With<Select>, Changed<AccessibleLabel>),
>;

/// Puts the name the app gave to the select on the trigger, which is what takes the focus and what a
/// screen reader reads. The select itself does not keep a node, which would carry the name a second time.
fn forward_label(
    named: NamedSelects,
    selects: Query<(&Children, Has<AccessibleLabel>), With<Select>>,
    triggers: Query<(), With<SelectTrigger>>,
    mut removed: RemovedComponents<AccessibleLabel>,
    mut commands: Commands,
) {
    let trigger_of = |children: &Children| children.iter().find(|child| triggers.contains(*child));

    for entity in drain_removed(&mut removed) {
        // A select that got another label in the same frame keeps one.
        if let Ok((children, false)) = selects.get(entity)
            && let Some(trigger) = trigger_of(children)
        {
            commands.entity(trigger).try_remove::<AccessibleLabel>();
        }
    }
    for (select, label, children) in &named {
        if let Some(trigger) = trigger_of(children) {
            commands.entity(trigger).try_insert(label.clone());
            commands.entity(select).try_remove::<AccessibilityNode>();
        }
    }
}

// Two queries on the same component: the filters keep them apart.
type TriggerNodes<'w, 's> =
    Query<'w, 's, &'static mut AccessibilityNode, (With<SelectTrigger>, Without<SelectPopup>)>;
type PopupNodes<'w, 's> =
    Query<'w, 's, &'static mut AccessibilityNode, (With<SelectPopup>, Without<SelectTrigger>)>;

type NewSelects<'w, 's> = Query<'w, 's, (Entity, &'static Children), (With<Select>, Added<Select>)>;

/// Tells a screen reader which list the trigger opens.
fn link_trigger_to_list(
    added: NewSelects,
    descendants: Query<&Children>,
    lists: Query<(), With<SelectList>>,
    mut nodes: TriggerNodes,
) {
    for (select, children) in &added {
        let Some(list) = descendants
            .iter_descendants(select)
            .find(|entity| lists.contains(*entity))
        else {
            continue;
        };
        for child in children.iter() {
            if let Ok(mut node) = nodes.get_mut(child) {
                node.set_controls(vec![NodeId(list.to_bits())]);
                break;
            }
        }
    }
}

/// Tells a screen reader whether the list is open: the trigger says so, and the list is out of its tree
/// while it is closed.
fn sync_open_state(
    opened: Query<&Children, (With<Select>, Added<SelectOpen>)>,
    selects: Query<&Children, With<Select>>,
    mut closed: RemovedComponents<SelectOpen>,
    mut triggers: TriggerNodes,
    mut popups: PopupNodes,
) {
    // Closing comes first: a list that was closed and opened again in the same frame ends up open.
    for entity in drain_removed(&mut closed) {
        if let Ok(children) = selects.get(entity) {
            set_open(&mut triggers, &mut popups, children, false);
        }
    }
    for children in &opened {
        set_open(&mut triggers, &mut popups, children, true);
    }
}

fn set_open(triggers: &mut TriggerNodes, popups: &mut PopupNodes, children: &Children, open: bool) {
    for child in children.iter() {
        if let Ok(mut node) = triggers.get_mut(child) {
            node.set_expanded(open);
        } else if let Ok(mut node) = popups.get_mut(child) {
            if open {
                node.clear_hidden();
            } else {
                node.set_hidden();
            }
        }
    }
}

/// What a screen reader reads of the options is the text of their labels: the name of each row, and the value
/// of the trigger (its placeholder while nothing is chosen). A label without a `Text` is not read.
fn sync_names(
    selects: Query<(Entity, &Children, &SelectedIndex, &SelectLabels), With<Select>>,
    descendants: Query<&Children>,
    contents: Query<(), With<SelectContent>>,
    rows: Query<Option<&AccessibleLabel>, With<SelectRow>>,
    texts: Query<&Text, With<ButtonLabel>>,
    mut triggers: TriggerNodes,
    mut commands: Commands,
) {
    let text_under = |parent: Entity| {
        let children = descendants.get(parent).ok()?;
        children
            .iter()
            .find_map(|child| texts.get(child).ok())
            .map(|text| text.0.as_str())
    };

    for (select, children, chosen, labels) in &selects {
        let shown = descendants
            .iter_descendants(select)
            .find(|entity| contents.contains(*entity))
            .and_then(text_under);
        let is_chosen = chosen
            .index
            .is_some_and(|index| index < labels.options.len());
        for child in children.iter() {
            if let Ok(mut node) = triggers.get_mut(child) {
                write_trigger_text(&mut node, shown, is_chosen);
                break;
            }
        }

        for row in descendants.iter_descendants(select) {
            let Ok(current) = rows.get(row) else {
                continue;
            };
            let Some(name) = text_under(row) else {
                continue;
            };
            if current.is_none_or(|label| label.0 != name) {
                commands.entity(row).try_insert(AccessibleLabel::new(name));
            }
        }
    }
}

/// The text of the shown label is the value of the trigger when an option is chosen, and its placeholder
/// when not. Writes only what differs.
fn write_trigger_text(node: &mut Mut<AccessibilityNode>, shown: Option<&str>, chosen: bool) {
    let (value, placeholder) = if chosen { (shown, None) } else { (None, shown) };
    match value {
        Some(value) if node.value() != Some(value) => node.set_value(value),
        None if node.value().is_some() => node.clear_value(),
        _ => {}
    }
    match placeholder {
        Some(text) if node.placeholder() != Some(text) => node.set_placeholder(text),
        None if node.placeholder().is_some() => node.clear_placeholder(),
        _ => {}
    }
}
