//! Helpers that the test files share. Only what is the same in all of them lives here: what belongs to a widget
//! (its parts, how to spawn it) stays in the file of the widget.

use crate::probe::probe_theme;
use bevy::{
    input_focus::{InputFocus, InputFocusVisible, tab_navigation::TabIndex},
    picking::Pickable,
    prelude::*,
};
use bevy_ui_component_prototypes::{GlobalTheme, UiComponentPrototypesPlugin};

/// An app with the plugin of the crate, and the probe theme as the global theme.
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app.insert_resource(GlobalTheme(probe_theme()));
    app
}

/// A red that no theme has by default: a test that sees it knows the value came from the test.
pub fn red() -> Color {
    Color::srgb(0.9, 0.1, 0.1)
}

pub fn bg(app: &App, entity: Entity) -> Color {
    app.world().get::<BackgroundColor>(entity).unwrap().0
}

pub fn outline(app: &App, entity: Entity) -> Outline {
    *app.world().get::<Outline>(entity).unwrap()
}

pub fn border_color(app: &App, entity: Entity) -> BorderColor {
    *app.world().get::<BorderColor>(entity).unwrap()
}

pub fn node(app: &App, entity: Entity) -> Node {
    app.world().get::<Node>(entity).unwrap().clone()
}

/// Gives the focus to `entity`, with the focus indicator shown or hidden.
pub fn focus(app: &mut App, entity: Entity, visible: bool) {
    app.insert_resource(InputFocus::from_entity(entity));
    app.insert_resource(InputFocusVisible(visible));
}

pub fn tab_index(app: &App, entity: Entity) -> i32 {
    app.world().get::<TabIndex>(entity).unwrap().0
}

/// Whether the pointer goes through `pickable` to what is behind it.
pub fn passes_through(pickable: &Pickable) -> bool {
    !pickable.should_block_lower && !pickable.is_hoverable
}
