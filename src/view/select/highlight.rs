use super::{
    components::{Select, SelectList, SelectRow},
    open::SelectOpen,
};
use bevy::{
    picking::events::PointerMove, prelude::*, ui::InteractionDisabled, ui_widgets::ActiveDescendant,
};

pub(super) struct SelectHighlightPlugin;
impl Plugin for SelectHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(highlight_row_under_pointer);
    }
}

/// The row the pointer moves over is the highlighted one, so that the pointer and the keys share one highlight.
/// Only a move of the pointer counts: when the list scrolls under a pointer that rests, the highlight stays where the keys put it.
fn highlight_row_under_pointer(
    moved: On<PointerMove>,
    rows: Query<(&ChildOf, Has<InteractionDisabled>), With<SelectRow>>,
    lists: Query<&ActiveDescendant, With<SelectList>>,
    parents: Query<&ChildOf>,
    open: Query<(), (With<Select>, With<SelectOpen>)>,
    mut commands: Commands,
) {
    // The event travels up through the ancestors of the row, and only the row itself counts.
    let row = moved.event_target();
    let Ok((parent, disabled)) = rows.get(row) else {
        return;
    };
    if disabled {
        return;
    }
    let list = parent.parent();
    let Ok(active) = lists.get(list) else {
        return;
    };
    if active.0 == Some(row) {
        return;
    }
    if parents
        .iter_ancestors(list)
        .any(|ancestor| open.contains(ancestor))
    {
        commands.entity(list).insert(ActiveDescendant(Some(row)));
    }
}
