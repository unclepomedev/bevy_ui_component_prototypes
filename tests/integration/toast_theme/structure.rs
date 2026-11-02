//! Tests of Structure of a toast and the pointer.

use super::{label, label_color, settle, spawn_toast, text};
use crate::{
    probe::probe_theme,
    support::{passes_through, test_app},
};
use bevy::{picking::Pickable, prelude::*};
use bevy_ui_component_prototypes::{Toast, ToastVariant, ToastVariantId, toast};

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

#[derive(Component)]
struct Marker(u8);

#[derive(Component)]
struct Other;

#[test]
fn label_bundles_land_on_the_text_only_and_accumulate() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(
            toast()
                .text("Saved")
                .label_bundle(Marker(7))
                .label_bundle(Other)
                .build(),
        )
        .id();

    let world = app.world();
    let label = label(&app, toast);
    assert_eq!(world.get::<Marker>(label).map(|m| m.0), Some(7));
    assert!(world.get::<Other>(label).is_some());
    // Neither the toast itself gets them.
    assert!(world.get::<Marker>(toast).is_none());
    assert_eq!(text(&app, toast), "Saved");
}

#[test]
fn the_other_options_still_work_after_a_label_bundle() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(
            toast()
                .label_bundle(Marker(1))
                .text("Hi")
                .variant(ToastVariantId::SUCCESS)
                .duration(2.5)
                .build(),
        )
        .id();

    let world = app.world();
    assert_eq!(text(&app, toast), "Hi");
    assert_eq!(world.get::<Toast>(toast).unwrap().duration, Some(2.5));
    assert_eq!(
        world.get::<ToastVariant>(toast),
        Some(&ToastVariant::Named(ToastVariantId::SUCCESS))
    );
}

#[test]
fn a_text_with_a_label_bundle_is_still_styled() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(toast().text("Saved").label_bundle(Marker(1)).build())
        .id();
    settle(&mut app, toast);

    app.update();

    assert_eq!(label_color(&app, toast), probe_theme().toasts.base.fg);
}

#[test]
fn the_builder_options_reach_the_components() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(
            toast()
                .text("Done")
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

fn label_layout(app: &App, toast: Entity) -> TextLayout {
    *app.world().get::<TextLayout>(label(app, toast)).unwrap()
}
