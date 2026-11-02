use super::{
    components::{Select, SelectList, SelectRow, SelectedIndex},
    open::SelectOpen,
};
use bevy::{
    prelude::*,
    ui::{ScrollPosition, UiSystems},
    ui_widgets::ScrollIntoView,
};

pub(super) struct SelectScrollPlugin;
impl Plugin for SelectScrollPlugin {
    fn build(&self, app: &mut App) {
        // Before the layout, so that the list is drawn at its scroll position from the first frame it is open.
        app.add_systems(
            PostUpdate,
            scroll_to_chosen_on_open.before(UiSystems::Prepare),
        );
    }
}

type OpenedSelects<'w, 's> =
    Query<'w, 's, (Entity, &'static SelectedIndex), (With<Select>, Added<SelectOpen>)>;

/// The list opens at the chosen option, or at its top when nothing is chosen.
fn scroll_to_chosen_on_open(
    opened: OpenedSelects,
    descendants: Query<&Children>,
    lists: Query<(), With<SelectList>>,
    rows: Query<&SelectRow>,
    mut commands: Commands,
) {
    for (select, chosen) in &opened {
        let Some(list) = descendants
            .iter_descendants(select)
            .find(|entity| lists.contains(*entity))
        else {
            continue;
        };
        let row = chosen.index.and_then(|index| {
            descendants.get(list).ok()?.iter().find(|row| {
                rows.get(*row)
                    .is_ok_and(|candidate| candidate.index == index)
            })
        });
        match row {
            Some(row) => commands.trigger(ScrollIntoView { entity: row }),
            None => {
                commands.entity(list).insert(ScrollPosition::default());
            }
        }
    }
}
