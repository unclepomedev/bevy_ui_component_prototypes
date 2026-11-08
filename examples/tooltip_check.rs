//! Sample for verifying functionality.
//!
//! Execution: cargo run --example tooltip_check
//!
//! What to verify:
//!   1. Rest the pointer on a button: after a short wait, a tooltip appears on the side it asks for
//!      ("Top", "Bottom", "Right"). Moving the pointer away hides it at once.
//!   2. Move along the row of buttons: after the first tooltip, the next ones appear without waiting.
//!   3. The button in the top-left corner asks for a tooltip above it. There is no room above, so it opens
//!      below, and since the text is wider than the button it lines up with the button's left edge
//!      instead of being cut off by the window.
//!   4. "Disabled" is grayed out and cannot be pressed, and its tooltip still explains why.
//!   5. Click a button while its tooltip is showing: the tooltip hides and stays away until the pointer leaves.
//!      Escape hides it as well.
//!   6. Tab moves the keyboard focus along the buttons: the tooltip of the focused one appears next to its focus ring.
//!      After a click the ring is hidden and so is the tooltip of the focus.
//!   7. [T] swaps the global theme: the tooltips change their colors and wait for a different time.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::popover::PopoverSide,
};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, UiComponentPrototypesPlugin,
    button::icon_button,
    tooltip::{Tooltip, TooltipMetrics, TooltipStyle},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TabNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .init_resource::<GlobalTheme>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            swap_global_theme.run_if(input_just_pressed(KeyCode::KeyT)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            TabGroup::default(),
        ))
        .with_children(|root| {
            root.spawn(Node {
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    icon_button().label("Top").build(),
                    Tooltip::new("Above the button").side(PopoverSide::Top),
                ));
                row.spawn((
                    icon_button().label("Bottom").build(),
                    Tooltip::new("Below the button"),
                ));
                row.spawn((
                    icon_button().label("Right").build(),
                    Tooltip::new(
                        "Beside the button, with a text long enough to wrap onto a second line",
                    )
                    .side(PopoverSide::Right),
                ));
                row.spawn((
                    icon_button().label("Instant").build(),
                    Tooltip::new("No wait").delay(0.0),
                ));
            });
            root.spawn((
                icon_button().label("Disabled").build(),
                InteractionDisabled,
                Tooltip::new("Why: nothing to save yet"),
            ));
            root.spawn(Text::new(
                "Rest on a button / Tab through them / [T] swap theme",
            ));
        });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(8.0),
            top: Val::Px(8.0),
            ..default()
        },
        children![(
            icon_button().label("Corner").build(),
            Tooltip::new("Asks for the top, opens below").side(PopoverSide::Top),
        )],
    ));
}

/// Toggles between the default theme and a bright, slow one.
fn swap_global_theme(mut theme: ResMut<GlobalTheme>, mut bright: Local<bool>) {
    *bright = !*bright;
    *theme = GlobalTheme(if *bright {
        Theme {
            tooltip: TooltipStyle {
                bg: Color::srgb(1.0, 0.9, 0.4),
                fg: Color::BLACK,
                ..default()
            },
            tooltip_metrics: TooltipMetrics {
                delay: 1.0,
                ..default()
            },
            ..default()
        }
    } else {
        Theme::default()
    });
}
