//! Sample for verifying the select.
//!
//! Execution: cargo run --example select_check
//!
//! The select is controlled: it shows what `SelectedOption` says. The keys of this example and the choices made in
//! the list both change it, and the choices go through `on_choice`, where the app decides what to do with them.
//!
//! What to verify:
//!   1. "Fruit" starts on its placeholder. [1] [2] [3] choose Apple / Banana / Cherry: the label changes and
//!      is colored by the theme. [0] removes the value and the placeholder returns.
//!   2. "Preselected" starts on Banana (`.selected(...)`). "From an iterator" shows its options given by `.options(...)`
//!      (press [1] [2] [3]: the keys also drive it).
//!   3. Hover brightens the box, and press darkens it; the label and the chevron change with it.
//!   4. Tab / Shift+Tab move the focus between the selects, and a ring appears around the focused one.
//!      Clicking hides the ring again.
//!   5. [T] swaps the global theme (the box turns orange). The framed panel, which has its own `Theme`, does not change.
//!   6. Click the box: the list opens below it (or above, when there is no room), and it is in front of the rows below.
//!      Escape closes it, and clicking outside closes it too.
//!   7. Click an option, or move with the arrow keys and press Enter: the label changes, the list closes and the
//!      focus goes back to the box. The ring is there after the keys, and not after a click.
//!   8. Choose the option that is already chosen: the list closes and nothing changes.
//!   9. Move the pointer over the rows of an open list: the highlight follows it, one row at a time. The arrow keys
//!      move the same highlight, and Enter chooses the highlighted row.
//!  10. "Many options" has 20 options and the list shows 8 of them: the wheel scrolls it, and the arrow keys,
//!      Home and End bring the highlighted row into view. A known limitation: the list opens at the scroll
//!      position it was left at, not at the chosen option.
//!  11. "Disabled" is gray, does not react to the pointer or to Enter, and Tab skips it.
//!  12. "Some disabled" has Banana disabled: its row is gray and has no highlight, and clicking it or pressing
//!      Enter on it does nothing: the list stays open. The arrow keys still stop on it.
//!
//! The open lists are drawn in front of the rows below because the theme says where they are in the stacking
//! order: see `theme_for`.

use bevy::{
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::ValueChange,
};
use bevy_ui_component_prototypes::{
    GlobalTheme, SelectBuilder, SelectKey, SelectedOption, Theme, UiComponentPrototypesPlugin,
    select,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

impl Fruit {
    const ALL: [Fruit; 3] = [Fruit::Apple, Fruit::Banana, Fruit::Cherry];

    fn name(self) -> &'static str {
        match self {
            Fruit::Apple => "Apple",
            Fruit::Banana => "Banana",
            Fruit::Cherry => "Cherry",
        }
    }
}

/// The selects the keys drive.
#[derive(Component)]
struct Driven;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TabNavigationPlugin,
            UiComponentPrototypesPlugin,
        ))
        .insert_resource(GlobalTheme(theme_for(false)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                choose,
                swap_global_theme.run_if(input_just_pressed(KeyCode::KeyT)),
            ),
        )
        .add_observer(on_choice::<Fruit>)
        .add_observer(on_choice::<u32>)
        .run();
}

fn label(text: &'static str) -> Text {
    Text::new(text)
}

fn fruit_options() -> [(Fruit, Text); 3] {
    Fruit::ALL.map(|fruit| (fruit, label(fruit.name())))
}

/// The select that most rows start from. The rows add what they show of the API to it.
fn fruits() -> SelectBuilder<Fruit> {
    select()
        .placeholder(label("Pick a fruit"))
        .options(fruit_options())
}

/// The options come from an iterator, which `.options(...)` takes as it is.
fn from_an_iterator() -> impl Bundle {
    select::<Fruit>()
        .placeholder(label("Nothing yet"))
        .options(
            Fruit::ALL
                .into_iter()
                .map(|fruit| (fruit, label(fruit.name()))),
        )
        .build()
}

/// More options than the list shows: it scrolls.
fn many_options() -> impl Bundle {
    select::<u32>()
        .placeholder(label("Pick a number"))
        .options((1..=20).map(|n| (n, Text::new(format!("Option {n}")))))
        .build()
}

/// An option can be disabled one by one with `.option_disabled(...)`, or all at once like here.
fn some_disabled() -> impl Bundle {
    select::<Fruit>()
        .placeholder(label("Pick a fruit"))
        .options_with_disabled(
            Fruit::ALL.map(|fruit| (fruit, label(fruit.name()), fruit == Fruit::Banana)),
        )
        .build()
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

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
            spawn_row(root, "Fruit", (fruits().build(), Driven));
            spawn_row(
                root,
                "Preselected",
                fruits().selected(Fruit::Banana).build(),
            );
            spawn_row(root, "From an iterator", (from_an_iterator(), Driven));
            spawn_row(root, "Many options", many_options());
            spawn_row(
                root,
                "Disabled",
                (
                    fruits().selected(Fruit::Cherry).build(),
                    InteractionDisabled,
                ),
            );
            spawn_row(root, "Some disabled", some_disabled());
            spawn_scoped_panel(root);
            root.spawn((
                Text::new(
                    "[1] [2] [3] choose   [0] clear   [Tab] move focus\n\
                     [T] swap GLOBAL theme; the framed panel is unaffected",
                ),
                Node {
                    margin: UiRect::top(px(24)),
                    ..default()
                },
            ));
        });
}

/// The label is a sibling `Text`; the select is the box. The select takes the full width of the box it is put in.
fn spawn_row(root: &mut ChildSpawnerCommands, name: &'static str, widget: impl Bundle) {
    root.spawn(Node {
        column_gap: px(16),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new(name),
            Node {
                width: px(200),
                ..default()
            },
        ));
        row.spawn(Node {
            width: px(320),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn((widget, AccessibleLabel::new(name)));
        });
    });
}

/// Framed panel with its own `Theme` (unaffected by the global theme).
fn spawn_scoped_panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(2)),
            ..default()
        },
        BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
        Theme::default(),
    ))
    .with_children(|panel| {
        panel.spawn(Node::default()).with_children(|row| {
            row.spawn((
                Text::new("Scoped (own Theme)"),
                Node {
                    width: px(176),
                    ..default()
                },
            ));
            row.spawn(Node {
                width: px(320),
                ..default()
            })
            .with_children(|slot| {
                slot.spawn(fruits().selected(Fruit::Apple).build());
            });
        });
    });
}

/// The app's policy: apply what the select asks for. One observer for each type of option values.
fn on_choice<V: SelectKey>(change: On<ValueChange<V>>, mut commands: Commands) {
    commands.entity(change.source).insert(SelectedOption {
        value: change.value.clone(),
    });
}

fn choose(
    keys: Res<ButtonInput<KeyCode>>,
    driven: Query<Entity, With<Driven>>,
    mut commands: Commands,
) {
    let chosen = [
        (KeyCode::Digit1, Fruit::Apple),
        (KeyCode::Digit2, Fruit::Banana),
        (KeyCode::Digit3, Fruit::Cherry),
    ]
    .into_iter()
    .find(|(key, _)| keys.just_pressed(*key))
    .map(|(_, fruit)| fruit);

    for entity in &driven {
        if let Some(fruit) = chosen {
            commands
                .entity(entity)
                .insert(SelectedOption { value: fruit });
        } else if keys.just_pressed(KeyCode::Digit0) {
            commands.entity(entity).remove::<SelectedOption<Fruit>>();
        }
    }
}

fn swap_global_theme(mut global: ResMut<GlobalTheme>, mut swapped: Local<bool>) {
    *swapped = !*swapped;
    *global = GlobalTheme(theme_for(*swapped));
}

/// The global theme of this example. When `swapped`, the select box is orange, so that the change is easy to see.
fn theme_for(swapped: bool) -> Theme {
    let mut theme = Theme::default();
    // Where the open lists are in the stacking order (`GlobalZIndex`). They have to be in front of the rows that
    // come after their select; the default does that already, and this is how to change it.
    theme.select_metrics.popup_z_index = 10;
    if swapped {
        let bg = &mut theme.select.trigger.bg;
        bg.normal = Color::srgb(0.85, 0.45, 0.10);
        bg.hovered = Color::srgb(0.95, 0.58, 0.20);
        bg.pressed = Color::srgb(0.65, 0.33, 0.07);
        bg.focused = Color::srgb(0.95, 0.58, 0.20);
    }
    theme
}
