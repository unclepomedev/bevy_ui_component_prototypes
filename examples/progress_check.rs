//! Sample for verifying the progress bar.
//!
//! Execution: cargo run --example progress_check
//!
//! The bar is controlled and has no input: this example changes `ProgressValue`, and the bar follows.
//! The same bar serves as a progress bar and as an HP bar: the app watches the value and swaps the variant.
//!
//! What to verify:
//!   1. "Auto" fills from empty to full in four seconds and starts over. The fill ends exactly at the value.
//!   2. "Manual": [Up] / [Down] move the value by 10%. Holding either end does not overflow the rail.
//!   3. "HP" (0..=100): [D] takes 10 off, [H] gives 10 back. At 30% and below the bar turns red
//!      (this example swaps the variant), and above it, it is green again. The text on the left shows the number.
//!   4. "Blocks" is a ten-block bar from its panel's own `Theme` (square, thicker). It follows "Auto",
//!      and a block lights once the value has reached its end.
//!   5. [T] swaps the global theme (the default variant gets an orange fill and a dark rail).
//!      "Auto" and "Manual" change. "HP" (named variants) and the framed panel (its own `Theme`) do not.

use bevy::{ecs::query::QueryData, input::common_conditions::input_just_pressed, prelude::*};
use bevy_ui_component_prototypes::{
    BarSegments, GlobalTheme, ProgressRange, ProgressShape, ProgressValue, ProgressVariant,
    ProgressVariantId, Theme, UiComponentPrototypesPlugin, progress_bar,
};

const HP_MAX: f32 = 100.0;
const HP_STEP: f32 = 10.0;
/// Below this share of the maximum, the HP bar turns red.
const HP_LOW: f32 = 0.3;
const MANUAL_STEP: f32 = 0.1;
const AUTO_SECONDS: f32 = 4.0;

/// Follows the clock.
#[derive(Component)]
struct Auto;

/// Moved with [Up] / [Down].
#[derive(Component)]
struct Manual;

#[derive(Component)]
struct Hp;

#[derive(Component)]
struct HpText;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UiComponentPrototypesPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                advance_auto,
                move_manual,
                change_hp,
                update_hp_variant,
                update_hp_text,
                swap_global_theme.run_if(input_just_pressed(KeyCode::KeyT)),
            ),
        )
        .run();
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(20),
            ..default()
        })
        .with_children(|root| {
            spawn_row(root, "Auto", progress_bar().build(), Auto);
            spawn_row(root, "Manual", progress_bar().value(0.5).build(), Manual);
            spawn_hp_row(root);
            spawn_blocks_panel(root);
            spawn_help(root);
        });
}

/// The label is a sibling `Text`: the bar is the rail and the fill only. Screen readers do not pair
/// the two, so the bar gets an `AccessibleLabel` of its own. The bar takes the full width of the box it is put in.
fn spawn_row(
    root: &mut ChildSpawnerCommands,
    label: &'static str,
    bar: impl Bundle,
    extra: impl Bundle,
) {
    root.spawn(Node {
        column_gap: px(16),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new(label),
            Node {
                width: px(160),
                ..default()
            },
        ));
        row.spawn(Node {
            width: px(320),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn((bar, AccessibleLabel::new(label), extra));
        });
    });
}

fn spawn_hp_row(root: &mut ChildSpawnerCommands) {
    let bar = progress_bar()
        .range(0.0, HP_MAX)
        .value(HP_MAX)
        .variant(ProgressVariantId::AFFIRMATIVE)
        .build();
    root.spawn(Node {
        column_gap: px(16),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new("HP 100 / 100"),
            Node {
                width: px(160),
                ..default()
            },
            HpText,
        ));
        row.spawn(Node {
            width: px(320),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn((bar, AccessibleLabel::new("HP"), Hp));
        });
    });
}

/// Framed panel whose own `Theme` gives its bar another shape: ten square blocks.
fn spawn_blocks_panel(root: &mut ChildSpawnerCommands) {
    let mut theme = Theme::default();
    theme.progress.base.shape = ProgressShape {
        height: 18.0,
        rounded: false,
        segments: Some(BarSegments {
            count: 10,
            gap: 0.2,
        }),
    };
    root.spawn((
        Node {
            column_gap: px(16),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(2)),
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
        theme,
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Blocks (own Theme)"),
            Node {
                width: px(136),
                ..default()
            },
        ));
        panel
            .spawn(Node {
                width: px(320),
                ..default()
            })
            .with_children(|slot| {
                slot.spawn((progress_bar().build(), AccessibleLabel::new("Blocks"), Auto));
            });
    });
}

fn spawn_help(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Text::new(
            "[Up] / [Down] move Manual   [D] damage HP   [H] heal HP\n\
             [T] swap GLOBAL theme; HP and the framed panel are unaffected",
        ),
        Node {
            margin: UiRect::top(px(24)),
            ..default()
        },
    ));
}

// ---------------------------------------------------------------------------
// Behavior
// ---------------------------------------------------------------------------

fn advance_auto(time: Res<Time>, mut bars: Query<&mut ProgressValue, With<Auto>>) {
    let value = (time.elapsed_secs() % AUTO_SECONDS) / AUTO_SECONDS;
    for mut bar in &mut bars {
        **bar = value;
    }
}

fn move_manual(keys: Res<ButtonInput<KeyCode>>, mut bars: Query<&mut ProgressValue, With<Manual>>) {
    let step = if keys.just_pressed(KeyCode::ArrowUp) {
        MANUAL_STEP
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        -MANUAL_STEP
    } else {
        return;
    };
    for mut bar in &mut bars {
        **bar = (**bar + step).clamp(0.0, 1.0);
    }
}

fn change_hp(keys: Res<ButtonInput<KeyCode>>, mut bars: Query<&mut ProgressValue, With<Hp>>) {
    let step = if keys.just_pressed(KeyCode::KeyD) {
        -HP_STEP
    } else if keys.just_pressed(KeyCode::KeyH) {
        HP_STEP
    } else {
        return;
    };
    for mut bar in &mut bars {
        **bar = (**bar + step).clamp(0.0, HP_MAX);
    }
}

#[derive(QueryData)]
struct HpBar {
    entity: Entity,
    value: &'static ProgressValue,
    range: &'static ProgressRange,
    variant: &'static ProgressVariant,
}

/// The app's policy: the bar does not know what "low" means, so the variant is swapped here.
fn update_hp_variant(
    mut commands: Commands,
    bars: Query<HpBar, (With<Hp>, Changed<ProgressValue>)>,
) {
    for bar in &bars {
        let id = if bar.range.fraction(**bar.value) < HP_LOW {
            ProgressVariantId::DANGER
        } else {
            ProgressVariantId::AFFIRMATIVE
        };
        let want = ProgressVariant::from(id);
        if *bar.variant != want {
            commands.entity(bar.entity).insert(want);
        }
    }
}

fn update_hp_text(
    bars: Query<&ProgressValue, (With<Hp>, Changed<ProgressValue>)>,
    mut text: Query<&mut Text, With<HpText>>,
) {
    for value in &bars {
        for mut text in &mut text {
            text.0 = format!("HP {:.0} / {HP_MAX:.0}", **value);
        }
    }
}

fn swap_global_theme(mut global: ResMut<GlobalTheme>, mut swapped: Local<bool>) {
    *swapped = !*swapped;
    *global = GlobalTheme(theme_for(*swapped));
}

/// When `swapped`, returns a theme whose default variant is orange on a dark rail, so that the change is easy to see.
fn theme_for(swapped: bool) -> Theme {
    let mut theme = Theme::default();
    if swapped {
        theme.progress.base.fill = Color::srgb(0.85, 0.45, 0.10);
        theme.progress.base.rail = Color::srgb(0.12, 0.10, 0.08);
    }
    theme
}
