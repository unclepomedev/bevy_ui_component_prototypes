use super::components::{UiWindow, WindowTitle};
use accesskit::{Node as AccessNode, Role};
use bevy::{a11y::AccessibilityNode, prelude::*};

/// What a screen reader finds for a window: a dialog.
pub(super) fn window_node_role() -> AccessibilityNode {
    AccessibilityNode(AccessNode::new(Role::Dialog))
}

/// The tab index of the close button: after everything in the window.
pub(super) const CLOSE_TAB_INDEX: i32 = i32::MAX;

type ChangedTitles<'w, 's> =
    Query<'w, 's, (Entity, &'static Text), (With<WindowTitle>, Changed<Text>)>;

/// Gives the window the text of its title as its name.
pub(super) fn name_window(
    titles: ChangedTitles,
    parents: Query<&ChildOf>,
    windows: Query<(), With<UiWindow>>,
    mut commands: Commands,
) {
    for (title, text) in &titles {
        let Some(window) = parents
            .iter_ancestors(title)
            .find(|entity| windows.contains(*entity))
        else {
            continue;
        };
        commands
            .entity(window)
            .try_insert(AccessibleLabel::new(text.0.clone()));
    }
}
