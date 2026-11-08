use super::components::Tooltip;
use crate::common::focus::FocusIndicator;
use bevy::{
    ecs::system::SystemParam,
    input_focus::{InputFocus, InputFocusVisible},
    math::FloatOrd,
    picking::{hover::HoverMap, pointer::PointerId},
    prelude::*,
};
use std::{cmp::Reverse, iter::once as iter_once};

/// What the pointer and the focus ask for, as of this frame.
#[derive(Resource, Default)]
pub(super) struct Wanted {
    /// The widget a tooltip is asked for.
    pub(super) widget: Option<Entity>,
    /// This frame asks the tooltip on screen to go away (a press, Escape).
    pub(super) dismiss: bool,
}

/// The nearest entity up from `entity` (itself included) that has a [`Tooltip`].
fn owner(entity: Entity, tooltips: &Query<&Tooltip>, parents: &Query<&ChildOf>) -> Option<Entity> {
    iter_once(entity)
        .chain(parents.iter_ancestors(entity))
        .find(|entity| tooltips.contains(*entity))
}

/// Finds the widget that wants a tooltip.
/// The pointer wins over the focus: while the pointer rests on a widget with a tooltip, the focus is not asked.
pub(super) fn ask_for_tooltip(
    pointer: MousePointer,
    focus: Option<Res<InputFocus>>,
    focus_visible: Option<Res<InputFocusVisible>>,
    indicator: Res<FocusIndicator>,
    tooltips: Query<&Tooltip>,
    parents: Query<&ChildOf>,
    mut wanted: ResMut<Wanted>,
) {
    let hovered = pointer.front(&tooltips, &parents);
    let focused = focus
        .as_deref()
        .and_then(InputFocus::get)
        .filter(|_| indicator.is_visible(focus_visible.as_deref()))
        .and_then(|entity| owner(entity, &tooltips, &parents));

    wanted.widget = hovered.or(focused);
}

/// What the mouse rests on, and the cameras that say which of those hits is in front.
#[derive(SystemParam)]
pub(super) struct MousePointer<'w, 's> {
    hover: Option<Res<'w, HoverMap>>,
    cameras: Query<'w, 's, &'static Camera>,
}

impl MousePointer<'_, '_> {
    /// The widget the mouse rests on, from the hit in front of all the others.
    fn front(&self, tooltips: &Query<&Tooltip>, parents: &Query<&ChildOf>) -> Option<Entity> {
        self.hover
            .as_deref()
            .and_then(|map| map.get(&PointerId::Mouse))
            .and_then(|hits| {
                hits.iter()
                    .filter_map(|(entity, hit)| {
                        let layer = self
                            .cameras
                            .get(hit.camera)
                            .map_or(0, |camera| camera.order);
                        owner(*entity, tooltips, parents)
                            .map(|owner| ((Reverse(layer), FloatOrd(hit.depth)), owner))
                    })
                    .min_by_key(|(order, _)| *order)
            })
            .map(|(_, owner)| owner)
    }
}

/// Tells whether this frame asks the tooltip on screen to go away: a pointer press, or Escape.
pub(super) fn watch_dismiss(
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut wanted: ResMut<Wanted>,
) {
    wanted.dismiss = mouse
        .as_deref()
        .is_some_and(|mouse| mouse.get_just_pressed().next().is_some())
        || keys
            .as_deref()
            .is_some_and(|keys| keys.just_pressed(KeyCode::Escape));
}
