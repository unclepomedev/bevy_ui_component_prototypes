use super::components::SelectPopup;
use crate::common::theme_lookup::ThemeLookup;
use bevy::{prelude::*, ui::UiSystems};

pub(super) struct SelectLayerPlugin;
impl Plugin for SelectLayerPlugin {
    fn build(&self, app: &mut App) {
        // Before the stack is built, so that the popup is at its layer in the same frame.
        app.add_systems(PostUpdate, sync_popup_layer.before(UiSystems::Prepare));
    }
}

/// One above the nearest ancestor that has a `GlobalZIndex`; `fallback` when none has.
pub(crate) fn popup_layer(ancestor: Option<i32>, fallback: i32) -> i32 {
    ancestor.map_or(fallback, |z| z.saturating_add(1))
}

/// Writes the `GlobalZIndex` of each popup.
fn sync_popup_layer(
    lookup: ThemeLookup,
    mut popups: Query<(Entity, &mut GlobalZIndex), With<SelectPopup>>,
    layers: Query<&GlobalZIndex, Without<SelectPopup>>,
    parents: Query<&ChildOf>,
) {
    for (entity, mut z) in &mut popups {
        let ancestor = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| layers.get(ancestor).ok())
            .map(|layer| layer.0);
        let fallback = lookup.for_entity(entity).select_metrics.popup_z_index;
        let want = popup_layer(ancestor, fallback);
        if z.0 != want {
            z.0 = want;
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_popup_is_one_above_its_ancestor_or_at_the_fallback() {
        assert_eq!(popup_layer(Some(30), 50), 31);
        assert_eq!(popup_layer(Some(-5), 50), -4);
        assert_eq!(popup_layer(None, 50), 50);
        assert_eq!(popup_layer(Some(i32::MAX), 50), i32::MAX);
    }
}
