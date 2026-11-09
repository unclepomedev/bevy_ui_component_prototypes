use super::components::SelectPopup;
use crate::common::{layer::layer_above, theme_lookup::ThemeLookup};
use bevy::{prelude::*, ui::UiSystems};

pub(super) struct SelectLayerPlugin;
impl Plugin for SelectLayerPlugin {
    fn build(&self, app: &mut App) {
        // Before the stack is built, so that the popup is at its layer in the same frame.
        app.add_systems(PostUpdate, sync_popup_layer.before(UiSystems::Prepare));
    }
}

/// Writes the `GlobalZIndex` of each popup: `layer_gap` above the nearest ancestor that has one.
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
        let gap = lookup.for_entity(entity).select_metrics.layer_gap;
        let want = layer_above(ancestor, gap);
        if z.0 != want {
            z.0 = want;
        }
    }
}
