//! Tests of theme resolution.

use super::{label_font, primary, spawn_primary, spawn_primary_under};
use crate::{
    probe::probe_theme,
    support::{bg, test_app},
};
use bevy::{
    asset::uuid_handle,
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{InteractionDisabled, VisualBox, widget::NodeImageMode},
};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme,
    button::{ButtonVariantId, SlicedImage},
};

// [T] swap the global theme
#[test]
fn global_theme_change_updates_unscoped_buttons_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_primary(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_primary_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(swapped_theme()));
    app.update();

    let swapped = swapped_theme();
    assert_eq!(bg(&app, unscoped), primary(&swapped).bg.normal);
    assert_eq!(label_font(&app, unscoped), swapped.font);
    assert_eq!(bg(&app, scoped), primary(&probe_theme()).bg.normal);
}

#[test]
fn theme_font_smoothing_is_applied_to_the_label_and_follows_theme_changes() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();
    assert_eq!(label_smoothing(&app, button), FontSmoothing::AntiAliased);

    app.insert_resource(GlobalTheme(Theme {
        font_smoothing: FontSmoothing::None,
        ..probe_theme()
    }));
    app.update();
    assert_eq!(label_smoothing(&app, button), FontSmoothing::None);
}

#[test]
fn surface_theme_tints_the_image_and_keeps_the_background_transparent() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(surface_theme()));
    let button = spawn_primary(&mut app);
    app.update();

    let style = primary(&surface_theme());
    let node = image_node(&app, button).expect("surface adds an ImageNode");
    assert_eq!(node.image, panel_image());
    assert!(matches!(node.image_mode, NodeImageMode::Sliced(_)));
    assert_eq!(node.visual_box, VisualBox::BorderBox);
    assert_eq!(node.color, style.bg.normal);
    assert_eq!(bg(&app, button), Color::NONE);

    for (state, expected) in [
        (PickingInteraction::Hovered, style.bg.hovered),
        (PickingInteraction::Pressed, style.bg.pressed),
    ] {
        app.world_mut().entity_mut(button).insert(state);
        app.update();
        assert_eq!(image_node(&app, button).unwrap().color, expected);
    }

    app.world_mut()
        .entity_mut(button)
        .insert((PickingInteraction::None, InteractionDisabled));
    app.update();
    assert_eq!(image_node(&app, button).unwrap().color, style.bg.disabled);
}

#[test]
fn a_fill_is_the_background_under_the_image_and_follows_the_state() {
    let mut theme = surface_theme();
    for style in theme.buttons.named.values_mut() {
        style.fill = Some(style.border);
    }
    let fill = primary(&theme).fill.expect("the primary style has a fill");
    let mut app = test_app();
    app.insert_resource(GlobalTheme(theme));
    let button = spawn_primary(&mut app);
    app.update();
    assert_eq!(bg(&app, button), fill.normal);

    app.world_mut()
        .entity_mut(button)
        .insert(PickingInteraction::Hovered);
    app.update();
    assert_eq!(bg(&app, button), fill.hovered);
    // The image is still tinted by `bg`, not by the fill.
    assert_eq!(
        image_node(&app, button).unwrap().color,
        primary(&surface_theme()).bg.hovered
    );
}

#[test]
fn swapping_to_a_theme_without_surface_removes_the_image_and_restores_the_background() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(surface_theme()));
    let button = spawn_primary(&mut app);
    app.update();
    assert!(image_node(&app, button).is_some());

    app.insert_resource(GlobalTheme(probe_theme()));
    app.update();
    assert!(image_node(&app, button).is_none());
    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.normal);

    app.insert_resource(GlobalTheme(surface_theme()));
    app.update();
    assert!(image_node(&app, button).is_some());
    assert_eq!(bg(&app, button), Color::NONE);
}

// A default `ImageNode` paints a white box, so a style without a surface must never add one.
#[test]
fn a_style_without_a_surface_never_adds_an_image_node() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    app.update();

    assert!(image_node(&app, button).is_none());
}

#[test]
fn scoped_theme_change_updates_only_buttons_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_primary(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_primary_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = swapped_theme();
    app.update();

    assert_eq!(bg(&app, inside), primary(&swapped_theme()).bg.normal);
    assert_eq!(bg(&app, outside), primary(&probe_theme()).bg.normal);
}

#[test]
fn button_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    app.update();

    let late = spawn_primary_under(&mut app, panel);
    app.update();

    assert_eq!(bg(&app, late), primary(&swapped_theme()).bg.normal);
    assert_eq!(label_font(&app, late), swapped_theme().font);
}

#[test]
fn removing_the_theme_component_makes_buttons_follow_the_global_theme() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    let button = spawn_primary_under(&mut app, panel);
    app.update();
    assert_eq!(bg(&app, button), primary(&swapped_theme()).bg.normal);

    app.world_mut().entity_mut(panel).remove::<Theme>();
    app.update();

    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.normal);
}

#[test]
fn moving_a_button_to_another_scope_makes_it_follow_that_scope() {
    let mut app = test_app();
    let swapped = app.world_mut().spawn(swapped_theme()).id();
    let plain = app.world_mut().spawn(probe_theme()).id();
    let button = spawn_primary_under(&mut app, swapped);
    app.update();

    app.world_mut().entity_mut(button).insert(ChildOf(plain));
    app.update();

    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.normal);
}

#[test]
fn detaching_a_button_from_a_scope_makes_it_follow_the_global_theme() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    let button = spawn_primary_under(&mut app, panel);
    app.update();

    app.world_mut().entity_mut(button).remove::<ChildOf>();
    app.update();

    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.normal);
}

/// The probe theme with its Primary/Affirmative palettes exchanged and another font.
fn swapped_theme() -> Theme {
    let mut theme = probe_theme();
    let named = &mut theme.buttons.named;
    let primary = named.remove(&ButtonVariantId::PRIMARY);
    let affirmative = named.remove(&ButtonVariantId::AFFIRMATIVE);
    named.extend(affirmative.map(|style| (ButtonVariantId::PRIMARY, style)));
    named.extend(primary.map(|style| (ButtonVariantId::AFFIRMATIVE, style)));
    theme.font = FontSource::sans_serif();
    theme
}

fn panel_image() -> Handle<Image> {
    uuid_handle!("6a1f5c3e-0b52-4d8e-9a77-3c1d2e4f5a60")
}

/// The probe theme with a 9-sliced image on every button.
fn surface_theme() -> Theme {
    let mut theme = probe_theme();
    let surface = SlicedImage {
        image: panel_image(),
        slicer: TextureSlicer {
            border: BorderRect::all(8.0),
            ..default()
        },
    };
    for style in theme.buttons.named.values_mut() {
        style.surface = Some(surface.clone());
    }
    theme
}

fn image_node(app: &App, button: Entity) -> Option<&ImageNode> {
    app.world().get::<ImageNode>(button)
}

fn label_smoothing(app: &App, button: Entity) -> FontSmoothing {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextFont>(child).map(|f| f.font_smoothing))
        .expect("button has a label child")
}
