use bevy::{
    ecs::system::SystemParam,
    input_focus::{InputFocus, InputFocusVisible},
    prelude::*,
};

/// Follows which widget should show its focus ring.
#[derive(SystemParam)]
pub(crate) struct FocusRingTracker<'w, 's> {
    // Optional so that apps without `InputFocusPlugin` simply have no focused widget.
    focus: Option<Res<'w, InputFocus>>,
    visible: Option<Res<'w, InputFocusVisible>>,
    last: Local<'s, Option<Entity>>,
}

impl FocusRingTracker<'_, '_> {
    /// Returns the entity whose focus indicator is visible right now, and the entities
    /// whose ring state has changed since the previous call (the old and the new owner).
    pub(crate) fn update(&mut self) -> (Option<Entity>, Vec<Entity>) {
        let focused = self
            .focus
            .as_deref()
            .and_then(InputFocus::get)
            .filter(|_| self.visible.as_deref().is_some_and(|visible| visible.0));
        let previous = std::mem::replace(&mut *self.last, focused);
        let moved = if previous == focused {
            Vec::new()
        } else {
            previous.into_iter().chain(focused).collect()
        };
        (focused, moved)
    }
}
