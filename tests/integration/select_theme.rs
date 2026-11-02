//! Behavior tests for the select: structure, the label that is shown, the look of the trigger,
//! the popup and its rows, opening and closing, and theme resolution.
//! The pointer and the keys are covered by `examples/select_check.rs`.

use crate::support::test_app;
use bevy::{
    a11y::AccessibilityNode,
    picking::hover::PickingInteraction,
    prelude::*,
    ui::Selected,
    ui_widgets::{MenuAction, MenuEvent, MenuFocusState, ValueChange},
};
use bevy_ui_component_prototypes::{SelectedIndex, select};

mod accessibility;
mod choose;
mod disabled;
mod highlight;
mod label;
mod open_close;
mod popup_frame;
mod rows;
mod selected_row;
mod structure;
mod theme;
mod trigger_style;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

#[derive(Component, Clone)]
struct Marker(u8);

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, Fruit)>);

fn app_that_logs_requests() -> App {
    let mut app = test_app();
    app.init_resource::<Requests>().add_observer(
        |change: On<ValueChange<Fruit>>, mut log: ResMut<Requests>| {
            log.0.push((change.source, change.value));
        },
    );
    app
}

fn report_row(app: &mut App, source: Entity, row: Entity) {
    app.world_mut().trigger(ValueChange::<Entity> {
        source,
        value: row,
        is_final: true,
    });
    app.update();
}

fn text(name: &'static str) -> impl Bundle + Clone {
    Text::new(name)
}

fn fruit_select() -> impl Bundle {
    select::<Fruit>()
        .placeholder(text("Pick one"))
        .option(Fruit::Apple, text("Apple"))
        .option(Fruit::Banana, text("Banana"))
        .option(Fruit::Cherry, text("Cherry"))
        .build()
}

fn spawn_fruit(app: &mut App) -> Entity {
    app.world_mut().spawn(fruit_select()).id()
}

fn spawn_preselected(app: &mut App, value: Fruit) -> Entity {
    app.world_mut()
        .spawn(
            select::<Fruit>()
                .placeholder(text("Pick one"))
                .options([
                    (Fruit::Apple, text("Apple")),
                    (Fruit::Banana, text("Banana")),
                    (Fruit::Cherry, text("Cherry")),
                ])
                .selected(value)
                .build(),
        )
        .id()
}

fn spawn_fruit_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((fruit_select(), ChildOf(parent)))
        .id()
}

fn trigger(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(root).unwrap()[0]
}

fn popup(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(root).unwrap()[1]
}

fn list(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(popup(app, root)).unwrap()[0]
}

fn rows(app: &App, root: Entity) -> Vec<Entity> {
    app.world()
        .get::<Children>(list(app, root))
        .unwrap()
        .to_vec()
}

fn selected_rows(app: &App, root: Entity) -> Vec<bool> {
    rows(app, root)
        .iter()
        .map(|row| app.world().get::<Selected>(*row).is_some())
        .collect()
}

fn row_label(app: &App, row: Entity) -> Entity {
    app.world().get::<Children>(row).unwrap()[0]
}

fn content(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(trigger(app, root)).unwrap()[0]
}

/// The label that is shown now: the chosen option's, or the placeholder.
fn shown(app: &App, root: Entity) -> Option<Entity> {
    app.world()
        .get::<Children>(content(app, root))
        .and_then(|children| children.first().copied())
}

fn trigger_node(app: &App, root: Entity) -> &AccessibilityNode {
    app.world()
        .get::<AccessibilityNode>(trigger(app, root))
        .unwrap()
}

fn index(app: &App, root: Entity) -> Option<usize> {
    app.world().get::<SelectedIndex>(root).unwrap().index
}

fn label_color(app: &App, label: Entity) -> Color {
    app.world().get::<TextColor>(label).unwrap().0
}

fn point_at(app: &mut App, row: Entity, interaction: PickingInteraction) {
    app.world_mut().entity_mut(row).insert(interaction);
    app.update();
}

fn menu_event(app: &mut App, source: Entity, action: MenuAction) {
    app.world_mut().trigger(MenuEvent { source, action });
    app.update();
}

fn visibility(app: &App, entity: Entity) -> Visibility {
    *app.world().get::<Visibility>(entity).unwrap()
}

fn focus_state(app: &App, popup: Entity) -> MenuFocusState {
    app.world().get::<MenuFocusState>(popup).unwrap().clone()
}
