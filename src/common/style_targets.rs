use bevy::prelude::*;

/// The widgets whose look has to be rewritten the current frame.
/// `removed`, read with [`drain_removed`].
pub(crate) fn pick_targets(
    resync: bool,
    all: impl IntoIterator<Item = Entity>,
    changed: impl IntoIterator<Item = Entity>,
    removed: impl IntoIterator<Item = Entity>,
    focus_moved: impl IntoIterator<Item = Entity>,
) -> Vec<Entity> {
    if resync {
        all.into_iter().collect()
    } else {
        changed
            .into_iter()
            .chain(removed)
            .chain(focus_moved)
            .collect()
    }
}

/// Reads every pending removal. It has to run every frame, also when the removals are not used:
/// an unread removal is delivered again the next frame.
pub(crate) fn drain_removed<C: Component>(removed: &mut RemovedComponents<C>) -> Vec<Entity> {
    removed.read().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entities(n: usize) -> Vec<Entity> {
        (0..n)
            .map(|i| Entity::from_raw_u32(i as u32).unwrap())
            .collect()
    }

    #[test]
    fn a_resync_picks_every_widget_and_nothing_else() {
        let [a, b, c, d] = entities(4)[..] else {
            unreachable!()
        };
        assert_eq!(pick_targets(true, [a, b], [c], [d], [c]), vec![a, b]);
    }

    #[test]
    fn without_a_resync_it_picks_the_changed_the_removed_and_the_focus_moved() {
        let [a, b, c, d] = entities(4)[..] else {
            unreachable!()
        };
        assert_eq!(
            pick_targets(false, [a, b, c, d], [a], [b], [c]),
            vec![a, b, c]
        );
    }

    #[test]
    fn nothing_changed_picks_nothing() {
        let all = entities(3);
        assert!(pick_targets(false, all, [], [], []).is_empty());
    }

    #[derive(Component)]
    struct Marker;

    #[derive(Resource, Default)]
    struct Seen(Vec<Vec<Entity>>);

    fn record(mut removed: RemovedComponents<Marker>, mut seen: ResMut<Seen>) {
        seen.0.push(drain_removed(&mut removed));
    }

    // An iterator that is not consumed leaves the removal to be delivered again; draining must not.
    #[test]
    fn a_removal_is_delivered_once() {
        let mut app = App::new();
        app.init_resource::<Seen>().add_systems(Update, record);
        let entity = app.world_mut().spawn(Marker).id();
        app.update();

        app.world_mut().entity_mut(entity).remove::<Marker>();
        app.update();
        app.update();

        let seen = &app.world().resource::<Seen>().0;
        assert_eq!(seen[1], vec![entity]);
        assert!(seen[2].is_empty());
    }
}
