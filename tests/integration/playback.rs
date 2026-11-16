//! Behavior tests for the playback clock: it follows the real time of the frame, and says when it ends.

use crate::support::test_app;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use bevy_ui_component_prototypes::playback::{PlaybackClock, PlaybackEnded};
use std::time::Duration;

#[derive(Resource, Default)]
struct Ended(Vec<Entity>);

/// An app whose real time moves by `step` seconds every update, and that records the ends.
fn timed_app(step: f32) -> App {
    let mut app = test_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        step,
    )))
    .init_resource::<Ended>()
    .add_observer(|ended: On<PlaybackEnded>, mut log: ResMut<Ended>| log.0.push(ended.clock));
    // The first update only sets the clock; there is no delta before the second.
    app.update();
    app
}

fn spawn(app: &mut App, clock: PlaybackClock) -> Entity {
    app.world_mut().spawn(clock).id()
}

fn clock(app: &App, entity: Entity) -> PlaybackClock {
    *app.world().get::<PlaybackClock>(entity).unwrap()
}

fn with(app: &mut App, entity: Entity, change: impl FnOnce(&mut PlaybackClock)) {
    change(&mut app.world_mut().get_mut::<PlaybackClock>(entity).unwrap());
}

fn ended(app: &App) -> Vec<Entity> {
    app.world().resource::<Ended>().0.clone()
}

#[test]
fn a_paused_clock_does_not_move() {
    let mut app = timed_app(0.5);
    let id = spawn(&mut app, PlaybackClock::new(10.0));

    app.update();
    app.update();

    assert_eq!(clock(&app, id).position(), 0.0);
}

#[test]
fn a_playing_clock_moves_by_the_time_of_each_frame_times_its_speed() {
    let mut app = timed_app(0.5);
    let id = spawn(&mut app, PlaybackClock::new(10.0).with_speed(2.0));
    with(&mut app, id, PlaybackClock::play);

    app.update();
    assert_eq!(clock(&app, id).position(), 1.0);
    app.update();
    assert_eq!(clock(&app, id).position(), 2.0);
}

#[test]
fn a_clock_stops_at_its_end_and_triggers_the_event_once() {
    let mut app = timed_app(0.5);
    let id = spawn(&mut app, PlaybackClock::new(1.0));
    with(&mut app, id, PlaybackClock::play);

    for _ in 0..5 {
        app.update();
    }

    let clock = clock(&app, id);
    assert_eq!(clock.position(), 1.0);
    assert!(!clock.is_playing());
    assert_eq!(ended(&app), [id]);
}

#[test]
fn a_clock_that_plays_again_triggers_the_event_again() {
    let mut app = timed_app(0.5);
    let id = spawn(&mut app, PlaybackClock::new(1.0));

    for _ in 0..2 {
        with(&mut app, id, PlaybackClock::play);
        app.update();
        app.update();
    }

    assert_eq!(ended(&app), [id, id]);
}

#[test]
fn a_seek_while_playing_moves_the_position_and_the_clock_goes_on() {
    let mut app = timed_app(0.5);
    let id = spawn(&mut app, PlaybackClock::new(10.0));
    with(&mut app, id, PlaybackClock::play);
    app.update();

    with(&mut app, id, |clock| clock.seek(6.0));
    app.update();

    assert_eq!(clock(&app, id).position(), 6.5);
}

#[test]
fn a_step_pauses_the_clock() {
    let mut app = timed_app(0.5);
    let id = spawn(&mut app, PlaybackClock::new(10.0).with_frame_rate(2.0));
    with(&mut app, id, PlaybackClock::play);

    with(&mut app, id, |clock| clock.step(1));
    app.update();

    assert_eq!(clock(&app, id).position(), 0.5);
}

#[test]
fn the_clocks_are_independent_and_only_the_one_that_ended_triggers_the_event() {
    let mut app = timed_app(0.5);
    let short = spawn(&mut app, PlaybackClock::new(0.5));
    let long = spawn(&mut app, PlaybackClock::new(10.0));
    let paused = spawn(&mut app, PlaybackClock::new(10.0));
    for id in [short, long] {
        with(&mut app, id, PlaybackClock::play);
    }

    app.update();
    app.update();

    assert_eq!(ended(&app), [short]);
    assert_eq!(clock(&app, long).position(), 1.0);
    assert_eq!(clock(&app, paused).position(), 0.0);
}

#[test]
fn a_clock_does_not_follow_the_time_of_the_game() {
    let mut app = timed_app(0.5);
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(0.0);
    let id = spawn(&mut app, PlaybackClock::new(10.0));
    with(&mut app, id, PlaybackClock::play);

    app.update();

    assert_eq!(clock(&app, id).position(), 0.5);
}
