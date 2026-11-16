//! Sample for verifying the playback clock.
//!
//! Execution: cargo run --example playback_check
//!
//! The clock only keeps time. Everything else in this example is the app's: the buttons ask the clock to
//! play, step or change speed, the slider is controlled (it reports a value, the app seeks the clock), and a
//! system writes the position to the text and to the slider. The same widgets and the same clock are what a
//! replay window is made of.
//!
//! What to verify:
//!   1. The clock is paused at 0.00 s to start with and stays there.
//!   2. [Play / Pause]: the position runs in real time, and the slider follows it. Pressing again stops it.
//!   3. [Speed] goes through 0.5x, 1x and 2x: the position runs at that speed (compare with a watch).
//!   4. At 10.00 s the clock stops by itself, the state turns to "paused" and "Ended" counts up by one, not
//!      more. [Play / Pause] then starts it again from 0.00 s.
//!   5. Drag the slider while it plays: the position jumps to the pointer and the clock goes on from there.
//!      Drag it while it is paused: it stays paused at the new position.
//!   6. [< Frame] and [Frame >] pause the clock and move it by one frame (1/30 s, so the text shows
//!      0.03, 0.07, 0.10 ...). They stop at 0.00 s and at 10.00 s and do not go beyond.
//!   7. [Restart] goes back to 0.00 s and plays.

use bevy::{
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui_widgets::{Activate, SliderValue, ValueChange},
};
use bevy_ui_component_prototypes::{
    UiComponentPrototypesPlugin,
    button::icon_button,
    playback::{PlaybackClock, PlaybackEnded},
    slider::slider,
};

const DURATION: f32 = 10.0;
const SPEEDS: [f32; 3] = [0.5, 1.0, 2.0];

/// What a button asks the clock to do.
#[derive(Component, Clone, Copy)]
enum Action {
    PlayPause,
    StepBack,
    StepForward,
    Speed,
    Restart,
}

/// The slider that seeks the clock.
#[derive(Component)]
struct Seek;

#[derive(Component)]
struct Status;

#[derive(Resource, Default)]
struct EndedCount(u32);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TabNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .init_resource::<EndedCount>()
        .add_systems(Startup, setup)
        .add_systems(Update, (show_status, follow_slider))
        .add_observer(on_action)
        .add_observer(on_seek)
        .add_observer(count_ends)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(PlaybackClock::new(DURATION));

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(20),
                ..default()
            },
            TabGroup::default(),
        ))
        .with_children(|root| {
            root.spawn((Text::new(""), Status));
            root.spawn(Node {
                width: px(480),
                ..default()
            })
            .with_children(|slot| {
                slot.spawn((slider().build(), Seek));
            });
            root.spawn(Node {
                column_gap: px(8),
                ..default()
            })
            .with_children(|row| {
                for (label, action) in [
                    ("< Frame", Action::StepBack),
                    ("Play / Pause", Action::PlayPause),
                    ("Frame >", Action::StepForward),
                    ("Speed", Action::Speed),
                    ("Restart", Action::Restart),
                ] {
                    row.spawn((icon_button().label(label).build(), action));
                }
            });
        });
}

/// App policy: what each button does to the clock.
fn on_action(
    activate: On<Activate>,
    actions: Query<&Action>,
    mut clocks: Query<&mut PlaybackClock>,
) {
    let (Ok(action), Ok(mut clock)) = (actions.get(activate.entity), clocks.single_mut()) else {
        return;
    };
    match action {
        Action::PlayPause => clock.toggle(),
        Action::StepBack => clock.step(-1),
        Action::StepForward => clock.step(1),
        Action::Speed => {
            let next = SPEEDS.iter().position(|speed| *speed == clock.speed());
            clock.set_speed(SPEEDS[next.map_or(0, |at| (at + 1) % SPEEDS.len())]);
        }
        Action::Restart => {
            clock.seek(0.0);
            clock.play();
        }
    }
}

/// The slider is controlled: it asks for a position, and the clock takes it.
fn on_seek(
    change: On<ValueChange<f32>>,
    seeks: Query<(), With<Seek>>,
    mut clocks: Query<&mut PlaybackClock>,
) {
    if let (true, Ok(mut clock)) = (seeks.contains(change.source), clocks.single_mut()) {
        clock.seek_fraction(change.value);
    }
}

fn count_ends(_: On<PlaybackEnded>, mut count: ResMut<EndedCount>) {
    count.0 += 1;
}

/// The slider shows where the clock is.
fn follow_slider(
    clocks: Query<&PlaybackClock, Changed<PlaybackClock>>,
    seeks: Query<Entity, With<Seek>>,
    mut commands: Commands,
) {
    if let (Ok(clock), Ok(seek)) = (clocks.single(), seeks.single()) {
        commands.entity(seek).insert(SliderValue(clock.fraction()));
    }
}

fn show_status(
    clocks: Query<&PlaybackClock>,
    count: Res<EndedCount>,
    mut status: Query<&mut Text, With<Status>>,
) {
    let (Ok(clock), Ok(mut status)) = (clocks.single(), status.single_mut()) else {
        return;
    };
    status.0 = format!(
        "{:.2} / {:.2} s   {}   {}x   Ended: {}",
        clock.position(),
        clock.duration(),
        if clock.is_playing() {
            "playing"
        } else {
            "paused"
        },
        clock.speed(),
        count.0,
    );
}
