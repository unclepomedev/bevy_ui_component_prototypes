use accesskit::{Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;

/// What a screen reader finds for a popup: a modal dialog.
pub(super) fn popup_node_role() -> AccessibilityNode {
    let mut node = AccessNode::new(Role::Dialog);
    node.set_modal();
    AccessibilityNode(node)
}
