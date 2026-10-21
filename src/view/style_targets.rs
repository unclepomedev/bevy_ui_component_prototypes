use bevy::prelude::*;

/// The widgets whose look has to be rewritten the current frame.
pub(crate) fn pick_targets(
    resync: bool,
    all: impl IntoIterator<Item = Entity>,
    changed: impl IntoIterator<Item = Entity>,
    removed: impl IntoIterator<Item = Entity>,
    focus_moved: impl IntoIterator<Item = Entity>,
) -> Vec<Entity> {
    if resync {
        all.into_iter().collect()
    } else {
        changed
            .into_iter()
            .chain(removed)
            .chain(focus_moved)
            .collect()
    }
}

/// Reads every pending removal. It has to run every frame, also when the removals are not used:
/// an unread removal is delivered again the next frame.
pub(crate) fn drain_removed<C: Component>(removed: &mut RemovedComponents<C>) -> Vec<Entity> {
    removed.read().collect()
}
