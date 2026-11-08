use super::components::PopupRoot;
use crate::common::theme_lookup::ThemeLookup;
use bevy::prelude::*;

/// Marks a root whose `GlobalZIndex` comes from the theme, so that it follows the theme and a layer
/// set by the spawner is left alone.
#[derive(Component)]
pub(super) struct ThemedLayer;

type Unlayered = (With<PopupRoot>, Without<GlobalZIndex>, Without<ThemedLayer>);

/// Gives a root without a layer the one of its theme, and keeps it there.
pub(super) fn sync_popup_layer(
    lookup: ThemeLookup,
    unlayered: Query<Entity, Unlayered>,
    mut themed: Query<(Entity, &mut GlobalZIndex), With<ThemedLayer>>,
    mut commands: Commands,
) {
    for entity in &unlayered {
        let z = lookup.for_entity(entity).popup_metrics.z_index;
        commands
            .entity(entity)
            .insert((GlobalZIndex(z), ThemedLayer));
    }
    for (entity, mut layer) in &mut themed {
        let z = lookup.for_entity(entity).popup_metrics.z_index;
        if layer.0 != z {
            layer.0 = z;
        }
    }
}
