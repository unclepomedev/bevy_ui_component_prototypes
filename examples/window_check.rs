//! Sample for verifying the window. It is also what the windows of a deduction game look like: one window
//! shows the hints, another takes the answer.
//!
//! Execution: cargo run --example window_check
//!
//! What the app does (the library only asks; these are the places where this example answers):
//!   - `WindowDefaultsPlugin` raises, moves, resizes and closes the windows. An app with its own policy
//!     leaves it out and observes `RaiseWindow`, `MoveWindow`, `ResizeWindow` and `CloseWindow` itself.
//!   - The selects and the slider are controlled too: the observers in this file apply their changes.
//!   - The pointer shape over the frame is set here, from `WindowGrip`.
//!   - The hint window is in a panel with the mac `Theme`, and the answer window in one with the Windows
//!     `Theme`: that is all it takes to give a window its look.
//!
//! Keys: [E] puts the answer window at the bottom right of the screen and back. [H] and [A] open the hint
//! window and the answer window again after they were closed.
//!
//! What to verify:
//!   1. Look. The hint window has the mac look: a circular red close button at the left of the title bar, a
//!      centered title, and the cross only while the pointer is over the button. The frame has rounded
//!      corners and the title bar follows them. The answer window has the Windows look: a square close
//!      button at the right end of the bar, flush with its top and bottom, a left-aligned title, red under
//!      the pointer.
//!   2. The title of the hint window is longer than the room it has: it is cut off before the close
//!      button, and again at the new width when the window is resized.
//!   3. Close. Press on a close button: it gets darker. Release on it: the window closes. Press on it and
//!      release away from it: nothing closes. [H] and [A] open the window again.
//!   4. Raise. The windows overlap. Press anywhere on a window (the title bar, the text, the frame, a
//!      select, the slider, Submit): it comes to the front. Only the front window has the bright title bar
//!      and title. Press on the desktop: nothing changes.
//!   5. Move. Drag a title bar: the window follows the pointer, also over the other window. Dragging the
//!      content, the frame or the close button does not move it, and neither does the right button. Past
//!      the edge of the screen the window stops with 64 px of its width inside at the left and the right,
//!      with its top edge at the top, and with the title bar visible at the bottom.
//!   6. Resize. Drag the frame at a side or a corner: the edge follows the pointer. The strip you can grab
//!      is 8 px thick, half of it outside the frame, and cannot be seen. A window cannot be smaller than
//!      120 x 60: the edge you drag stops and the opposite one stays. The pointer is a resize arrow over a
//!      strip and keeps the shape during a drag. The title bar still moves the window, except at its
//!      outermost few pixels.
//!   7. The answer window works: click the selects, drag the slider (the window does not move), press Submit.
//!      Butler and Poison are the right answer.
//!   8. The list of a select opens past the edge of its window and is in front of every window. Escape
//!      closes the open list and does not close the window. After [E] the list stays on the screen (it
//!      opens upward, or is moved). Move or resize a window while its list is open: see what happens to the
//!      list, and tell what you see.
//!   9. Scroll. The wheel scrolls the hint text, which is cut off at the edge of the content area. A thin
//!      bar at the right edge shows where the visible part is, and moves with the wheel. Make the window
//!      smaller: the bar gets shorter and the wheel still reaches the end of the text. Make it so tall that
//!      all of the text fits: the bar goes away.
//!  10. Keyboard. Tab moves the focus through the widgets of a window, and its close button is the last
//!      stop in it; after that the focus goes to the other window. The close button shows a ring after Tab
//!      and not after a click. Enter or Space on it closes the window. Tab into a window behind the other:
//!      it comes to the front.
//!  11. After a window closes, the other one is the bright one.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::TabNavigationPlugin,
    picking::events::{PointerDragEnd, PointerDragStart, PointerOut, PointerOver},
    prelude::*,
    ui_widgets::{Activate, ScrollArea, SliderValue, ValueChange},
    window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};
use bevy_ui_component_prototypes::{
    SelectKey, SelectedOption, Theme, UiComponentPrototypesPlugin, WindowDefaultsPlugin,
    WindowEdge, WindowGrip, WindowMetrics, WindowRect, WindowStyle, icon_button, scroll_indicator,
    select, slider, window,
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

/// The panels that carry the `Theme` of the windows in them.
#[derive(Resource)]
struct Panels {
    hint: Entity,
    answer: Entity,
}

#[derive(Resource, Default)]
struct AtEdge(bool);

/// The grip under the pointer, and the one being dragged. The cursor keeps the shape of the dragged one,
/// also when the pointer is not over its strip any more.
#[derive(Resource, Default)]
struct GripCursor {
    hovered: Option<WindowEdge>,
    dragging: Option<WindowEdge>,
}

const HINT_RECT: WindowRect = WindowRect::new(40.0, 40.0, 360.0, 260.0);
const ANSWER_RECT: WindowRect = WindowRect::new(260.0, 110.0, 360.0, 300.0);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            UiComponentPrototypesPlugin,
            TabNavigationPlugin,
            // App policy: raise, move, resize and close the windows as asked.
            WindowDefaultsPlugin::default(),
        ))
        .init_resource::<AtEdge>()
        .init_resource::<GripCursor>()
        .add_observer(grip_over)
        .add_observer(grip_out)
        .add_observer(grip_drag_start)
        .add_observer(grip_drag_end)
        // App policy: apply what the widgets ask for.
        .add_observer(on_choice::<Suspect>)
        .add_observer(on_choice::<Cause>)
        .add_observer(on_slider)
        .add_observer(on_submit)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_edge.run_if(input_just_pressed(KeyCode::KeyE)),
                open_hint.run_if(input_just_pressed(KeyCode::KeyH)),
                open_answer.run_if(input_just_pressed(KeyCode::KeyA)),
                show_grip_cursor,
            ),
        )
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
    let panels = Panels {
        hint: spawn_panel(&mut commands, desktop, mac_theme()),
        answer: spawn_panel(&mut commands, desktop, windows_theme()),
    };
    spawn_hint_window(&mut commands, panels.hint);
    spawn_answer_window(&mut commands, panels.answer);
    commands.insert_resource(panels);
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

fn mac_theme() -> Theme {
    Theme {
        window: WindowStyle::mac(),
        window_metrics: WindowMetrics::mac(),
        ..default()
    }
}

fn windows_theme() -> Theme {
    Theme {
        window: WindowStyle::windows(),
        window_metrics: WindowMetrics::windows(),
        ..default()
    }
}

/// A node as large as the desktop that carries a `Theme`; the windows in it have that look.
fn spawn_panel(commands: &mut Commands, desktop: Entity, theme: Theme) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            theme,
            ChildOf(desktop),
            // The panel is only there to carry the theme: the pointer goes through it.
            Pickable::IGNORE,
        ))
        .id()
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

fn spawn_hint_window(commands: &mut Commands, panel: Entity) {
    commands.spawn((
        window()
            .rect(HINT_RECT)
            .title(Text::new(
                "Hints, with a title that does not fit in the window",
            ))
            .content_bundle(Children::spawn(Spawn((
                // The indicator is placed in the node that holds the scroll area and is as large as it.
                Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                    ..default()
                },
                Children::spawn((
                    Spawn((
                        Node {
                            flex_grow: 1.0,
                            min_height: px(0),
                            padding: UiRect::all(px(10)),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollArea,
                        Children::spawn(Spawn(Text::new(HINTS))),
                    )),
                    Spawn(scroll_indicator()),
                )),
            ))))
            .build(),
        HintWindow,
        ChildOf(panel),
    ));
}

fn spawn_answer_window(commands: &mut Commands, panel: Entity) {
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
        ChildOf(panel),
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

fn open_hint(mut commands: Commands, panels: Res<Panels>, existing: Query<(), With<HintWindow>>) {
    if existing.is_empty() {
        spawn_hint_window(&mut commands, panels.hint);
    }
}

fn open_answer(
    mut commands: Commands,
    panels: Res<Panels>,
    existing: Query<(), With<AnswerWindow>>,
) {
    if existing.is_empty() {
        spawn_answer_window(&mut commands, panels.answer);
    }
}

/// App policy: the pointer shape is the app's. The library only says which strip is under the pointer.
fn grip_over(over: On<PointerOver>, grips: Query<&WindowGrip>, mut state: ResMut<GripCursor>) {
    if let Ok(WindowGrip { edge }) = grips.get(over.entity) {
        state.hovered = Some(*edge);
    }
}

fn grip_out(out: On<PointerOut>, grips: Query<&WindowGrip>, mut state: ResMut<GripCursor>) {
    // Only the grip that is the hovered one lets go: the pointer may have entered another one already.
    if let Ok(WindowGrip { edge }) = grips.get(out.entity)
        && state.hovered == Some(*edge)
    {
        state.hovered = None;
    }
}

fn grip_drag_start(
    start: On<PointerDragStart>,
    grips: Query<&WindowGrip>,
    mut state: ResMut<GripCursor>,
) {
    if let Ok(WindowGrip { edge }) = grips.get(start.entity) {
        state.dragging = Some(*edge);
    }
}

fn grip_drag_end(
    end: On<PointerDragEnd>,
    grips: Query<&WindowGrip>,
    mut state: ResMut<GripCursor>,
) {
    if grips.contains(end.entity) {
        state.dragging = None;
    }
}

fn show_grip_cursor(
    state: Res<GripCursor>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if !state.is_changed() {
        return;
    }
    let icon = match state.dragging.or(state.hovered) {
        Some(WindowEdge::Top | WindowEdge::Bottom) => SystemCursorIcon::NsResize,
        Some(WindowEdge::Left | WindowEdge::Right) => SystemCursorIcon::EwResize,
        Some(WindowEdge::TopLeft | WindowEdge::BottomRight) => SystemCursorIcon::NwseResize,
        Some(WindowEdge::TopRight | WindowEdge::BottomLeft) => SystemCursorIcon::NeswResize,
        None => SystemCursorIcon::Default,
    };
    commands.entity(*window).insert(CursorIcon::from(icon));
}
