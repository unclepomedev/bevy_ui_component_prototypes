//! How the rail and the fill of a bar are painted.

use crate::common::bar::BarSegments;
use bevy::{
    prelude::*,
    ui::{ColorStop, LinearGradient},
};

/// Paints a rail: every block in `color`, or one solid color without `segments`.
pub(crate) fn paint_rail(
    bg: &mut Mut<BackgroundColor>,
    gradient: Option<&mut Mut<BackgroundGradient>>,
    segments: Option<BarSegments>,
    color: Color,
) {
    let stops = segments.map(|segments| segments.stops(segments.count, color));
    paint(bg, gradient, stops, color);
}

/// Paints a fill and returns the width it takes.
///
/// A segmented fill spans the whole bar and lights blocks; a solid one grows with `share` (0 to 1).
pub(crate) fn paint_fill(
    bg: &mut Mut<BackgroundColor>,
    gradient: Option<&mut Mut<BackgroundGradient>>,
    segments: Option<BarSegments>,
    share: f32,
    color: Color,
) -> Val {
    match segments {
        Some(segments) => {
            paint(
                bg,
                gradient,
                Some(segments.stops(segments.lit(share), color)),
                color,
            );
            Val::Percent(100.0)
        }
        None => {
            paint(bg, gradient, None, color);
            Val::Percent(share * 100.0)
        }
    }
}

/// With `stops`, the blocks are the gradient and the plain background stays out of the way.
/// Without, the plain background is the color and the gradient is empty.
fn paint(
    bg: &mut Mut<BackgroundColor>,
    gradient: Option<&mut Mut<BackgroundGradient>>,
    stops: Option<Vec<ColorStop>>,
    color: Color,
) {
    let (background, want) = match stops {
        Some(stops) => (Color::NONE, vec![LinearGradient::to_right(stops).into()]),
        None => (color, Vec::new()),
    };
    bg.set_if_neq(BackgroundColor(background));
    if let Some(gradient) = gradient
        && gradient.0 != want
    {
        gradient.0 = want;
    }
}
