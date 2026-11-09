mod builder;
mod components;
mod style;
mod variant;

pub use self::{
    builder::{ProgressBarBuilder, progress_bar},
    components::{ProgressBar, ProgressRange, ProgressValue},
    variant::ProgressVariant,
};

pub use self::style::{ProgressShape, ProgressStyle, ProgressStyles, ProgressVariantId};

use self::components::{ProgressFill, fill_node, progress_node};
use self::style::ProgressGeometry;
use crate::common::{
    bar_paint::{paint_fill, paint_rail},
    style_targets::pick_targets,
    theme_lookup::ThemeLookup,
};
use bevy::{ecs::query::QueryData, prelude::*, ui::UiSystems};

pub(crate) struct ProgressStylePlugin;
impl Plugin for ProgressStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_progress_style.before(UiSystems::Prepare));
    }
}

type ChangedBars<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<ProgressBar>,
        Or<(
            Added<ProgressBar>,
            Changed<ProgressValue>,
            Changed<ProgressRange>,
            Changed<ProgressVariant>,
        )>,
    ),
>;

fn sync_progress_style(
    mut lookup: ThemeLookup,
    changed: ChangedBars,
    all: Query<Entity, With<ProgressBar>>,
    mut bars: Query<BarRow, With<ProgressBar>>,
    mut fills: Query<FillRow, (With<ProgressFill>, Without<ProgressBar>)>,
    children: Query<&Children>,
) {
    let resync = lookup.needs_resync();
    let targets = pick_targets(resync, all.iter(), changed.iter(), [], []);

    for entity in targets {
        let Ok(mut bar) = bars.get_mut(entity) else {
            continue;
        };
        let style = *bar.variant.resolve(lookup.for_entity(entity));
        let geometry = style.shape.geometry();
        let share = bar.range.fraction(**bar.value);

        write_bar(&mut bar, &style, &geometry);
        write_fill(&mut fills, &children, entity, &style, &geometry, share);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct BarRow {
    variant: &'static ProgressVariant,
    value: &'static ProgressValue,
    range: &'static ProgressRange,
    bg: &'static mut BackgroundColor,
    gradient: &'static mut BackgroundGradient,
    node: &'static mut Node,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct FillRow {
    bg: &'static mut BackgroundColor,
    gradient: &'static mut BackgroundGradient,
    node: &'static mut Node,
}

fn write_bar(bar: &mut BarRowItem, style: &ProgressStyle, geometry: &ProgressGeometry) {
    let want = progress_node(geometry);
    if bar.node.height != want.height {
        bar.node.height = want.height;
    }
    if bar.node.border_radius != want.border_radius {
        bar.node.border_radius = want.border_radius;
    }
    paint_rail(
        &mut bar.bg,
        Some(&mut bar.gradient),
        geometry.segments,
        style.rail,
    );
}

fn write_fill(
    fills: &mut Query<FillRow, (With<ProgressFill>, Without<ProgressBar>)>,
    children: &Query<&Children>,
    bar: Entity,
    style: &ProgressStyle,
    geometry: &ProgressGeometry,
    share: f32,
) {
    for child in children.get(bar).into_iter().flat_map(|c| c.iter()) {
        let Ok(mut fill) = fills.get_mut(child) else {
            continue;
        };
        let want = fill_node(geometry);
        if fill.node.border_radius != want.border_radius {
            fill.node.border_radius = want.border_radius;
        }
        let width = paint_fill(
            &mut fill.bg,
            Some(&mut fill.gradient),
            geometry.segments,
            share,
            style.fill,
        );
        if fill.node.width != width {
            fill.node.width = width;
        }
    }
}
