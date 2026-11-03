//! Sample for verifying the look of the window.
//!
//! Execution: cargo run --example window_check
//!
//! A window comes to the front when pressed, moves when its title bar is dragged, resizes when its frame is
//! dragged, and closes with its close button.
//!
//! What to verify:
//!   1. The window on the left has the mac look: a circular red close button at the left of the title bar
//!      and a centered title. The cross shows only while the pointer is over the button. The frame has rounded
//!      corners, and the title bar follows them: it does not stick out at the top corners.
//!   2. The window on the right has the Windows look: a square close button at the right end of the title
//!      bar, flush with its top and bottom, and a left-aligned title. It turns red under the pointer.
//!   3. The mac close button is a soft red that keeps its color under the pointer: only the cross appears.
//!      Press on a close button: it gets darker. Release on it: the window closes. Press on it and release
//!      away from it: nothing closes.
//!   4. The windows are placed by `WindowRect`. Their frames, title bars and corners come from their own
//!      `Theme` (the panels they are in), not from the global one.
//!   5. The content area clips: the long text in the narrow window is cut off at its edge and does not
//!      spill over the frame.
//!   6. [T] swaps the global theme between the mac and the Windows look. The window at the bottom, which
//!      has no `Theme` of its own, follows it; the other two do not.
//!   7. A title longer than the room it has is cut off before the close button.
//!   8. The windows overlap. Press on a window anywhere (the title bar, the text, the frame) and it comes to
//!      the front. Press on the OK button inside a window behind the others: the window comes to the front
//!      too.
//!   9. Only the window in front has the bright title bar and title. The others have the dim ones. After
//!      a press, the dim and the bright swap.
//!  10. Press on the desktop (outside the windows): nothing changes.
//!  11. Drag a title bar: the window follows the pointer, and comes to the front. Dragging the content area
//!      or the frame does not move it. Dragging from the close button does not move it either.
//!  12. Drag a window past the edge of the screen: it stops with 64 px of its width inside at the left and
//!      the right, with its top edge at the top of the screen, and with the title bar visible at the bottom.
//!  13. The pointer keeps hold of the window while it is dragged over the other windows.
//!  14. Drag with the right button: nothing moves.
//!  15. Drag the frame of a window, at a side or at a corner: the edge follows the pointer, and the window
//!      comes to the front. The strip you can grab is 8 px thick, half of it outside the frame. The strips
//!      cannot be seen; the pointer has to be near the edge.
//!  16. A window cannot be made smaller than 120 x 60: the edge you drag stops, and the opposite edge stays.
//!  17. An edge cannot be dragged out of the screen. The content area keeps clipping while the window is
//!      resized, and the long title is cut off again at the new width.
//!  18. Dragging from the middle of a title bar still moves the window; the strip at the top edge does not
//!      take over, except at the outermost few pixels of the bar.
//!  19. The pointer becomes a resize arrow over a grip: up-down at the top and the bottom, left-right at the
//!      sides, and a diagonal one at each corner. It keeps the shape during a drag, also when it is not over
//!      the strip any more, and goes back to the arrow when the drag ends. The library does not set the
//!      cursor; this example does, from `WindowGrip`.
//!  20. After a window closes, the one below it is the bright one and the others keep their order. [N] opens
//!      a new window, which comes on top of the others and is the bright one.
//!  21. Tab moves the focus from the close button of a window to the close button of the next one (the
//!      focus is shown by the outline only when the close button has one: this example does not look for it).
//!      Enter or Space on a focused close button closes its window. A window is a tab group of its own, and
//!      its close button is the last stop in it.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::TabNavigationPlugin,
    picking::events::{PointerDragEnd, PointerDragStart, PointerOut, PointerOver},
    prelude::*,
    window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, UiComponentPrototypesPlugin, WindowDefaultsPlugin, WindowEdge, WindowGrip,
    WindowMetrics, WindowRect, WindowStyle, icon_button, window,
};

#[derive(Resource)]
struct MacLook(bool);

#[derive(Resource)]
struct Desktop(Entity);

/// The grip under the pointer, and the one being dragged. The cursor keeps the shape of the dragged one,
/// also when the pointer is not over its strip any more.
#[derive(Resource, Default)]
struct GripCursor {
    hovered: Option<WindowEdge>,
    dragging: Option<WindowEdge>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            UiComponentPrototypesPlugin,
            TabNavigationPlugin,
            WindowDefaultsPlugin::default(),
        ))
        .insert_resource(GlobalTheme(theme(true)))
        .insert_resource(MacLook(true))
        .init_resource::<GripCursor>()
        .add_observer(grip_over)
        .add_observer(grip_out)
        .add_observer(grip_drag_start)
        .add_observer(grip_drag_end)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                swap_theme.run_if(input_just_pressed(KeyCode::KeyT)),
                open_window.run_if(input_just_pressed(KeyCode::KeyN)),
                show_grip_cursor,
            ),
        )
        .run();
}

fn theme(mac: bool) -> Theme {
    if mac {
        Theme {
            window: WindowStyle::mac(),
            window_metrics: WindowMetrics::mac(),
            ..default()
        }
    } else {
        Theme {
            window: WindowStyle::windows(),
            window_metrics: WindowMetrics::windows(),
            ..default()
        }
    }
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

    spawn_window(
        &mut commands,
        desktop,
        Some(theme(true)),
        WindowRect::new(40.0, 40.0, 300.0, 200.0),
        "Mac look (own Theme)",
    );
    spawn_window(
        &mut commands,
        desktop,
        Some(theme(false)),
        WindowRect::new(260.0, 110.0, 300.0, 200.0),
        "Windows look, with a title that does not fit",
    );
    spawn_window(
        &mut commands,
        desktop,
        None,
        WindowRect::new(120.0, 200.0, 200.0, 160.0),
        "Global theme",
    );
    commands.spawn((
        Text::new(
            "[T] swap the GLOBAL theme; the windows with a Theme of their own are unaffected. [N] open a window",
        ),
        Node {
            position_type: PositionType::Absolute,
            left: px(40),
            bottom: px(16),
            ..default()
        },
    ));
}

/// A window under a panel that carries its own `Theme`, or straight under the desktop.
fn spawn_window(
    commands: &mut Commands,
    desktop: Entity,
    theme: Option<Theme>,
    rect: WindowRect,
    title: &'static str,
) {
    let parent = match theme {
        Some(theme) => commands
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
            .id(),
        None => desktop,
    };
    commands.spawn((
        window()
            .rect(rect)
            .title(Text::new(title))
            .content_bundle((
                Node {
                    padding: UiRect::all(px(8)),
                    row_gap: px(8),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Children::spawn((
                    Spawn(Text::new(
                        "A long line of text that does not fit in the narrow window",
                    )),
                    Spawn(icon_button().label("OK").build()),
                )),
            ))
            .build(),
        ChildOf(parent),
    ));
}

fn swap_theme(mut global: ResMut<GlobalTheme>, mut mac: ResMut<MacLook>) {
    mac.0 = !mac.0;
    *global = GlobalTheme(theme(mac.0));
}

fn open_window(mut commands: Commands, desktop: Res<Desktop>, mut opened: Local<u32>) {
    let offset = 24.0 * (*opened % 8) as f32;
    *opened += 1;
    spawn_window(
        &mut commands,
        desktop.0,
        None,
        WindowRect::new(80.0 + offset, 80.0 + offset, 240.0, 160.0),
        "New window",
    );
}

fn grip_over(over: On<PointerOver>, grips: Query<&WindowGrip>, mut state: ResMut<GripCursor>) {
    if let Ok(WindowGrip(edge)) = grips.get(over.entity) {
        state.hovered = Some(*edge);
    }
}

fn grip_out(out: On<PointerOut>, grips: Query<&WindowGrip>, mut state: ResMut<GripCursor>) {
    // Only the grip that is the hovered one lets go: the pointer may have entered another one already.
    if let Ok(WindowGrip(edge)) = grips.get(out.entity)
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
    if let Ok(WindowGrip(edge)) = grips.get(start.entity) {
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
