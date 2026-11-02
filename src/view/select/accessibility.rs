use super::{
    components::{Select, SelectTrigger},
    open::SelectOpen,
};
use crate::view::style_targets::drain_removed;
use bevy::{a11y::AccessibilityNode, prelude::*};

pub(super) struct SelectAccessibilityPlugin;
impl Plugin for SelectAccessibilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (forward_label, sync_expanded));
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

type TriggerNodes<'w, 's> = Query<'w, 's, &'static mut AccessibilityNode, With<SelectTrigger>>;

/// Tells a screen reader whether the list is open.
fn sync_expanded(
    opened: Query<&Children, (With<Select>, Added<SelectOpen>)>,
    selects: Query<&Children, With<Select>>,
    mut closed: RemovedComponents<SelectOpen>,
    mut nodes: TriggerNodes,
) {
    // Closing comes first: a list that was closed and opened again in the same frame ends up open.
    for entity in drain_removed(&mut closed) {
        if let Ok(children) = selects.get(entity) {
            set_expanded(&mut nodes, children, false);
        }
    }
    for children in &opened {
        set_expanded(&mut nodes, children, true);
    }
}

fn set_expanded(nodes: &mut TriggerNodes, children: &Children, expanded: bool) {
    for child in children.iter() {
        if let Ok(mut node) = nodes.get_mut(child) {
            node.set_expanded(expanded);
            return;
        }
    }
}
