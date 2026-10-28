mod builder;
mod components;
mod lifecycle;
mod variant;

pub use self::{
    builder::{ToastBuilder, ToastHostBuilder, toast, toast_host},
    components::{Toast, ToastHost, ToastStack},
    variant::ToastVariant,
};

use self::components::ToastLabel;
use super::{style_targets::pick_targets, theme_lookup::ThemeLookup};
use crate::style::toast::ToastShape;
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    prelude::*,
    ui::UiSystems,
};

pub(crate) struct ToastPlugin;
impl Plugin for ToastPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (sync_host_direction, sync_toast_style).before(UiSystems::Prepare),
        );
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

/// Resolves each toast's look from its theme and writes it to the toast and its text.
fn sync_toast_style(
    mut lookup: ThemeLookup,
    targets: ToastTargets,
    mut toasts: Query<ToastRow, With<Toast>>,
    mut labels: Query<&mut TextColor, With<ToastLabel>>,
) {
    let targets = targets.collect(lookup.needs_resync());

    for entity in targets {
        let Ok(mut toast) = toasts.get_mut(entity) else {
            continue;
        };

        let style = *toast.variant.resolve(lookup.for_entity(entity));

        toast.bg.set_if_neq(BackgroundColor(style.bg));
        toast.border.set_if_neq(BorderColor::all(style.border));
        write_shape(&mut toast.node, &style.shape);
        for child in toast.children.into_iter().flat_map(|c| c.iter()) {
            if let Ok(mut color) = labels.get_mut(child) {
                color.set_if_neq(TextColor(style.fg));
            }
        }
    }
}

/// Writes the border and the corners, only where they changed.
fn write_shape(node: &mut Mut<Node>, shape: &ToastShape) {
    let border = UiRect::all(Val::Px(shape.border_width));
    if node.border != border {
        node.border = border;
    }
    let radius = BorderRadius::all(Val::Px(shape.radius));
    if node.border_radius != radius {
        node.border_radius = radius;
    }
}

/// What the sync reads from a toast and writes to it.
#[derive(QueryData)]
#[query_data(mutable)]
struct ToastRow {
    variant: &'static ToastVariant,
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    node: &'static mut Node,
    children: Option<&'static Children>,
}

/// Finds the toasts whose look may have changed from the last run.
#[derive(SystemParam)]
struct ToastTargets<'w, 's> {
    changed: ChangedToasts<'w, 's>,
    all: Query<'w, 's, Entity, With<Toast>>,
}

impl ToastTargets<'_, '_> {
    /// Every toast after a theme change, otherwise only the new ones and the ones whose variant changed.
    fn collect(&self, resync: bool) -> Vec<Entity> {
        pick_targets(resync, self.all.iter(), self.changed.iter(), [], [])
    }
}

type ChangedToasts<'w, 's> =
    Query<'w, 's, Entity, (With<Toast>, Or<(Added<Toast>, Changed<ToastVariant>)>)>;
