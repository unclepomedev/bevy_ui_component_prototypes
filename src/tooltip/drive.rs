use super::{
    components::{Tooltip, TooltipPopup, spawn_popup},
    timing::{Timing, Tracker},
    wanted::Wanted,
};
use crate::common::theme_lookup::ThemeLookup;
use bevy::prelude::*;

/// The tooltip on screen, and the clock that decides when the next one shows.
#[derive(Resource, Default)]
pub(super) struct TooltipState {
    tracker: Tracker,
    popup: Option<Entity>,
}

/// Forgets the tooltip on screen when it is gone without the tracker asking (its widget was despawned).
pub(super) fn forget_lost_popup(
    popups: Query<(), With<TooltipPopup>>,
    mut state: ResMut<TooltipState>,
) {
    if state.popup.is_some_and(|popup| !popups.contains(popup)) {
        state.popup = None;
        state.tracker.lost();
    }
}

/// Shows and hides the tooltip as the tracker says, for what [`Wanted`] holds.
pub(super) fn drive_tooltips(
    time: Res<Time<Real>>,
    wanted: Res<Wanted>,
    lookup: ThemeLookup,
    tooltips: Query<&Tooltip>,
    mut state: ResMut<TooltipState>,
    mut commands: Commands,
) {
    let step = state.tracker.step(
        wanted.widget,
        wanted.dismiss,
        time.elapsed_secs_f64(),
        |entity| {
            let metrics = &lookup.for_entity(entity).tooltip_metrics;
            Timing {
                delay: tooltips
                    .get(entity)
                    .ok()
                    .and_then(|tooltip| tooltip.delay)
                    .unwrap_or(metrics.delay),
                warmup: metrics.warmup,
            }
        },
    );

    if step.hide
        && let Some(popup) = state.popup.take()
    {
        commands.entity(popup).try_despawn();
    }
    if let Some(target) = step.show {
        state.popup = Some(spawn_popup(&mut commands, target));
    }
}
