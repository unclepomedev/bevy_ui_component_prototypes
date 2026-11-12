//! Tests of the states of the parts, the label, and the icons.

use super::{
    cells, children_of, color_of, header, mark, next_button, spawn_default, spawn_stepper,
};
use crate::{
    probe::probe_theme,
    support::{border_color, test_app},
};
use bevy::{picking::hover::PickingInteraction, prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::stepper::{StepperMarks, stepper};

#[derive(Component)]
struct Caption;

#[test]
fn hovering_and_pressing_a_cell_change_its_color_only() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);
    app.update();
    let style = probe_theme().stepper;
    let cells = cells(&app, stepper);

    app.world_mut()
        .entity_mut(cells[4])
        .insert(PickingInteraction::Hovered);
    app.update();
    assert_eq!(color_of(&app, cells[4]), style.upcoming.hovered);
    assert_eq!(color_of(&app, cells[3]), style.upcoming.normal);

    app.world_mut()
        .entity_mut(cells[4])
        .insert(PickingInteraction::Pressed);
    app.update();
    assert_eq!(color_of(&app, cells[4]), style.upcoming.pressed);

    app.world_mut()
        .entity_mut(cells[4])
        .insert(PickingInteraction::None);
    app.update();
    assert_eq!(color_of(&app, cells[4]), style.upcoming.normal);
}

#[test]
fn hovering_a_button_changes_its_background_and_its_arrow() {
    let mut app = test_app();
    let stepper = spawn_default(&mut app);
    app.update();
    let style = probe_theme().stepper;
    let next = next_button(&app, stepper);

    app.world_mut()
        .entity_mut(next)
        .insert(PickingInteraction::Hovered);
    app.update();

    assert_eq!(color_of(&app, next), style.button.hovered);
    let arrow = children_of(&app, next)[0];
    assert_eq!(
        border_color(&app, arrow),
        BorderColor::all(style.arrow.hovered)
    );
}

#[test]
fn the_marks_follow_the_stepper_marks() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(3).marks([0]));
    app.update();
    let shown = |app: &App| -> Vec<bool> {
        cells(app, stepper)
            .into_iter()
            .map(|cell| color_of(app, mark(app, cell)) != Color::NONE)
            .collect()
    };
    assert_eq!(shown(&app), [true, false, false]);

    app.world_mut()
        .entity_mut(stepper)
        .insert(StepperMarks::new([1, 2]));
    app.update();
    assert_eq!(shown(&app), [false, true, true]);
}

#[test]
fn the_label_takes_the_theme_color_and_dims_when_disabled() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(3).label_bundle(Caption));
    app.update();
    let style = probe_theme().stepper;
    let label = header(&app, stepper)[1];
    let color = |app: &App| app.world().get::<TextColor>(label).unwrap().0;
    assert_eq!(color(&app), style.label.normal);

    app.world_mut()
        .entity_mut(stepper)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(color(&app), style.label.disabled);
}

#[test]
fn a_compact_stepper_paints_its_buttons() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5).compact().label_bundle(Caption));
    app.update();

    let style = probe_theme().stepper;
    assert_eq!(
        color_of(&app, next_button(&app, stepper)),
        style.button.normal
    );
    assert_eq!(header(&app, stepper).len(), 3);
}

#[test]
fn an_icon_takes_the_arrow_color() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(5).next_icon(Handle::default()));
    app.update();

    let icon = children_of(&app, next_button(&app, stepper))[0];
    let image = app.world().get::<ImageNode>(icon).unwrap();
    assert_eq!(image.color, probe_theme().stepper.arrow.normal);
}

#[test]
fn a_single_position_shows_its_cell_as_current() {
    let mut app = test_app();
    let stepper = spawn_stepper(&mut app, stepper().steps(1));
    app.update();

    let cells = cells(&app, stepper);
    assert_eq!(cells.len(), 1);
    assert_eq!(
        color_of(&app, cells[0]),
        probe_theme().stepper.current.normal
    );
}
