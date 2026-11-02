use super::{
    components::{Select, SelectList, SelectPopup, SelectTrigger},
    open::SelectOpen,
};
use crate::view::style_targets::drain_removed;
use accesskit::NodeId;
use bevy::{a11y::AccessibilityNode, prelude::*};

pub(super) struct SelectAccessibilityPlugin;
impl Plugin for SelectAccessibilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (forward_label, link_trigger_to_list, sync_open_state),
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
