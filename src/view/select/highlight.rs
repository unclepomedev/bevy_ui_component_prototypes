use super::{
    components::{Select, SelectList, SelectRow},
    open::SelectOpen,
};
use bevy::{
    picking::hover::PickingInteraction, prelude::*, ui::InteractionDisabled,
    ui_widgets::ActiveDescendant,
};

pub(super) struct SelectHighlightPlugin;
impl Plugin for SelectHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, highlight_row_under_pointer);
    }
}

type RowsUnderPointer<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ChildOf,
        &'static PickingInteraction,
        Has<InteractionDisabled>,
    ),
    (With<SelectRow>, Changed<PickingInteraction>),
>;

/// The row the pointer comes to is the highlighted one, so that the pointer and the keys share one highlight.
fn highlight_row_under_pointer(
    rows: RowsUnderPointer,
    lists: Query<&ActiveDescendant, With<SelectList>>,
    parents: Query<&ChildOf>,
    open: Query<(), (With<Select>, With<SelectOpen>)>,
    mut commands: Commands,
) {
    for (row, parent, interaction, disabled) in &rows {
        if *interaction == PickingInteraction::None || disabled {
            continue;
        }
        let list = parent.parent();
        let Ok(active) = lists.get(list) else {
            continue;
        };
        if active.0 == Some(row) {
            continue;
        }
        if parents
            .iter_ancestors(list)
            .any(|ancestor| open.contains(ancestor))
        {
            commands.entity(list).insert(ActiveDescendant(Some(row)));
        }
    }
}
