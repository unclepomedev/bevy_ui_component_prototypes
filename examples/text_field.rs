//! Text fields in a small sign-up form.
//!
//! Execution: cargo run --example text_field
//!
//! Shows how an app uses `text_field()`:
//! - The field keeps its own text. The app hears about it as `ValueChange<String>` (here: `on_value` copies it
//!   into the `Form`), and replaces it with the `SetFieldText` event ("Clear").
//! - "Send" locks the form with `InteractionDisabled`, "Edit" unlocks it. A disabled field takes no focus,
//!   no selection and no copy; "Member ID" is `read_only` instead, which can be selected and copied.
//! - "About you" is a text area (`lines`): Enter starts a new line, and a scroll indicator shows up when the
//!   text is longer than the lines shown.
//! - The "Feedback" window has a text field in its content.
//!
//! Colors in each state, the focus ring, Tab order, the placeholder, the limits and the layout are covered by
//! the tests. Only these need a person, with the real keyboard, window and clipboard:
//! - Input method (Japanese, for instance): the composing text is underlined in the field and the candidate
//!   list opens next to the cursor, also in the "Feedback" window while it is dragged.
//! - Copy and paste (Ctrl+C / Ctrl+V, Cmd on macOS) go through the clipboard of the OS, also from and to other
//!   applications.
//! - The mouse wheel over "About you" scrolls its lines, and hands over to the page at the ends.

use bevy::{
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{Activate, ValueChange},
};
use bevy_ui_component_prototypes::{
    SetFieldText, UiComponentPrototypesPlugin, WindowDefaultsPlugin, WindowRect, icon_button,
    text_field, window,
};

/// Which field of the form a `TextField` is.
#[derive(Component, Clone, Copy)]
enum Field {
    Name,
    Email,
    About,
}

/// What the app knows about the form: the last texts the fields told it.
#[derive(Resource, Default)]
struct Form {
    name: String,
    email: String,
    about: String,
}

#[derive(Component, Clone, Copy)]
enum Action {
    Send,
    Edit,
    Clear,
}

#[derive(Component)]
struct Summary;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            UiComponentPrototypesPlugin,
            TabNavigationPlugin,
            WindowDefaultsPlugin::default(),
        ))
        .init_resource::<Form>()
        .add_observer(on_value)
        .add_observer(on_button)
        .add_systems(Startup, setup)
        .run();
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
            // Tab and Shift+Tab move the focus between the fields and the buttons under it.
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
                Spawn(Text::new("Name")),
                Spawn((
                    text_field()
                        .placeholder(Text::new("Your name"))
                        .max_characters(40)
                        .build(),
                    Field::Name,
                )),
                Spawn(Text::new("Email")),
                Spawn((
                    text_field()
                        .placeholder(Text::new("you@example.com"))
                        .build(),
                    Field::Email,
                )),
                Spawn(Text::new("Member ID")),
                Spawn(text_field().initial("A-1024").read_only(true).build()),
                Spawn(Text::new("About you")),
                Spawn((
                    text_field()
                        .lines(4.0)
                        .placeholder(Text::new("A few words"))
                        .build(),
                    Field::About,
                )),
            ),
            (
                Spawn((
                    Node {
                        column_gap: px(8),
                        ..default()
                    },
                    Children::spawn((
                        Spawn((icon_button().label("Send").build(), Action::Send)),
                        Spawn((icon_button().label("Edit").build(), Action::Edit)),
                        Spawn((icon_button().label("Clear").build(), Action::Clear)),
                    )),
                )),
                Spawn((Text::new(""), Summary)),
            ),
        )),
    ));

    commands.spawn((
        window()
            .rect(WindowRect::new(384.0, 24.0, 320.0, 160.0))
            .title(Text::new("Feedback"))
            .content_bundle((
                Node {
                    padding: UiRect::all(px(12)),
                    row_gap: px(8),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Children::spawn((
                    Spawn(Text::new("What would you change?")),
                    Spawn(text_field().lines(3.0).build()),
                )),
            ))
            .build(),
        ChildOf(desktop),
    ));
}

/// A field tells the app its text while it changes (`is_final` is `false`), and again when it loses the focus
/// (`is_final` is `true`). `source` is the `TextField`.
fn on_value(change: On<ValueChange<String>>, fields: Query<&Field>, mut form: ResMut<Form>) {
    let Ok(field) = fields.get(change.source) else {
        return;
    };
    let text = change.value.clone();
    match field {
        Field::Name => form.name = text,
        Field::Email => form.email = text,
        Field::About => form.about = text,
    }
}

fn on_button(
    activate: On<Activate>,
    actions: Query<&Action>,
    fields: Query<Entity, With<Field>>,
    mut form: ResMut<Form>,
    mut summary: Single<&mut Text, With<Summary>>,
    mut commands: Commands,
) {
    let Ok(action) = actions.get(activate.entity) else {
        return;
    };
    match action {
        Action::Send => {
            summary.0 = format!(
                "Sent {} <{}>, {} characters about them",
                form.name,
                form.email,
                form.about.chars().count()
            );
            for field in &fields {
                commands.entity(field).insert(InteractionDisabled);
            }
        }
        Action::Edit => {
            for field in &fields {
                commands.entity(field).remove::<InteractionDisabled>();
            }
        }
        Action::Clear => {
            // Replacing the text is not reported back, so the app resets what it keeps.
            *form = Form::default();
            summary.0.clear();
            for field in &fields {
                commands.trigger(SetFieldText {
                    field,
                    text: String::new(),
                });
            }
        }
    }
}
