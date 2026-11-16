//! The clock of what plays over time. See [`PlaybackClock`].

mod clock;

pub use self::clock::PlaybackClock;

use bevy::prelude::*;

/// A [`PlaybackClock`] that was playing has reached its end and stopped there.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct PlaybackEnded {
    #[event_target]
    pub clock: Entity,
}

pub(crate) struct PlaybackPlugin;
impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, advance_clocks);
    }
}

/// Moves the clocks that are playing by the real time of the frame. A paused clock is not touched, so
/// `Changed<PlaybackClock>` finds the ones that moved.
fn advance_clocks(
    time: Res<Time<Real>>,
    mut clocks: Query<(Entity, &mut PlaybackClock)>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();
    for (clock_entity, mut clock) in &mut clocks {
        if clock.is_playing() && clock.advance(delta) {
            commands.trigger(PlaybackEnded {
                clock: clock_entity,
            });
        }
    }
}
