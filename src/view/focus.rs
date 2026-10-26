use super::KeyboardNavigable;
use bevy::{
    ecs::system::SystemParam,
    input_focus::{InputFocus, InputFocusVisible},
    prelude::*,
    ui::{InteractionDisabled, UiSystems},
};

/// When the focus indicator is drawn on the focused widget.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusIndicator {
    /// Follows Bevy's [`InputFocusVisible`]: a pointer click hides it, the keyboard shows it again.
    #[default]
    Auto,
    /// Always drawn on the focused widget, whatever `InputFocusVisible` says.
    Always,
}

impl FocusIndicator {
    fn is_visible(self, visible: Option<&InputFocusVisible>) -> bool {
        self == FocusIndicator::Always || visible.is_some_and(|visible| visible.0)
    }
}

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
        app.init_resource::<FocusIndicator>()
            .add_systems(PostUpdate, mark_visible_focus.before(UiSystems::Prepare));
    }
}

/// Follows which widget should show its focus ring.
#[derive(SystemParam)]
pub(crate) struct FocusRingTracker<'w, 's> {
    // Optional so that apps without `InputFocusPlugin` simply have no focused widget.
    focus: Option<Res<'w, InputFocus>>,
    visible: Option<Res<'w, InputFocusVisible>>,
    indicator: Res<'w, FocusIndicator>,
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
            .filter(|_| self.indicator.is_visible(self.visible.as_deref()));
        let previous = std::mem::replace(&mut *self.last, focused);
        let moved = if previous == focused {
            Vec::new()
        } else {
            previous.into_iter().chain(focused).collect()
        };
        (focused, moved)
    }
}

/// Returns an `Outline` without color or size.
pub(crate) fn hidden_outline() -> Outline {
    Outline::new(Val::Px(0.0), Val::Px(0.0), Color::NONE)
}

fn mark_visible_focus(
    focus: Option<Res<InputFocus>>,
    visible: Option<Res<InputFocusVisible>>,
    indicator: Res<FocusIndicator>,
    widgets: Query<Has<InteractionDisabled>, With<KeyboardNavigable>>,
    mut last: Local<Option<Entity>>,
    mut commands: Commands,
) {
    let marked = focus
        .as_deref()
        .and_then(InputFocus::get)
        .filter(|_| indicator.is_visible(visible.as_deref()))
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
