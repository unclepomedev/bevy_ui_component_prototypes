//! Behavior tests for the toast: structure and the pointer.
//!
//! Colors, sizes and the theme come with the style sync; the slide and the real screen are covered by
//! `examples/toast_check.rs`.

use crate::probe::probe_theme;
use bevy::{picking::Pickable, prelude::*};
use bevy_ui_component_prototypes::{
    GlobalTheme, Toast, ToastVariant, ToastVariantId, UiComponentPrototypesPlugin, toast,
};

#[test]
fn a_toast_carries_its_structure_and_starts_unseen() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");

    let world = app.world();
    assert_eq!(
        world.get::<Toast>(toast),
        Some(&Toast {
            elapsed: 0.0,
            duration: None
        })
    );
    assert_eq!(
        world.get::<ToastVariant>(toast),
        Some(&ToastVariant::Default)
    );
    assert!(world.get::<UiTransform>(toast).is_some());
    assert_eq!(world.get::<Children>(toast).map(|c| c.len()), Some(1));
    assert_eq!(text(&app, toast), "Saved");
    // A default `TextColor` is white: the text must not show before the first style sync.
    assert_eq!(label_color(&app, toast), Color::NONE);
    assert_eq!(
        world.get::<BackgroundColor>(toast).map(|c| c.0),
        Some(Color::NONE)
    );
}

#[test]
fn the_builder_options_reach_the_components() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(
            toast("Done")
                .variant(ToastVariantId::SUCCESS)
                .duration(2.5)
                .build(),
        )
        .id();

    let world = app.world();
    assert_eq!(world.get::<Toast>(toast).unwrap().duration, Some(2.5));
    assert_eq!(
        world.get::<ToastVariant>(toast),
        Some(&ToastVariant::Named(ToastVariantId::SUCCESS))
    );
}

#[test]
fn the_text_keeps_its_line_breaks_and_wraps() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "first\nsecond");

    assert_eq!(text(&app, toast), "first\nsecond");
    // Unlike a button label, a toast wraps at the width the theme gives it.
    assert_eq!(label_layout(&app, toast).linebreak, LineBreak::WordBoundary);
}

#[test]
fn a_toast_and_its_text_let_the_pointer_through() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");

    let world = app.world();
    assert!(passes_through(world.get::<Pickable>(toast).unwrap()));
    assert!(passes_through(
        world.get::<Pickable>(label(&app, toast)).unwrap()
    ));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app.insert_resource(GlobalTheme(probe_theme()));
    app
}

fn spawn_toast(app: &mut App, text: &str) -> Entity {
    app.world_mut().spawn(toast(text).build()).id()
}

fn passes_through(pickable: &Pickable) -> bool {
    !pickable.should_block_lower && !pickable.is_hoverable
}

fn label(app: &App, toast: Entity) -> Entity {
    app.world().get::<Children>(toast).unwrap()[0]
}

fn text(app: &App, toast: Entity) -> String {
    app.world()
        .get::<Text>(label(app, toast))
        .unwrap()
        .0
        .clone()
}

fn label_color(app: &App, toast: Entity) -> Color {
    app.world().get::<TextColor>(label(app, toast)).unwrap().0
}

fn label_layout(app: &App, toast: Entity) -> TextLayout {
    *app.world().get::<TextLayout>(label(app, toast)).unwrap()
}
