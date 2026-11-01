mod builder;
mod components;
mod variant;

pub use self::{
    builder::{ProgressBarBuilder, progress_bar},
    components::{ProgressBar, ProgressRange, ProgressValue},
    variant::ProgressVariant,
};

use self::components::{ProgressFill, fill_node, progress_node};
use super::{style_targets::pick_targets, theme_lookup::ThemeLookup};
use crate::style::progress::{ProgressGeometry, ProgressStyle};
use bevy::{
    ecs::query::QueryData,
    prelude::*,
    ui::{ColorStop, LinearGradient, UiSystems},
};

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
    paint(
        &mut bar.bg,
        &mut bar.gradient,
        geometry.segments.map(|s| s.stops(s.count, style.rail)),
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
        // A segmented fill spans the whole bar and lights blocks; a solid one grows.
        let width = match geometry.segments {
            Some(s) => {
                paint(
                    &mut fill.bg,
                    &mut fill.gradient,
                    Some(s.stops(s.lit(share), style.fill)),
                    style.fill,
                );
                Val::Percent(100.0)
            }
            None => {
                paint(&mut fill.bg, &mut fill.gradient, None, style.fill);
                Val::Percent(share * 100.0)
            }
        };
        if fill.node.width != width {
            fill.node.width = width;
        }
    }
}

fn paint(
    bg: &mut Mut<BackgroundColor>,
    gradient: &mut Mut<BackgroundGradient>,
    stops: Option<Vec<ColorStop>>,
    color: Color,
) {
    match stops {
        Some(stops) => {
            bg.set_if_neq(BackgroundColor(Color::NONE));
            let want = vec![LinearGradient::to_right(stops).into()];
            if gradient.0 != want {
                gradient.0 = want;
            }
        }
        None => {
            bg.set_if_neq(BackgroundColor(color));
            if !gradient.0.is_empty() {
                gradient.0.clear();
            }
        }
    }
}
