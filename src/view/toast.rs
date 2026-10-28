mod builder;
mod components;
mod lifecycle;
mod variant;

pub use self::{
    builder::{ToastBuilder, ToastHostBuilder, toast, toast_host},
    components::{Toast, ToastHost, ToastStack},
    variant::ToastVariant,
};

use bevy::{prelude::*, ui::UiSystems};

pub(crate) struct ToastPlugin;
impl Plugin for ToastPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_host_direction.before(UiSystems::Prepare));
    }
}

/// Writes the stacking direction of each host from its `stack`. The `Node` of the host is the app's,
/// and an app that inserts one later does not know about this, so a changed `Node` is checked too.
fn sync_host_direction(mut hosts: ChangedHosts) {
    for (host, mut node) in &mut hosts {
        let want = host.stack.direction();
        if node.flex_direction != want {
            node.flex_direction = want;
        }
    }
}

type ChangedHosts<'w, 's> =
    Query<'w, 's, (&'static ToastHost, &'static mut Node), Or<(Changed<ToastHost>, Changed<Node>)>>;
