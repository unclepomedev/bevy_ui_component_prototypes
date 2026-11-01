//! Reading the blocks of a bar back from its gradient.

use bevy::{
    prelude::*,
    ui::{ColorStop, Gradient},
};

/// The stops of the part's only gradient; empty when it has none.
pub fn stops(app: &App, part: Entity) -> Vec<ColorStop> {
    let gradient = app.world().get::<BackgroundGradient>(part).unwrap();
    match gradient.0.first() {
        Some(Gradient::Linear(linear)) => linear.stops.clone(),
        Some(other) => panic!("not a linear gradient: {other:?}"),
        None => Vec::new(),
    }
}

/// How many blocks `stops` paints with `color` (each block is a pair of stops of that color).
pub fn painted_blocks(stops: &[ColorStop], color: Color) -> usize {
    stops.iter().filter(|stop| stop.color == color).count() / 2
}
