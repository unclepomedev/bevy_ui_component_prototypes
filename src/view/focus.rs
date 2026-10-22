use super::KeyboardNavigable;
use bevy::{
    ecs::system::SystemParam,
    input_focus::{InputFocus, InputFocusVisible},
    prelude::*,
    ui::{InteractionDisabled, UiSystems},
};

/// Present on the widget of this crate that holds the visible focus indicator.
///
/// - The state behind the focus ring, for a look the ring cannot give.
/// - Set while the entity has the input focus, the indicator is visible, and the widget is not disabled.
/// - The crate adds and removes it; do not insert it yourself.
/// - Updated in `PostUpdate`, together with the ring.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ShowsFocus;

pub(crate) struct ShowsFocusPlugin;
impl Plugin for ShowsFocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, mark_visible_focus.before(UiSystems::Prepare));
    }
}

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

fn mark_visible_focus(
    focus: Option<Res<InputFocus>>,
    visible: Option<Res<InputFocusVisible>>,
    widgets: Query<Has<InteractionDisabled>, With<KeyboardNavigable>>,
    mut last: Local<Option<Entity>>,
    mut commands: Commands,
) {
    let marked = focus
        .as_deref()
        .and_then(InputFocus::get)
        .filter(|_| visible.as_deref().is_some_and(|visible| visible.0))
        .filter(|entity| widgets.get(*entity) == Ok(false));
    if marked == *last {
        return;
    }
    if let Some(previous) = *last {
        commands.entity(previous).try_remove::<ShowsFocus>();
    }
    if let Some(entity) = marked {
        commands.entity(entity).try_insert(ShowsFocus);
    }
    *last = marked;
}
