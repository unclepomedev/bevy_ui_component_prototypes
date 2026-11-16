use super::{
    components::{
        ActiveWindow, Maximized, UiWindow, WindowGrip, WindowPart, WindowRect, WindowTitle,
        close_node, glyph_bar_node, glyph_box_node, grip_node, maximize_back_node,
        maximize_front_node, title_bar_node, title_button_node, window_node,
    },
    style::{TitleAlign, TitleButtonLook, WindowMetrics, WindowStyle},
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
    let picked = targets.collect(lookup.needs_resync());

    for entity in picked {
        let Ok(mut window) = windows.get_mut(entity) else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let (style, metrics) = (&theme.window, &theme.window_metrics);
        let active = window.active;
        let maximized = window.maximized;
        let own = own_parts(entity, &children, &targets.all);
        let (state, focused) = button_state(&own, WindowPart::Close, &interactions);
        let close = style.close.look(state, focused);
        let (state, focused) = button_state(&own, WindowPart::Maximize, &interactions);
        let maximize = style.maximize.look(state, focused);

        sync_frame(&mut window, style);
        for part in own {
            if let Ok(mut part) = parts.get_mut(part) {
                match *part.kind {
                    WindowPart::TitleBar => sync_title_bar(&mut part, style, metrics, active),
                    WindowPart::Close => {
                        sync_title_button(&mut part, close_node(metrics, style), close);
                    }
                    WindowPart::GlyphBox => sync_glyph_box(&mut part, metrics),
                    WindowPart::GlyphBar => sync_glyph_bar(&mut part, metrics, close),
                    WindowPart::Grip => sync_grip(&mut part, style, metrics, maximized),
                    WindowPart::Maximize => {
                        let node = title_button_node(metrics, style.maximize.radius(metrics));
                        sync_title_button(&mut part, node, maximize);
                    }
                    WindowPart::MaximizeBox => sync_glyph_box(&mut part, metrics),
                    WindowPart::MaximizeFront | WindowPart::MaximizeBack => {
                        sync_maximize_glyph(&mut part, metrics, maximize, maximized);
                    }
                }
            } else if let Ok(mut title) = titles.get_mut(part) {
                sync_title(&mut title, theme, active);
            }
        }
    }
}

/// The descendants of `window` that belong to it: a window inside its content has parts of its own.
fn own_parts(
    window: Entity,
    children: &Query<&Children>,
    frames: &Query<Entity, With<UiWindow>>,
) -> Vec<Entity> {
    let mut own = Vec::new();
    let mut pending = vec![window];
    while let Some(parent) = pending.pop() {
        let Ok(kids) = children.get(parent) else {
            continue;
        };
        for kid in kids.iter().filter(|kid| !frames.contains(*kid)) {
            own.push(kid);
            pending.push(kid);
        }
    }
    own
}

/// The pointer state of the `button` among the `parts` of a window.
fn button_state(
    parts: &[Entity],
    button: WindowPart,
    interactions: &Query<(&WindowPart, &PickingInteraction, Has<ShowsFocus>)>,
) -> (VisualState, bool) {
    parts
        .iter()
        .find_map(|part| {
            let (kind, interaction, focused) = interactions.get(*part).ok()?;
            (*kind == button).then_some((interaction, focused))
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

/// A button of the title bar: its background, its size and its focus ring.
fn sync_title_button(part: &mut PartRowItem, node: Node, look: TitleButtonLook) {
    part.bg.set_if_neq(BackgroundColor(look.bg));
    part.node.set_if_neq(node);
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

fn sync_glyph_bar(part: &mut PartRowItem, metrics: &WindowMetrics, look: TitleButtonLook) {
    part.bg.set_if_neq(BackgroundColor(look.glyph));
    part.node.set_if_neq(glyph_bar_node(metrics));
}

/// A maximized window has no edge to drag: its grips are hidden.
fn sync_grip(
    part: &mut PartRowItem,
    style: &WindowStyle,
    metrics: &WindowMetrics,
    maximized: bool,
) {
    if let Some(WindowGrip { edge }) = part.grip {
        let mut node = grip_node(*edge, metrics, &style.shape);
        if maximized {
            node.display = Display::None;
        }
        part.node.set_if_neq(node);
    }
}

/// A box of the maximize glyph, in the shape for the window's state.
fn sync_maximize_glyph(
    part: &mut PartRowItem,
    metrics: &WindowMetrics,
    look: TitleButtonLook,
    maximized: bool,
) {
    let node = match *part.kind {
        WindowPart::MaximizeBack => maximize_back_node(metrics, maximized),
        _ => maximize_front_node(metrics, maximized),
    };
    part.node.set_if_neq(node);
    if let Some(border) = part.border.as_mut() {
        border.set_if_neq(BorderColor::all(look.glyph));
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
    maximized: Has<Maximized>,
}

/// What the sync writes to a part of a window.
#[derive(QueryData)]
#[query_data(mutable)]
struct PartRow {
    kind: &'static WindowPart,
    grip: Option<&'static WindowGrip>,
    bg: &'static mut BackgroundColor,
    border: Option<&'static mut BorderColor>,
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
    maximized: Query<'w, 's, Entity, (With<UiWindow>, Added<Maximized>)>,
    restored: RemovedComponents<'w, 's, Maximized>,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl WindowTargets<'_, '_> {
    /// A window is also restyled when a button was touched or got or lost the focus ring, and when it got or lost
    /// [`ActiveWindow`] or [`Maximized`].
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
        let restored = drain_removed(&mut self.restored);
        pick_targets(
            resync,
            self.all.iter(),
            self.added
                .iter()
                .chain(touched)
                .chain(self.activated.iter())
                .chain(self.maximized.iter()),
            deactivated.into_iter().chain(restored),
            [],
        )
    }
}
