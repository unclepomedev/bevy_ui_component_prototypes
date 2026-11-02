//! Helpers that the test files share. Only what is the same in all of them lives here: what belongs to a widget
//! (its parts, how to spawn it) stays in the file of the widget.

use crate::probe::probe_theme;
use bevy::prelude::*;
use bevy_ui_component_prototypes::{GlobalTheme, UiComponentPrototypesPlugin};

/// An app with the plugin of the crate, and the probe theme as the global theme.
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app.insert_resource(GlobalTheme(probe_theme()));
    app
}
