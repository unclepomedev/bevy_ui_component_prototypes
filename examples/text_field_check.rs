//! Sample for verifying the text field.
//!
//! Execution: cargo run --example text_field_check
//!
//! The field keeps its text itself. This example listens to its `ValueChange<String>` (the status line), sets its
//! text with `SetFieldText`, and toggles `InteractionDisabled` and the theme with the buttons (keys would be
//! typed into the field).
//!
//! What to verify:
//!   1. Each field is as high as one line of text with its padding. "Name" is empty: the placeholder
//!      shows inside it, at the left of where the text goes. Click it, then type: the placeholder goes,
//!      the text shows at the same place, and the cursor blinks after the last letter. Delete
//!      everything: the placeholder comes back.
//!   2. Clicking the field (the padding too) gives it the bright border and the bright background. A
//!      ring is drawn around it only after Tab, not after a click. Clicking outside the field takes the
//!      focus away.
//!   3. Tab / Shift+Tab move the focus between the fields in order. The disabled field is skipped, and
//!      the read-only one is not.
//!   4. Select with Shift and the arrow keys or with a drag: the selected text has a colored
//!      background. Click another field: the selection is still shown, in the dimmer color.
//!   5. Copy and paste work (Ctrl+C / Ctrl+V, Cmd on macOS), also between the fields.
//!   6. "At most 8" takes no more than 8 characters.
//!   7. "Read only" has text that can be selected and copied, and cannot be changed.
//!   8. "Disabled" is gray, takes no click (on the text too, with no flicker of the cursor), no selection and no key, and has no ring. The "Disable Name" button
//!      toggles it on "Name" too: if "Name" has the focus when it is disabled, it loses it.
//!   9. The pointer over a field gives it the hovered background.
//!  10. "Narrow" is 10 glyphs wide and does not follow the width of its parent. A longer text scrolls
//!      inside it and keeps the cursor in view.
//!  11. Input method (Japanese, for instance): the composing text shows in the field with its
//!      underline, the candidate list appears next to the cursor, and a committed text goes into the
//!      field.
//!  12. The field in the window: drag the window while a field has the focus. Typing still works, and
//!      the candidate list follows the cursor.
//!  13. "Swap theme" swaps the global theme (a bright, rounder field). The framed panel, which has its own
//!      `Theme`, does not change.
//!  14. The status line shows the text of "Name" after each change ("typing"), and "done" once the field has
//!      lost the focus. A cursor move or a selection changes nothing in it.
//!  15. "Fill Name" puts "Ferris" in "Name" (cursor at the end). The status line does not report it. A
//!      limited field would cut the text.
//!  16. "Notes" is a text area four lines high. Enter starts a new line; a long line wraps at the width of the
//!      field instead of scrolling sideways. With more than four lines the text scrolls vertically and keeps
//!      the cursor in view; Up / Down move between lines. The placeholder shows while it is empty.
//!  18. "Notes" has a scroll indicator at its right edge while the text is longer than the four lines: the thumb
//!      follows the visible part when you type, move the cursor or turn the mouse wheel over the field. It
//!      disappears when the text fits. A single-line field never has one.
//!  17. Copy and paste now go through the OS clipboard (no "in-process fallback buffer" warning): copy
//!      something in another app and paste it here, and the other way round.

use bevy::{
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{Activate, ValueChange},
};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, UiComponentPrototypesPlugin,
    button::icon_button,
    text_field::{SetFieldText, TextFieldShape, TextFieldStyle, text_field},
    window::{WindowDefaultsPlugin, WindowRect, window},
};

#[derive(Component)]
struct Name;

#[derive(Component)]
struct Status;

#[derive(Component, Clone, Copy)]
enum Action {
    Disable,
    Fill,
    Theme,
}

#[derive(Resource)]
struct Bright(bool);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            UiComponentPrototypesPlugin,
            TabNavigationPlugin,
            WindowDefaultsPlugin::default(),
        ))
        .insert_resource(GlobalTheme(Theme::default()))
        .insert_resource(Bright(false))
        .add_observer(on_button)
        .add_observer(on_value)
        .add_systems(Startup, setup)
        .run();
}

fn label(text: &'static str) -> impl Bundle {
    Text::new(text)
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let desktop = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            TabGroup::default(),
        ))
        .id();
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            top: px(24),
            width: px(320),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        ChildOf(desktop),
        Children::spawn((
            (
                Spawn(label("Name")),
                Spawn((
                    text_field().placeholder(Text::new("Your name")).build(),
                    Name,
                )),
                Spawn(label("At most 8")),
                Spawn(text_field().max_characters(8).build()),
                Spawn(label("Read only")),
                Spawn(
                    text_field()
                        .initial("Select me and copy")
                        .read_only(true)
                        .build(),
                ),
            ),
            (
                Spawn(label("Disabled")),
                Spawn((
                    text_field().initial("Not for you").build(),
                    InteractionDisabled,
                )),
                Spawn(label("Narrow")),
                Spawn(text_field().visible_width(10.0).build()),
                Spawn(label("Notes")),
                Spawn(
                    text_field()
                        .lines(4.0)
                        .initial("One\nTwo\nThree\nFour\nFive\nSix")
                        .placeholder(Text::new("Several lines"))
                        .build(),
                ),
                Spawn((icon_button().label("Disable Name").build(), Action::Disable)),
                Spawn((icon_button().label("Swap theme").build(), Action::Theme)),
                Spawn((icon_button().label("Fill Name").build(), Action::Fill)),
                Spawn((Text::new(""), Status)),
            ),
        )),
    ));
    // A panel with a theme of its own.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(384),
            top: px(24),
            width: px(260),
            padding: UiRect::all(px(8)),
            border: UiRect::all(px(1)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        BorderColor::all(Color::WHITE),
        Theme {
            text_field: TextFieldStyle {
                shape: TextFieldShape {
                    border_width: 2.0,
                    radius: 12.0,
                },
                ..default()
            },
            ..default()
        },
        ChildOf(desktop),
        Children::spawn((
            Spawn(label("Own theme")),
            Spawn(text_field().placeholder(Text::new("rounder")).build()),
        )),
    ));
    commands.spawn((
        window()
            .rect(WindowRect::new(384.0, 200.0, 320.0, 160.0))
            .title(Text::new("Answer"))
            .content_bundle((
                Node {
                    padding: UiRect::all(px(12)),
                    row_gap: px(8),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Children::spawn((
                    Spawn(label("Who did it?")),
                    Spawn(text_field().placeholder(Text::new("Type a name")).build()),
                )),
            ))
            .build(),
        ChildOf(desktop),
    ));
}

fn on_button(
    activate: On<Activate>,
    actions: Query<&Action>,
    names: Query<(Entity, Has<InteractionDisabled>), With<Name>>,
    mut global: ResMut<GlobalTheme>,
    mut bright: ResMut<Bright>,
    mut commands: Commands,
) {
    let Ok(action) = actions.get(activate.entity) else {
        return;
    };
    if matches!(action, Action::Disable) {
        for (name, disabled) in &names {
            if disabled {
                commands.entity(name).remove::<InteractionDisabled>();
            } else {
                commands.entity(name).insert(InteractionDisabled);
            }
        }
    } else if matches!(action, Action::Fill) {
        for (name, _) in &names {
            commands.trigger(SetFieldText {
                field: name,
                text: "Ferris".into(),
            });
        }
    } else {
        bright.0 = !bright.0;
        let mut next = Theme::default();
        if bright.0 {
            let light = Color::srgb(0.9, 0.9, 0.95);
            let bg = &mut next.text_field.bg;
            (bg.normal, bg.hovered, bg.pressed) = (light, light, light);
            bg.focused = Color::WHITE;
            let text = &mut next.text_field.text;
            (text.normal, text.hovered, text.pressed, text.focused) =
                (Color::BLACK, Color::BLACK, Color::BLACK, Color::BLACK);
            next.text_field.cursor = Color::BLACK;
            next.text_field.shape.radius = 10.0;
        }
        *global = GlobalTheme(next);
    }
}

fn on_value(
    change: On<ValueChange<String>>,
    names: Query<(), With<Name>>,
    mut status: Single<&mut Text, With<Status>>,
) {
    if names.contains(change.source) {
        let phase = if change.is_final { "done" } else { "typing" };
        status.0 = format!("Name ({phase}): \"{}\"", change.value);
    }
}
