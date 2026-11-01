use super::{
    components::{Select, SelectList, SelectRequest, SelectRow},
    open::OptionCommitted,
};
use bevy::{
    prelude::*,
    ui_widgets::{ReselectListRow, ValueChange},
};

pub(super) struct SelectChoosePlugin;
impl Plugin for SelectChoosePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_list_change).add_observer(on_reselect);
    }
}

/// The list reports the row that was asked for; the select reports the option behind it.
fn on_list_change(
    change: On<ValueChange<Entity>>,
    lists: Query<(), With<SelectList>>,
    rows: Query<&SelectRow>,
    parents: Query<&ChildOf>,
    selects: Query<(), With<Select>>,
    requests: Query<&SelectRequest>,
    mut commands: Commands,
) {
    if !lists.contains(change.source) {
        return;
    }
    if !parents
        .get(change.value)
        .is_ok_and(|parent| parent.parent() == change.source)
    {
        return;
    }
    let Ok(row) = rows.get(change.value) else {
        return;
    };
    let Some(select) = select_above(change.source, &parents, &selects) else {
        return;
    };
    let Ok(request) = requests.get(select) else {
        return;
    };
    (request.emit)(&mut commands, select, row.index);
    commands.trigger(OptionCommitted { select });
}

/// The chosen option was chosen again: nothing changes, and the list is done.
fn on_reselect(
    reselect: On<ReselectListRow>,
    lists: Query<(), With<SelectList>>,
    parents: Query<&ChildOf>,
    selects: Query<(), With<Select>>,
    mut commands: Commands,
) {
    if !lists.contains(reselect.source) {
        return;
    }
    if !parents
        .get(reselect.row)
        .is_ok_and(|parent| parent.parent() == reselect.source)
    {
        return;
    }
    if let Some(select) = select_above(reselect.source, &parents, &selects) {
        commands.trigger(OptionCommitted { select });
    }
}

fn select_above(
    list: Entity,
    parents: &Query<&ChildOf>,
    selects: &Query<(), With<Select>>,
) -> Option<Entity> {
    parents
        .iter_ancestors(list)
        .find(|ancestor| selects.contains(*ancestor))
}
