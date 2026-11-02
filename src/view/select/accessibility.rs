use super::components::{Select, SelectTrigger};
use crate::view::style_targets::drain_removed;
use bevy::a11y::AccessibilityNode;
use bevy::prelude::*;

pub(super) struct SelectAccessibilityPlugin;
impl Plugin for SelectAccessibilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, forward_label);
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
