//! Behavior tests for button styling and theme resolution.
//!
//! Real pointer picking, layout, and text shaping are not covered by `examples/button_check.rs`.

use bevy::{
    a11y::AccessibilityNode, input_focus::tab_navigation::TabIndex,
    picking::hover::PickingInteraction, prelude::*, ui::InteractionDisabled, ui_widgets::Button,
};
use bevy_ui_component_prototypes::{
    ButtonSize, ButtonStyles, ButtonVariant, GlobalTheme, Theme, UiComponentPrototypesPlugin,
    icon_button,
};

// ---------------------------------------------------------------------------
// Behavior wiring
// ---------------------------------------------------------------------------

#[test]
fn icon_button_carries_the_bevy_button_behavior_components() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    let world = app.world();
    assert!(world.get::<Button>(button).is_some());
    assert!(world.get::<AccessibilityNode>(button).is_some());
    assert!(world.get::<PickingInteraction>(button).is_some());
    assert_eq!(world.get::<TabIndex>(button).map(|t| t.0), Some(0));
}

// ---------------------------------------------------------------------------
// Initial styling
// ---------------------------------------------------------------------------

#[test]
fn newly_spawned_button_is_styled_after_one_update() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    app.update();

    let expected = Theme::default();
    assert_eq!(bg(&app, button), expected.buttons.primary.bg.normal);
    assert_eq!(
        label_color(&app, button),
        expected.buttons.primary.fg.normal
    );
    assert_eq!(label_font(&app, button), expected.font);
}

// ---------------------------------------------------------------------------
// Interaction states
// ---------------------------------------------------------------------------

#[test]
fn hover_and_press_change_colors_and_release_restores_them() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    let style = Theme::default().buttons.primary;
    app.update();

    for (state, expected) in [
        (PickingInteraction::Hovered, style.bg.hovered),
        (PickingInteraction::Pressed, style.bg.pressed),
        (PickingInteraction::None, style.bg.normal),
    ] {
        app.world_mut().entity_mut(button).insert(state);
        app.update();
        assert_eq!(bg(&app, button), expected);
    }
}

// [D] toggle disabled
#[test]
fn interaction_disabled_grays_the_button_and_removal_restores_it() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    let style = Theme::default().buttons.primary;
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(bg(&app, button), style.bg.disabled);
    assert_eq!(label_color(&app, button), style.fg.disabled);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(bg(&app, button), style.bg.normal);
    assert_eq!(label_color(&app, button), style.fg.normal);
}

// ---------------------------------------------------------------------------
// Theme resolution
// ---------------------------------------------------------------------------

// [T] swap the global theme
#[test]
fn global_theme_change_updates_unscoped_buttons_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_primary(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let scoped = spawn_primary_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(swapped_theme()));
    app.update();

    let swapped = swapped_theme();
    assert_eq!(bg(&app, unscoped), swapped.buttons.primary.bg.normal);
    assert_eq!(label_font(&app, unscoped), swapped.font);
    assert_eq!(bg(&app, scoped), Theme::default().buttons.primary.bg.normal);
}

// [S] swap the panel's own theme
#[test]
fn scoped_theme_change_updates_only_buttons_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_primary(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let inside = spawn_primary_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = swapped_theme();
    app.update();

    assert_eq!(bg(&app, inside), swapped_theme().buttons.primary.bg.normal);
    assert_eq!(
        bg(&app, outside),
        Theme::default().buttons.primary.bg.normal
    );
}

// [N] spawn at runtime
#[test]
fn button_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    app.update();

    let late = spawn_primary_under(&mut app, panel);
    app.update();

    assert_eq!(bg(&app, late), swapped_theme().buttons.primary.bg.normal);
    assert_eq!(label_font(&app, late), swapped_theme().font);
}

#[test]
fn removing_the_theme_component_makes_buttons_follow_the_global_theme() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    let button = spawn_primary_under(&mut app, panel);
    app.update();
    assert_eq!(bg(&app, button), swapped_theme().buttons.primary.bg.normal);

    app.world_mut().entity_mut(panel).remove::<Theme>();
    app.update();

    assert_eq!(bg(&app, button), Theme::default().buttons.primary.bg.normal);
}

#[test]
fn detaching_a_button_from_a_scope_makes_it_follow_the_global_theme() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    let button = spawn_primary_under(&mut app, panel);
    app.update();

    app.world_mut().entity_mut(button).remove::<ChildOf>();
    app.update();

    assert_eq!(bg(&app, button), Theme::default().buttons.primary.bg.normal);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app
}

/// A theme whose Primary/Affirmative palettes are exchanged and whose font differs
/// so that it is distinguishable from `Theme::default()`.
fn swapped_theme() -> Theme {
    let base = Theme::default();
    Theme {
        buttons: ButtonStyles {
            primary: base.buttons.affirmative,
            affirmative: base.buttons.primary,
            ..base.buttons
        },
        font: FontSource::sans_serif(),
    }
}

fn spawn_primary(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(
            icon_button("button")
                .size(ButtonSize::small())
                .variant(ButtonVariant::Primary)
                .build(),
        )
        .id()
}

fn spawn_primary_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((
            icon_button("button")
                .size(ButtonSize::small())
                .variant(ButtonVariant::Primary)
                .build(),
            ChildOf(parent),
        ))
        .id()
}

fn bg(app: &App, button: Entity) -> Color {
    app.world().get::<BackgroundColor>(button).unwrap().0
}

fn label_color(app: &App, button: Entity) -> Color {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextColor>(child).map(|c| c.0))
        .expect("button has a label child")
}

fn label_font(app: &App, button: Entity) -> FontSource {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextFont>(child).map(|f| f.font.clone()))
        .expect("button has a label child")
}
