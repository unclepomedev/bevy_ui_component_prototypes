//! Sample for verifying the look of the window.
//!
//! Execution: cargo run --example window_check
//!
//! A window comes to the front when pressed. It does not move, close or resize yet.
//!
//! What to verify:
//!   1. The window on the left has the mac look: a circular red close button at the left of the title bar
//!      and a centered title. The cross shows only while the pointer is over the button.
//!   2. The window on the right has the Windows look: a square close button at the right end of the title
//!      bar, flush with its top and bottom, and a left-aligned title. It turns red under the pointer.
//!   3. Press on a close button: it gets darker. Nothing closes.
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

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, UiComponentPrototypesPlugin, WindowDefaultsPlugin, WindowMetrics,
    WindowRect, WindowStyle, icon_button, window,
};

#[derive(Resource)]
struct MacLook(bool);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            UiComponentPrototypesPlugin,
            WindowDefaultsPlugin::default(),
        ))
        .insert_resource(GlobalTheme(theme(true)))
        .insert_resource(MacLook(true))
        .add_systems(Startup, setup)
        .add_systems(Update, swap_theme.run_if(input_just_pressed(KeyCode::KeyT)))
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
            "[T] swap the GLOBAL theme; the windows with a Theme of their own are unaffected",
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
