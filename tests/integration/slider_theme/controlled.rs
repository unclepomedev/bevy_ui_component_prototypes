//! Tests of the controlled value.

use super::{fill_width, spawn_slider, thumb_left};
use crate::support::test_app;
use bevy::{
    prelude::*,
    ui_widgets::{SetSliderValue, SliderPlugin, SliderValue, SliderValueChange, ValueChange},
};

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, f32)>);

fn app_with_slider_behavior() -> App {
    let mut app = test_app();
    app.add_plugins(SliderPlugin)
        .init_resource::<Requests>()
        .add_observer(|change: On<ValueChange<f32>>, mut log: ResMut<Requests>| {
            log.0.push((change.source, change.value));
        });
    app
}

#[test]
fn a_request_reports_the_new_value_but_never_changes_the_value_by_itself() {
    let mut app = app_with_slider_behavior();
    let slider = spawn_slider(&mut app);
    app.update();

    app.world_mut().trigger(SetSliderValue {
        entity: slider,
        change: SliderValueChange::Absolute(0.75),
    });
    app.update();

    assert_eq!(app.world().resource::<Requests>().0, vec![(slider, 0.75)]);
    assert_eq!(
        app.world().get::<SliderValue>(slider),
        Some(&SliderValue(0.0))
    );
    assert_eq!(thumb_left(&app, slider), Val::Percent(0.0));
}

#[test]
fn the_app_applying_the_request_moves_the_parts() {
    let mut app = app_with_slider_behavior();
    app.add_observer(|change: On<ValueChange<f32>>, mut commands: Commands| {
        commands
            .entity(change.source)
            .insert(SliderValue(change.value));
    });
    let slider = spawn_slider(&mut app);
    app.update();

    app.world_mut().trigger(SetSliderValue {
        entity: slider,
        change: SliderValueChange::Absolute(0.75),
    });
    app.update();

    assert_eq!(thumb_left(&app, slider), Val::Percent(75.0));
    assert_eq!(fill_width(&app, slider), Val::Percent(75.0));
}
