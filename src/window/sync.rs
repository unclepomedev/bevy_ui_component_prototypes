use super::{
    components::{
        ActiveWindow, UiWindow, WindowGrip, WindowPart, WindowRect, WindowTitle, close_node,
        glyph_bar_node, glyph_box_node, grip_node, title_bar_node, window_node,
    },
    style::{CloseLook, TitleAlign, WindowMetrics, WindowStyle},
};
use crate::{
    Theme,
    common::{
        focus::ShowsFocus,
        state::VisualState,
        style_targets::{drain_removed, pick_targets},
        theme_lookup::ThemeLookup,
    },
};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::UiSystems,
};

pub(crate) struct WindowStylePlugin;
impl Plugin for WindowStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (sync_window_rect, sync_window_style).before(UiSystems::Prepare),
        );
    }
}

/// Writes the position and the size of each window whose `WindowRect` changed.
fn sync_window_rect(mut windows: Query<(&WindowRect, &mut Node), Changed<WindowRect>>) {
    for (rect, mut node) in &mut windows {
        let (left, top) = (Val::Px(rect.x), Val::Px(rect.y));
        let (width, height) = (Val::Px(rect.width), Val::Px(rect.height));
        if node.left != left {
            node.left = left;
        }
        if node.top != top {
            node.top = top;
        }
        if node.width != width {
            node.width = width;
        }
        if node.height != height {
            node.height = height;
        }
    }
}

/// Resolves each window's look from its theme, its state and the state of its close button, and writes it to
/// the window and its parts.
fn sync_window_style(
    mut lookup: ThemeLookup,
    mut targets: WindowTargets,
    mut windows: Query<WindowRow, With<UiWindow>>,
    mut parts: Query<PartRow, Without<UiWindow>>,
    mut titles: Query<TitleRow, With<WindowTitle>>,
    interactions: Query<(&WindowPart, &PickingInteraction, Has<ShowsFocus>)>,
    children: Query<&Children>,
) {
    let targets = targets.collect(lookup.needs_resync());

    for entity in targets {
        let Ok(mut window) = windows.get_mut(entity) else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let (style, metrics) = (&theme.window, &theme.window_metrics);
        let active = window.active;
        let (state, focused) = close_state(entity, &interactions, &children);
        let close = style.close.look(state, focused);

        sync_frame(&mut window, style);
        for part in children.iter_descendants(entity) {
            if let Ok(mut part) = parts.get_mut(part) {
                match *part.kind {
                    WindowPart::TitleBar => sync_title_bar(&mut part, style, metrics, active),
                    WindowPart::Close => sync_close(&mut part, style, metrics, close),
                    WindowPart::GlyphBox => sync_glyph_box(&mut part, metrics),
                    WindowPart::GlyphBar => sync_glyph_bar(&mut part, metrics, close),
                    WindowPart::Grip => sync_grip(&mut part, style, metrics),
                }
            } else if let Ok(mut title) = titles.get_mut(part) {
                sync_title(&mut title, theme, active);
            }
        }
    }
}

/// The state of the close button of `window`, from the pointer.
fn close_state(
    window: Entity,
    interactions: &Query<(&WindowPart, &PickingInteraction, Has<ShowsFocus>)>,
    children: &Query<&Children>,
) -> (VisualState, bool) {
    children
        .iter_descendants(window)
        .find_map(|part| {
            let (kind, interaction, focused) = interactions.get(part).ok()?;
            (*kind == WindowPart::Close).then_some((interaction, focused))
        })
        .map_or((VisualState::Normal, false), |(interaction, focused)| {
            (VisualState::resolve(interaction, false, false), focused)
        })
}

/// The window itself: the colors of the frame and its border and corners.
fn sync_frame(window: &mut WindowRowItem, style: &WindowStyle) {
    window.bg.set_if_neq(BackgroundColor(style.bg));
    window.border.set_if_neq(BorderColor::all(style.border));
    let frame = window_node(&style.shape);
    if window.node.border != frame.border {
        window.node.border = frame.border;
    }
    if window.node.border_radius != frame.border_radius {
        window.node.border_radius = frame.border_radius;
    }
}

fn sync_title_bar(
    part: &mut PartRowItem,
    style: &WindowStyle,
    metrics: &WindowMetrics,
    active: bool,
) {
    let (bg, _) = style.title_bar.colors(active);
    part.bg.set_if_neq(BackgroundColor(bg));
    part.node
        .set_if_neq(title_bar_node(metrics, &style.shape, style.close.side));
}

fn sync_close(
    part: &mut PartRowItem,
    style: &WindowStyle,
    metrics: &WindowMetrics,
    look: CloseLook,
) {
    part.bg.set_if_neq(BackgroundColor(look.bg));
    part.node.set_if_neq(close_node(metrics, style));
    if let Some(outline) = part.outline.as_mut() {
        outline.set_if_neq(Outline::new(
            look.ring.width,
            look.ring.offset,
            look.ring.color,
        ));
    }
}

fn sync_glyph_box(part: &mut PartRowItem, metrics: &WindowMetrics) {
    part.node.set_if_neq(glyph_box_node(metrics));
}

fn sync_glyph_bar(part: &mut PartRowItem, metrics: &WindowMetrics, look: CloseLook) {
    part.bg.set_if_neq(BackgroundColor(look.glyph));
    part.node.set_if_neq(glyph_bar_node(metrics));
}

fn sync_grip(part: &mut PartRowItem, style: &WindowStyle, metrics: &WindowMetrics) {
    if let Some(WindowGrip { edge }) = part.grip {
        part.node
            .set_if_neq(grip_node(*edge, metrics, &style.shape));
    }
}

/// The title text: its color, font, size and alignment.
fn sync_title(title: &mut TitleRowItem, theme: &Theme, active: bool) {
    let (style, metrics) = (&theme.window, &theme.window_metrics);
    let (_, fg) = style.title_bar.colors(active);
    title.color.set_if_neq(TextColor(fg));
    if title.font.font != theme.font {
        title.font.font = theme.font.clone();
    }
    if title.font.font_smoothing != theme.font_smoothing {
        title.font.font_smoothing = theme.font_smoothing;
    }
    if title.font.font_size != metrics.title_font_size {
        title.font.font_size = metrics.title_font_size;
    }
    let justify = match style.title_bar.align {
        TitleAlign::Start => Justify::Left,
        TitleAlign::Center => Justify::Center,
    };
    if title.layout.justify != justify {
        title.layout.justify = justify;
    }
}

/// What the sync writes to a window itself.
#[derive(QueryData)]
#[query_data(mutable)]
struct WindowRow {
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    node: &'static mut Node,
    active: Has<ActiveWindow>,
}

/// What the sync writes to a part of a window.
#[derive(QueryData)]
#[query_data(mutable)]
struct PartRow {
    kind: &'static WindowPart,
    grip: Option<&'static WindowGrip>,
    bg: &'static mut BackgroundColor,
    node: &'static mut Node,
    outline: Option<&'static mut Outline>,
}

/// What the sync writes to the title.
#[derive(QueryData)]
#[query_data(mutable)]
struct TitleRow {
    color: &'static mut TextColor,
    font: &'static mut TextFont,
    layout: &'static mut TextLayout,
}

#[derive(SystemParam)]
struct WindowTargets<'w, 's> {
    added: Query<'w, 's, Entity, Added<UiWindow>>,
    all: Query<'w, 's, Entity, With<UiWindow>>,
    touched: Query<'w, 's, Entity, (With<WindowPart>, Changed<PickingInteraction>)>,
    focused: Query<'w, 's, Entity, (With<WindowPart>, Added<ShowsFocus>)>,
    unfocused: RemovedComponents<'w, 's, ShowsFocus>,
    activated: Query<'w, 's, Entity, (With<UiWindow>, Added<ActiveWindow>)>,
    deactivated: RemovedComponents<'w, 's, ActiveWindow>,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl WindowTargets<'_, '_> {
    /// A window is also restyled when its close button was touched or gained or lost the focus ring,
    /// and when it gained or lost [`ActiveWindow`].
    fn collect(&mut self, resync: bool) -> Vec<Entity> {
        let unfocused = drain_removed(&mut self.unfocused);
        let touched: Vec<Entity> = self
            .touched
            .iter()
            .chain(self.focused.iter())
            .chain(unfocused)
            .filter_map(|part| {
                self.parents
                    .iter_ancestors(part)
                    .find(|ancestor| self.all.contains(*ancestor))
            })
            .collect();
        let deactivated = drain_removed(&mut self.deactivated);
        pick_targets(
            resync,
            self.all.iter(),
            self.added
                .iter()
                .chain(touched)
                .chain(self.activated.iter()),
            deactivated,
            [],
        )
    }
}
