//! Behavior tests for button styling and theme resolution.
//!
//! Real pointer picking, layout, and text shaping are not covered by `examples/button_check.rs`.

use bevy::{
    a11y::AccessibilityNode,
    ecs::system::RunSystemOnce,
    input_focus::{
        InputFocus, InputFocusVisible,
        tab_navigation::{NavAction, TabGroup, TabIndex, TabNavigation},
    },
    picking::hover::PickingInteraction,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::Button,
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
    // `Outline::default()` would be a visible white line; the ring must start hidden.
    assert_eq!(outline(&app, button).color, Color::NONE);
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
// Focus
// ---------------------------------------------------------------------------

#[test]
fn visible_focus_draws_the_ring_and_uses_the_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, true);
    app.update();

    let style = focus_theme().buttons.primary;
    assert_eq!(bg(&app, button), style.bg.focused);
    assert_eq!(label_color(&app, button), style.fg.focused);
    let ring = outline(&app, button);
    assert_eq!(ring.color, style.focus_ring.color);
    assert_eq!(ring.width, style.focus_ring.width);
}

// Pointer clicks hide the indicator while the entity keeps focus.
#[test]
fn focus_is_not_shown_while_the_indicator_is_hidden() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, false);
    app.update();

    assert_eq!(bg(&app, button), focus_theme().buttons.primary.bg.normal);
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[test]
fn hiding_the_indicator_removes_the_ring_and_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.update();

    app.world_mut().resource_mut::<InputFocusVisible>().0 = false;
    app.update();

    assert_eq!(bg(&app, button), focus_theme().buttons.primary.bg.normal);
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[test]
fn moving_focus_moves_the_ring() {
    let mut app = test_app_with_focus_theme();
    let first = spawn_primary(&mut app);
    let second = spawn_primary(&mut app);
    app.update();
    focus(&mut app, first, true);
    app.update();

    focus(&mut app, second, true);
    app.update();

    let style = focus_theme().buttons.primary;
    assert_eq!(outline(&app, first).color, Color::NONE);
    assert_eq!(bg(&app, first), style.bg.normal);
    assert_eq!(outline(&app, second).color, style.focus_ring.color);
    assert_eq!(bg(&app, second), style.bg.focused);
}

// Hover and press take the colors; the ring keeps marking keyboard focus.
#[test]
fn hover_wins_over_focus_for_colors_but_the_ring_stays() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.world_mut()
        .entity_mut(button)
        .insert(PickingInteraction::Hovered);
    app.update();

    let style = focus_theme().buttons.primary;
    assert_eq!(bg(&app, button), style.bg.hovered);
    assert_eq!(outline(&app, button).color, style.focus_ring.color);
}

#[test]
fn disabled_button_shows_neither_ring_nor_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(bg(&app, button), focus_theme().buttons.primary.bg.disabled);
    assert_eq!(outline(&app, button).color, Color::NONE);
}

// ---------------------------------------------------------------------------
// Tab order
// ---------------------------------------------------------------------------

#[test]
fn disabled_button_leaves_the_tab_order_and_returns_when_enabled() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(tab_index(&app, button), -1);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, button), 0);
}

#[test]
fn a_custom_tab_index_is_restored_after_re_enabling() {
    let mut app = test_app();
    let button = app.world_mut().spawn((spawn_bundle(), TabIndex(3))).id();
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(tab_index(&app, button), -1);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, button), 3);
}

#[test]
fn a_button_spawned_disabled_starts_outside_the_tab_order() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((spawn_bundle(), InteractionDisabled))
        .id();
    app.update();
    assert_eq!(tab_index(&app, button), -1);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, button), 0);
}

// End to end: Tab skips the disabled middle button, in both directions.
#[test]
fn tab_navigation_skips_a_disabled_button() {
    let mut app = test_app();
    let group = app.world_mut().spawn(TabGroup::default()).id();
    let first = spawn_primary_under(&mut app, group);
    let middle = spawn_primary_under(&mut app, group);
    let last = spawn_primary_under(&mut app, group);
    // Without the visibility plugins nothing is ever propagated, and Tab navigation skips hidden entities.
    for button in [first, middle, last] {
        app.world_mut()
            .entity_mut(button)
            .insert(InheritedVisibility::VISIBLE);
    }

    // Control: with every button enabled, Tab visits the middle one.
    app.update();
    assert_eq!(navigate(&mut app, first, NavAction::Next), middle);

    app.world_mut()
        .entity_mut(middle)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(navigate(&mut app, first, NavAction::Next), last);
    assert_eq!(navigate(&mut app, last, NavAction::Previous), first);
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

/// Default theme whose `focused` colors differ from every other state, so a test can tell them apart.
fn focus_theme() -> Theme {
    let mut theme = Theme::default();
    theme.buttons.primary.bg.focused = Color::srgb(0.1, 0.2, 0.3);
    theme.buttons.primary.fg.focused = Color::srgb(0.9, 0.8, 0.7);
    theme
}

fn test_app_with_focus_theme() -> App {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(focus_theme()));
    app
}

fn focus(app: &mut App, button: Entity, visible: bool) {
    app.insert_resource(InputFocus::from_entity(button));
    app.insert_resource(InputFocusVisible(visible));
}

fn outline(app: &App, button: Entity) -> Outline {
    *app.world().get::<Outline>(button).unwrap()
}

fn spawn_bundle() -> impl Bundle {
    icon_button("button")
        .size(ButtonSize::small())
        .variant(ButtonVariant::Primary)
        .build()
}

fn tab_index(app: &App, button: Entity) -> i32 {
    app.world().get::<TabIndex>(button).unwrap().0
}

fn navigate(app: &mut App, from: Entity, action: NavAction) -> Entity {
    app.world_mut()
        .run_system_once(move |navigation: TabNavigation| {
            navigation.navigate(&InputFocus::from_entity(from), action)
        })
        .unwrap()
        .expect("tab navigation finds a target")
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
