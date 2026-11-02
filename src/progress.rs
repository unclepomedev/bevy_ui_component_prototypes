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
use bevy::{a11y::AccessibilityNode, ecs::query::QueryData, prelude::*, ui::UiSystems};

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

        write_accessibility(&mut bar);
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
    a11y: &'static mut AccessibilityNode,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct FillRow {
    bg: &'static mut BackgroundColor,
    gradient: &'static mut BackgroundGradient,
    node: &'static mut Node,
}

/// Gives screen readers the value and the ends. Writes only what differs: a write marks the node
/// as changed, and a theme change visits every bar.
fn write_accessibility(bar: &mut BarRowItem) {
    let Some((min, max, value)) = accessible_numbers(bar.range, **bar.value) else {
        return;
    };
    if bar.a11y.min_numeric_value() != Some(min) {
        bar.a11y.set_min_numeric_value(min);
    }
    if bar.a11y.max_numeric_value() != Some(max) {
        bar.a11y.set_max_numeric_value(max);
    }
    if bar.a11y.numeric_value() != Some(value) {
        bar.a11y.set_numeric_value(value);
    }
}

/// `(min, max, value)` for a screen reader. A reversed range and a stray value stay readable.
/// A NaN value reads as `min`. `None` when an end is not a finite number.
fn accessible_numbers(range: &ProgressRange, value: f32) -> Option<(f64, f64, f64)> {
    let (min, max) = if range.start <= range.end {
        (range.start, range.end)
    } else {
        (range.end, range.start)
    };
    if !min.is_finite() || !max.is_finite() {
        return None;
    }
    let value = if value.is_nan() {
        min
    } else {
        value.clamp(min, max)
    };
    Some((min as f64, max as f64, value as f64))
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

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: f32, end: f32) -> ProgressRange {
        ProgressRange { start, end }
    }

    #[test]
    fn the_numbers_are_the_range_and_the_value() {
        assert_eq!(
            accessible_numbers(&range(10.0, 30.0), 15.0),
            Some((10.0, 30.0, 15.0))
        );
    }

    #[test]
    fn a_reversed_range_is_put_in_order() {
        assert_eq!(
            accessible_numbers(&range(30.0, 10.0), 15.0),
            Some((10.0, 30.0, 15.0))
        );
    }

    #[test]
    fn the_value_stays_between_the_ends() {
        assert_eq!(
            accessible_numbers(&range(0.0, 1.0), 7.0),
            Some((0.0, 1.0, 1.0))
        );
        assert_eq!(
            accessible_numbers(&range(0.0, 1.0), -7.0),
            Some((0.0, 1.0, 0.0))
        );
        assert_eq!(
            accessible_numbers(&range(0.0, 1.0), f32::NAN),
            Some((0.0, 1.0, 0.0))
        );
    }

    #[test]
    fn an_end_that_is_not_finite_gives_nothing() {
        assert_eq!(accessible_numbers(&range(f32::NAN, 1.0), 0.5), None);
        assert_eq!(accessible_numbers(&range(0.0, f32::INFINITY), 0.5), None);
    }
}
