mod builder;
mod components;
mod lifecycle;
mod variant;

pub use self::{
    builder::{ToastBuilder, ToastHostBuilder, toast, toast_host},
    components::{Toast, ToastHost, ToastStack},
    variant::ToastVariant,
};

use self::components::{ToastLabel, toast_node};
use super::{style_targets::pick_targets, theme_lookup::ThemeLookup};
use crate::style::{theme::Theme, toast::ToastMetrics};
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
            (sync_host, sync_toast_style).before(UiSystems::Prepare),
        );
    }
}

/// Writes the stacking direction and the gap of each host.
fn sync_host(mut lookup: ThemeLookup, mut hosts: Query<(Entity, Ref<ToastHost>, &mut Node)>) {
    let resync = lookup.needs_resync();

    for (entity, host, mut node) in &mut hosts {
        if !resync && !host.is_changed() && !node.is_changed() {
            continue;
        }
        let direction = host.stack.direction();
        if node.flex_direction != direction {
            node.flex_direction = direction;
        }
        let gap = lookup.for_entity(entity).toast.row_gap;
        if node.row_gap != gap {
            node.row_gap = gap;
        }
    }
}

/// Resolves each toast's look from its theme and writes it to the toast and its text.
fn sync_toast_style(
    mut lookup: ThemeLookup,
    targets: ToastTargets,
    mut toasts: Query<ToastRow, With<Toast>>,
    mut labels: Query<(&mut TextColor, &mut TextFont), With<ToastLabel>>,
) {
    let targets = targets.collect(lookup.needs_resync());

    for entity in targets {
        let Ok(mut toast) = toasts.get_mut(entity) else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let style = *toast.variant.resolve(theme);
        let metrics = theme.toast;

        toast.bg.set_if_neq(BackgroundColor(style.bg));
        toast.border.set_if_neq(BorderColor::all(style.border));
        write_layout(&mut toast.node, &toast_node(&metrics, &style.shape));
        for child in toast.children.into_iter().flat_map(|c| c.iter()) {
            if let Ok((mut color, mut font)) = labels.get_mut(child) {
                color.set_if_neq(TextColor(style.fg));
                write_font(&mut font, theme, &metrics);
            }
        }
    }
}

/// Writes the size, the border, and the corners, only where they changed.
fn write_layout(node: &mut Mut<Node>, want: &Node) {
    if node.min_width != want.min_width {
        node.min_width = want.min_width;
    }
    if node.max_width != want.max_width {
        node.max_width = want.max_width;
    }
    if node.padding != want.padding {
        node.padding = want.padding;
    }
    if node.border != want.border {
        node.border = want.border;
    }
    if node.border_radius != want.border_radius {
        node.border_radius = want.border_radius;
    }
}

fn write_font(font: &mut Mut<TextFont>, theme: &Theme, metrics: &ToastMetrics) {
    if font.font != theme.font {
        font.font = theme.font.clone();
    }
    if font.font_smoothing != theme.font_smoothing {
        font.font_smoothing = theme.font_smoothing;
    }
    if font.font_size != metrics.font_size {
        font.font_size = metrics.font_size;
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
