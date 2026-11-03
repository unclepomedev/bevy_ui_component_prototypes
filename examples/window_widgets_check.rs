//! Sample for verifying the window with the widgets of this crate in it. It is also what the windows of a
//! deduction game look like: one window shows the hints, another takes the answer.
//!
//! Execution: cargo run --example window_widgets_check
//!
//! The window is controlled: the example adds `WindowDefaultsPlugin`, which raises, moves, resizes and closes
//! the windows. The selects and the slider are controlled too: the observers of this example apply their
//! changes.
//!
//! What to verify:
//!   1. The two windows hold a select, a slider and a button (the answer window) and scrolling text (the hint
//!      window). Every one of them works inside the window: click the selects, drag the slider, press Submit.
//!   2. Pressing on a select, on the slider or on the button brings its window to the front. Dragging the
//!      slider does not move the window.
//!   3. The list of a select opens past the edge of its window, and is in front of every window, also of the
//!      one that was in front before the press. (The window comes to the front when you press the select.)
//!   4. The wheel scrolls the hint text inside the hint window. The text is cut off at the edge of the content
//!      area. Make the window smaller: the text is still cut off at the edge, and the wheel still reaches the
//!      end of it.
//!   5. [E] puts the answer window at the bottom right of the screen, and [E] again puts it back. Open the
//!      selects in it: the list stays on the screen (it opens upward, or is moved).
//!   6. Submit says whether the answer is right. Choose "Butler" and "Poison" for the right answer.
//!   7. Escape closes the open list and does not close the window.
//!   8. Move or resize a window while its list is open: see what happens to the list, and tell what you see.
//!   9. Close a window with its close button. [H] and [A] open the hint window and the answer window again.

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    ui_widgets::{Activate, ScrollArea, SliderValue, ValueChange},
    window::PrimaryWindow,
};
use bevy_ui_component_prototypes::{
    GlobalTheme, SelectKey, SelectedOption, Theme, UiComponentPrototypesPlugin,
    WindowDefaultsPlugin, WindowMetrics, WindowRect, WindowStyle, icon_button, select, slider,
    window,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Suspect {
    Butler,
    Gardener,
    Cook,
    Driver,
}

impl Suspect {
    const ALL: [Self; 4] = [Self::Butler, Self::Gardener, Self::Cook, Self::Driver];

    fn name(self) -> &'static str {
        match self {
            Self::Butler => "Butler",
            Self::Gardener => "Gardener",
            Self::Cook => "Cook",
            Self::Driver => "Driver",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cause {
    Poison,
    Knife,
    Fall,
    Fire,
    Drowning,
    Gunshot,
    Strangling,
    Illness,
    Unknown,
}

impl Cause {
    const ALL: [Self; 9] = [
        Self::Poison,
        Self::Knife,
        Self::Fall,
        Self::Fire,
        Self::Drowning,
        Self::Gunshot,
        Self::Strangling,
        Self::Illness,
        Self::Unknown,
    ];
}

#[derive(Component)]
struct HintWindow;
#[derive(Component)]
struct AnswerWindow;
#[derive(Component)]
struct Submit;
#[derive(Component)]
struct ResultText;

#[derive(Resource)]
struct Desktop(Entity);

#[derive(Resource, Default)]
struct AtEdge(bool);

const ANSWER_RECT: WindowRect = WindowRect::new(440.0, 60.0, 360.0, 300.0);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            UiComponentPrototypesPlugin,
            WindowDefaultsPlugin::default(),
        ))
        .insert_resource(GlobalTheme(Theme {
            window: WindowStyle::mac(),
            window_metrics: WindowMetrics::mac(),
            ..default()
        }))
        .init_resource::<AtEdge>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_edge.run_if(input_just_pressed(KeyCode::KeyE)),
                open_hint.run_if(input_just_pressed(KeyCode::KeyH)),
                open_answer.run_if(input_just_pressed(KeyCode::KeyA)),
            ),
        )
        .add_observer(on_choice::<Suspect>)
        .add_observer(on_choice::<Cause>)
        .add_observer(on_slider)
        .add_observer(on_submit)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    // The desktop: one node as large as the screen.
    let desktop = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            ..default()
        })
        .id();
    commands.insert_resource(Desktop(desktop));
    spawn_hint_window(&mut commands, desktop);
    spawn_answer_window(&mut commands, desktop);
    commands.spawn((
        Text::new("[E] answer window to the screen edge   [H] hint window   [A] answer window"),
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            bottom: px(8),
            ..default()
        },
        // Under the windows: they start at 10.
        GlobalZIndex(1),
    ));
}

const HINTS: &str = "\
Hint 1. The study door was locked from the inside.\n\
Hint 2. A cup of tea was left on the desk, half drunk.\n\
Hint 3. The gardener was seen at the greenhouse until nine.\n\
Hint 4. The cook says the master asked for tea at eight.\n\
Hint 5. The driver's coat was wet, and it had not rained.\n\
Hint 6. The butler carried the tray to the study.\n\
Hint 7. A small bottle was found behind the books.\n\
Hint 8. The clock in the hall stopped at ten past nine.\n\
Hint 9. Nobody heard a sound from the study.\n\
Hint 10. The window of the study was open a hand's width.\n\
Hint 11. The master was alone when the tea was brought.\n\
Hint 12. The tray came back to the kitchen empty.";

fn spawn_hint_window(commands: &mut Commands, desktop: Entity) {
    commands.spawn((
        window()
            .rect(WindowRect::new(40.0, 40.0, 360.0, 260.0))
            .title(Text::new("Hints"))
            .content_bundle(Children::spawn(Spawn((
                // The content area of a window is a column that clips. This node fills it and scrolls.
                Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                    padding: UiRect::all(px(10)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollArea,
                Children::spawn(Spawn(Text::new(HINTS))),
            ))))
            .build(),
        HintWindow,
        ChildOf(desktop),
    ));
}

fn spawn_answer_window(commands: &mut Commands, desktop: Entity) {
    commands.spawn((
        window()
            .rect(ANSWER_RECT)
            .title(Text::new("Answer"))
            .content_bundle(Children::spawn(Spawn((
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    padding: UiRect::all(px(12)),
                    ..default()
                },
                Children::spawn((
                    Spawn(Text::new("Who did it?")),
                    Spawn(
                        select::<Suspect>()
                            .placeholder(Text::new("Pick a suspect"))
                            .options(Suspect::ALL.map(|s| (s, Text::new(s.name()))))
                            .build(),
                    ),
                    Spawn(Text::new("How?")),
                    Spawn(
                        select::<Cause>()
                            .placeholder(Text::new("Pick a cause"))
                            .options(Cause::ALL.map(|c| (c, Text::new(format!("{c:?}")))))
                            .build(),
                    ),
                    Spawn(Text::new("How sure are you?")),
                    Spawn(slider().value(0.5).build()),
                    Spawn((icon_button().label("Submit").build(), Submit)),
                    Spawn((Text::new(""), ResultText)),
                )),
            ))))
            .build(),
        AnswerWindow,
        ChildOf(desktop),
    ));
}

/// The app's policy: apply what a select asks for.
fn on_choice<V: SelectKey>(change: On<ValueChange<V>>, mut commands: Commands) {
    commands.entity(change.source).insert(SelectedOption {
        value: change.value.clone(),
    });
}

fn on_slider(change: On<ValueChange<f32>>, mut commands: Commands) {
    commands
        .entity(change.source)
        .insert(SliderValue(change.value));
}

fn on_submit(
    activate: On<Activate>,
    submits: Query<(), With<Submit>>,
    suspects: Query<&SelectedOption<Suspect>>,
    causes: Query<&SelectedOption<Cause>>,
    mut result: Single<&mut Text, With<ResultText>>,
) {
    if !submits.contains(activate.entity) {
        return;
    }
    let text = match (suspects.iter().next(), causes.iter().next()) {
        (Some(suspect), Some(cause)) => {
            if suspect.value == Suspect::Butler && cause.value == Cause::Poison {
                "Right."
            } else {
                "Not quite."
            }
        }
        _ => "Choose both first.",
    };
    result.0 = text.into();
}

fn toggle_edge(
    mut at_edge: ResMut<AtEdge>,
    screen: Single<&Window, With<PrimaryWindow>>,
    mut windows: Query<&mut WindowRect, With<AnswerWindow>>,
) {
    at_edge.0 = !at_edge.0;
    let rect = if at_edge.0 {
        WindowRect {
            x: screen.width() - ANSWER_RECT.width - 8.0,
            y: screen.height() - ANSWER_RECT.height - 8.0,
            ..ANSWER_RECT
        }
    } else {
        ANSWER_RECT
    };
    for mut window in &mut windows {
        *window = rect;
    }
}

fn open_hint(mut commands: Commands, desktop: Res<Desktop>, existing: Query<(), With<HintWindow>>) {
    if existing.is_empty() {
        spawn_hint_window(&mut commands, desktop.0);
    }
}

fn open_answer(
    mut commands: Commands,
    desktop: Res<Desktop>,
    existing: Query<(), With<AnswerWindow>>,
) {
    if existing.is_empty() {
        spawn_answer_window(&mut commands, desktop.0);
    }
}
