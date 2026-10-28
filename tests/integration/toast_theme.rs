//! Behavior tests for the toast: structure and the pointer.
//!
//! Colors, sizes and the theme come with the style sync; the slide and the real screen are covered by
//! `examples/toast_check.rs`.

use crate::probe::probe_theme;
use bevy::{picking::Pickable, prelude::*};
use bevy_ui_component_prototypes::{
    GlobalTheme, Toast, ToastHost, ToastStack, ToastVariant, ToastVariantId,
    UiComponentPrototypesPlugin, toast, toast_host,
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
// The host
// ---------------------------------------------------------------------------

#[test]
fn a_host_has_the_defaults_and_lets_the_pointer_through() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();

    let world = app.world();
    assert_eq!(
        world.get::<ToastHost>(host),
        Some(&ToastHost {
            max: 5,
            stack: ToastStack::NewestAtBottom,
            slide: Val2::ZERO,
        })
    );
    assert!(world.get::<Node>(host).is_some());
    assert!(passes_through(world.get::<Pickable>(host).unwrap()));
}

#[test]
fn the_host_options_reach_the_component() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn(
            toast_host()
                .max(2)
                .stack(ToastStack::NewestAtTop)
                .slide(Val2::percent(100, 0))
                .build(),
        )
        .id();

    assert_eq!(
        app.world().get::<ToastHost>(host),
        Some(&ToastHost {
            max: 2,
            stack: ToastStack::NewestAtTop,
            slide: Val2::percent(100, 0),
        })
    );
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
