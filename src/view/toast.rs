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
use super::theme_lookup::ThemeLookup;
use crate::style::{theme::Theme, toast::ToastMetrics};
use bevy::{ecs::query::QueryData, prelude::*, ui::UiSystems};

pub(crate) struct ToastPlugin;
impl Plugin for ToastPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, advance_toasts).add_systems(
            PostUpdate,
            (sync_host, sync_toast_style).before(UiSystems::Prepare),
        );
    }
}

/// Moves each toast along its life, and despawns it when it is over.
fn advance_toasts(
    time: Res<Time<Real>>,
    lookup: ThemeLookup,
    mut toasts: Query<(Entity, &mut Toast)>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();
    for (entity, mut toast) in &mut toasts {
        toast.elapsed += delta;
        let metrics = &lookup.for_entity(entity).toast;
        if toast.timing(metrics).frame(toast.elapsed).finished {
            commands.entity(entity).try_despawn();
        }
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

/// Resolves each toast's look from its theme and its place in time, and writes it to the toast and its text.
fn sync_toast_style(
    lookup: ThemeLookup,
    hosts: Query<&ToastHost>,
    mut toasts: Query<ToastRow>,
    mut labels: Query<(&mut TextColor, &mut TextFont), With<ToastLabel>>,
) {
    for mut row in &mut toasts {
        let theme = lookup.for_entity(row.entity);
        let style = *row.variant.resolve(theme);
        let metrics = theme.toast;
        let frame = row.toast.timing(&metrics).frame(row.toast.elapsed);
        let faded = |color: Color| color.with_alpha(color.alpha() * frame.opacity);
        let start = row
            .parent
            .and_then(|parent| hosts.get(parent.parent()).ok())
            .map_or(Val2::ZERO, |host| host.slide);

        row.bg.set_if_neq(BackgroundColor(faded(style.bg)));
        row.border.set_if_neq(BorderColor::all(faded(style.border)));
        write_layout(&mut row.node, &toast_node(&metrics, &style.shape));
        write_slide(&mut row.transform, start, frame.slide);
        for child in row.children.into_iter().flat_map(|c| c.iter()) {
            if let Ok((mut color, mut font)) = labels.get_mut(child) {
                color.set_if_neq(TextColor(faded(style.fg)));
                write_font(&mut font, theme, &metrics);
            }
        }
    }
}

fn write_slide(transform: &mut Mut<UiTransform>, start: Val2, slide: f32) {
    let want = Val2::new(start.x * slide, start.y * slide);
    if transform.translation != want {
        transform.translation = want;
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
    entity: Entity,
    toast: &'static Toast,
    variant: &'static ToastVariant,
    parent: Option<&'static ChildOf>,
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    node: &'static mut Node,
    transform: &'static mut UiTransform,
    children: Option<&'static Children>,
}
