mod builder;
mod components;
mod defaults;
mod requests;
mod stack;
mod style;

pub use self::{
    builder::{WindowBuilder, window},
    components::{ActiveWindow, UiWindow, WindowContent, WindowRect},
    defaults::WindowDefaultsPlugin,
    requests::RaiseWindow,
    stack::WindowStack,
    style::{
        CloseShape, CloseStyle, TitleAlign, TitleBarStyle, WindowMetrics, WindowShape, WindowSide,
        WindowStyle,
    },
};

use self::components::{
    WindowPart, WindowTitle, close_node, glyph_bar_node, glyph_box_node, title_bar_node,
    window_node,
};
use crate::common::{state::VisualState, style_targets::pick_targets, theme_lookup::ThemeLookup};
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

/// Resolves each window's look from its theme and the state of its close button, and writes it to its parts.
fn sync_window_style(
    mut lookup: ThemeLookup,
    targets: WindowTargets,
    mut windows: Query<WindowRow, With<UiWindow>>,
    mut parts: Query<PartRow, Without<UiWindow>>,
    mut titles: Query<TitleRow, With<WindowTitle>>,
    interactions: Query<(&WindowPart, &PickingInteraction)>,
    children: Query<&Children>,
) {
    let targets = targets.collect(lookup.needs_resync());

    for entity in targets {
        let Ok(mut window) = windows.get_mut(entity) else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let (style, metrics) = (&theme.window, &theme.window_metrics);
        let close_state = children
            .iter_descendants(entity)
            .find_map(|part| {
                let (kind, interaction) = interactions.get(part).ok()?;
                (*kind == WindowPart::Close).then_some(interaction)
            })
            .map_or(VisualState::Normal, |interaction| {
                VisualState::resolve(interaction, false, false)
            });
        let close = style.close.look(close_state);

        window.bg.set_if_neq(BackgroundColor(style.bg));
        window.border.set_if_neq(BorderColor::all(style.border));
        let frame = window_node(&style.shape);
        if window.node.border != frame.border {
            window.node.border = frame.border;
        }
        if window.node.border_radius != frame.border_radius {
            window.node.border_radius = frame.border_radius;
        }

        for part in children.iter_descendants(entity) {
            if let Ok(mut part) = parts.get_mut(part) {
                match *part.kind {
                    WindowPart::TitleBar => {
                        part.bg.set_if_neq(BackgroundColor(style.title_bar.bg));
                        part.node
                            .set_if_neq(title_bar_node(metrics, style.close.side));
                    }
                    WindowPart::Close => {
                        part.bg.set_if_neq(BackgroundColor(close.bg));
                        part.node.set_if_neq(close_node(metrics, style));
                    }
                    WindowPart::GlyphBox => {
                        part.node.set_if_neq(glyph_box_node(metrics));
                    }
                    WindowPart::GlyphBar => {
                        part.bg.set_if_neq(BackgroundColor(close.glyph));
                        part.node.set_if_neq(glyph_bar_node(metrics));
                    }
                }
            } else if let Ok(mut title) = titles.get_mut(part) {
                title.color.set_if_neq(TextColor(style.title_bar.fg));
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
        }
    }
}

/// What the sync writes to a window itself.
#[derive(QueryData)]
#[query_data(mutable)]
struct WindowRow {
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    node: &'static mut Node,
}

/// What the sync writes to a part of a window.
#[derive(QueryData)]
#[query_data(mutable)]
struct PartRow {
    kind: &'static WindowPart,
    bg: &'static mut BackgroundColor,
    node: &'static mut Node,
}

/// What the sync writes to the title.
#[derive(QueryData)]
#[query_data(mutable)]
struct TitleRow {
    color: &'static mut TextColor,
    font: &'static mut TextFont,
    layout: &'static mut TextLayout,
}

/// Finds the windows whose look may have changed from the last run.
#[derive(SystemParam)]
struct WindowTargets<'w, 's> {
    added: Query<'w, 's, Entity, Added<UiWindow>>,
    all: Query<'w, 's, Entity, With<UiWindow>>,
    touched: Query<'w, 's, Entity, (With<WindowPart>, Changed<PickingInteraction>)>,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl WindowTargets<'_, '_> {
    /// Every window after a theme change, otherwise the new ones and those whose close button was touched.
    fn collect(&self, resync: bool) -> Vec<Entity> {
        let touched: Vec<Entity> = self
            .touched
            .iter()
            .filter_map(|part| {
                self.parents
                    .iter_ancestors(part)
                    .find(|ancestor| self.all.contains(*ancestor))
            })
            .collect();
        pick_targets(
            resync,
            self.all.iter(),
            self.added.iter().chain(touched),
            [],
            [],
        )
    }
}
