//! Behavior tests for the toggle switch: styling, theme resolution, focus, tab order and the
//! controlled-state wiring on top of Bevy's `Checkbox`.
//!
//! Real pointer picking, layout and the sliding look are covered by `examples/toggle_check.rs`.

use accesskit::Role;
use bevy::{
    a11y::AccessibilityNode,
    input_focus::{InputFocus, InputFocusVisible, tab_navigation::TabIndex},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{Checked, InteractionDisabled},
    ui_widgets::{Checkbox, CheckboxPlugin, SetChecked, ValueChange},
};
use bevy_ui_component_prototypes::{
    GlobalTheme, StateColors, Theme, ToggleSwitch, UiComponentPrototypesPlugin, toggle_switch,
};

// ---------------------------------------------------------------------------
// Behavior wiring
// ---------------------------------------------------------------------------

#[test]
fn toggle_switch_carries_the_bevy_checkbox_behavior_components() {
    let mut app = test_app();
    let switch = spawn_switch(&mut app);

    let world = app.world();
    assert!(world.get::<Checkbox>(switch).is_some());
    assert!(world.get::<PickingInteraction>(switch).is_some());
    assert_eq!(world.get::<TabIndex>(switch).map(|t| t.0), Some(0));
    assert!(world.get::<Checked>(switch).is_none(), "starts off");
    assert_eq!(world.get::<Children>(switch).map(|c| c.len()), Some(1));
    // `Outline::default()` would be a visible white line; the ring must start hidden.
    assert_eq!(outline(&app, switch).color, Color::NONE);
}

// ---------------------------------------------------------------------------
// Initial styling and the checked state
// ---------------------------------------------------------------------------

#[test]
fn unchecked_switch_is_styled_after_one_update() {
    let mut app = test_app();
    let switch = spawn_switch(&mut app);

    app.update();

    let style = Theme::default().switch;
    assert_eq!(track(&app, switch), style.track.unchecked.normal);
    assert_eq!(thumb(&app, switch), style.thumb.unchecked.normal);
    assert_eq!(thumb_side(&app, switch), JustifyContent::Start);
}

// `ToggleSwitch` is public, so it can be spawned without the builder and its thumb.
#[test]
fn a_switch_without_a_thumb_is_still_styled() {
    let mut app = test_app();
    let switch = app.world_mut().spawn(ToggleSwitch).id();

    app.update();

    let style = Theme::default().switch;
    assert_eq!(track(&app, switch), style.track.unchecked.normal);
    assert_eq!(thumb_side(&app, switch), JustifyContent::Start);
}

#[test]
fn attaching_the_component_to_an_existing_entity_styles_it() {
    let mut app = test_app();
    let entity = app.world_mut().spawn(PickingInteraction::None).id();
    app.update();

    app.world_mut().entity_mut(entity).insert(ToggleSwitch);
    app.update();

    let style = Theme::default().switch;
    assert_eq!(track(&app, entity), style.track.unchecked.normal);
}

#[test]
fn a_switch_spawned_checked_is_styled_as_on_after_one_update() {
    let mut app = test_app();
    let switch = spawn_switch_checked(&mut app);

    app.update();

    let style = Theme::default().switch;
    assert_eq!(track(&app, switch), style.track.checked.normal);
    assert_eq!(thumb_side(&app, switch), JustifyContent::End);
}

#[test]
fn checked_toggles_the_track_color_and_moves_the_thumb_both_ways() {
    let mut app = test_app();
    let switch = spawn_switch(&mut app);
    let style = Theme::default().switch;
    app.update();

    app.world_mut().entity_mut(switch).insert(Checked);
    app.update();
    assert_eq!(track(&app, switch), style.track.checked.normal);
    assert_eq!(thumb_side(&app, switch), JustifyContent::End);

    // Removal is not a `Changed`; it has to be picked up through `RemovedComponents`.
    app.world_mut().entity_mut(switch).remove::<Checked>();
    app.update();
    assert_eq!(track(&app, switch), style.track.unchecked.normal);
    assert_eq!(thumb_side(&app, switch), JustifyContent::Start);
}

// ---------------------------------------------------------------------------
// Interaction states
// ---------------------------------------------------------------------------

#[test]
fn hover_and_press_change_colors_for_both_states_and_release_restores_them() {
    let mut app = test_app();
    let switch = spawn_switch(&mut app);
    let style = Theme::default().switch;
    app.update();

    for (checked, colors) in [(false, style.track.unchecked), (true, style.track.checked)] {
        if checked {
            app.world_mut().entity_mut(switch).insert(Checked);
        }
        for (state, expected) in [
            (PickingInteraction::Hovered, colors.hovered),
            (PickingInteraction::Pressed, colors.pressed),
            (PickingInteraction::None, colors.normal),
        ] {
            app.world_mut().entity_mut(switch).insert(state);
            app.update();
            assert_eq!(track(&app, switch), expected);
        }
    }
}

#[test]
fn interaction_disabled_grays_the_switch_and_removal_restores_it() {
    let mut app = test_app();
    let switch = spawn_switch_checked(&mut app);
    let style = Theme::default().switch;
    app.update();

    app.world_mut()
        .entity_mut(switch)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(track(&app, switch), style.track.checked.disabled);
    assert_eq!(thumb(&app, switch), style.thumb.checked.disabled);

    app.world_mut()
        .entity_mut(switch)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(track(&app, switch), style.track.checked.normal);
    assert_eq!(thumb(&app, switch), style.thumb.checked.normal);
}

// ---------------------------------------------------------------------------
// Theme resolution
// ---------------------------------------------------------------------------

#[test]
fn global_theme_change_updates_unscoped_switches_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_switch_checked(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let scoped = spawn_switch_checked_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_on_theme()));
    app.update();

    assert_eq!(track(&app, unscoped), red());
    assert_eq!(
        track(&app, scoped),
        Theme::default().switch.track.checked.normal
    );
}

#[test]
fn scoped_theme_change_updates_only_switches_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_switch_checked(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let inside = spawn_switch_checked_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_on_theme();
    app.update();

    assert_eq!(track(&app, inside), red());
    assert_eq!(
        track(&app, outside),
        Theme::default().switch.track.checked.normal
    );
}

#[test]
fn switch_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_on_theme()).id();
    app.update();

    let late = spawn_switch_checked_under(&mut app, panel);
    app.update();

    assert_eq!(track(&app, late), red());
}

// ---------------------------------------------------------------------------
// Focus
// ---------------------------------------------------------------------------

#[test]
fn visible_focus_draws_the_ring_and_uses_the_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let switch = spawn_switch(&mut app);
    app.update();

    focus(&mut app, switch, true);
    app.update();

    let style = focus_theme().switch;
    assert_eq!(track(&app, switch), style.track.unchecked.focused);
    let ring = outline(&app, switch);
    assert_eq!(ring.color, style.focus_ring.color);
    assert_eq!(ring.width, style.focus_ring.width);
}

// Pointer clicks hide the indicator while the entity keeps focus.
#[test]
fn focus_is_not_shown_while_the_indicator_is_hidden() {
    let mut app = test_app_with_focus_theme();
    let switch = spawn_switch(&mut app);
    app.update();

    focus(&mut app, switch, false);
    app.update();

    assert_eq!(
        track(&app, switch),
        focus_theme().switch.track.unchecked.normal
    );
    assert_eq!(outline(&app, switch).color, Color::NONE);
}

#[test]
fn moving_focus_moves_the_ring() {
    let mut app = test_app_with_focus_theme();
    let first = spawn_switch(&mut app);
    let second = spawn_switch(&mut app);
    app.update();
    focus(&mut app, first, true);
    app.update();

    focus(&mut app, second, true);
    app.update();

    assert_eq!(outline(&app, first).color, Color::NONE);
    assert_eq!(
        outline(&app, second).color,
        focus_theme().switch.focus_ring.color
    );
}

#[test]
fn disabled_switch_shows_neither_ring_nor_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let switch = spawn_switch(&mut app);
    app.update();
    focus(&mut app, switch, true);
    app.world_mut()
        .entity_mut(switch)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(
        track(&app, switch),
        focus_theme().switch.track.unchecked.disabled
    );
    assert_eq!(outline(&app, switch).color, Color::NONE);
}

// ---------------------------------------------------------------------------
// Tab order
// ---------------------------------------------------------------------------

#[test]
fn disabled_switch_leaves_the_tab_order_and_returns_when_enabled() {
    let mut app = test_app();
    let switch = spawn_switch(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(switch)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(tab_index(&app, switch), -1);

    app.world_mut()
        .entity_mut(switch)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, switch), 0);
}

#[test]
fn an_accessible_label_given_by_the_app_coexists_with_the_switch_role() {
    let mut app = test_app();
    let switch = app
        .world_mut()
        .spawn((toggle_switch().build(), AccessibleLabel::new("Sound")))
        .id();
    app.update();

    let node = app.world().get::<AccessibilityNode>(switch).unwrap();
    assert_eq!(node.role(), Role::Switch);
    assert_eq!(node.label(), Some("Sound"));
}

// ---------------------------------------------------------------------------
// Controlled state: the switch asks, the app decides
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, bool)>);

fn app_with_checkbox_behavior() -> App {
    let mut app = test_app();
    app.add_plugins(CheckboxPlugin)
        .init_resource::<Requests>()
        .add_observer(|change: On<ValueChange<bool>>, mut log: ResMut<Requests>| {
            log.0.push((change.source, change.value));
        });
    app
}

#[test]
fn a_request_reports_the_new_value_but_never_changes_the_state_by_itself() {
    let mut app = app_with_checkbox_behavior();
    let switch = spawn_switch(&mut app);
    app.update();

    app.world_mut().trigger(SetChecked {
        entity: switch,
        checked: true,
    });
    app.update();

    assert_eq!(app.world().resource::<Requests>().0, vec![(switch, true)]);
    assert!(app.world().get::<Checked>(switch).is_none());
    assert_eq!(thumb_side(&app, switch), JustifyContent::Start);
}

#[test]
fn the_app_applying_the_request_flips_the_look() {
    let mut app = app_with_checkbox_behavior();
    app.add_observer(|change: On<ValueChange<bool>>, mut commands: Commands| {
        if change.value {
            commands.entity(change.source).insert(Checked);
        } else {
            commands.entity(change.source).remove::<Checked>();
        }
    });
    let switch = spawn_switch(&mut app);
    app.update();

    app.world_mut().trigger(SetChecked {
        entity: switch,
        checked: true,
    });
    app.update();
    assert_eq!(thumb_side(&app, switch), JustifyContent::End);

    app.world_mut().trigger(SetChecked {
        entity: switch,
        checked: false,
    });
    app.update();
    assert_eq!(thumb_side(&app, switch), JustifyContent::Start);
}

#[test]
fn a_disabled_switch_makes_no_request() {
    let mut app = app_with_checkbox_behavior();
    let switch = spawn_switch(&mut app);
    app.world_mut()
        .entity_mut(switch)
        .insert(InteractionDisabled);
    app.update();

    app.world_mut().trigger(SetChecked {
        entity: switch,
        checked: true,
    });
    app.update();

    assert!(app.world().resource::<Requests>().0.is_empty());
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app
}

fn spawn_switch(app: &mut App) -> Entity {
    app.world_mut().spawn(toggle_switch().build()).id()
}

fn spawn_switch_checked(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((toggle_switch().build(), Checked))
        .id()
}

fn spawn_switch_checked_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((toggle_switch().build(), Checked, ChildOf(parent)))
        .id()
}

fn red() -> Color {
    Color::srgb(0.9, 0.1, 0.1)
}

/// Default theme whose checked track is red, so that it is distinguishable from the default.
fn red_on_theme() -> Theme {
    let mut theme = Theme::default();
    theme.switch.track.checked = StateColors::uniform(red());
    theme
}

/// Default theme whose `focused` track color differs from every other state.
fn focus_theme() -> Theme {
    let mut theme = Theme::default();
    theme.switch.track.unchecked.focused = Color::srgb(0.1, 0.2, 0.3);
    theme
}

fn test_app_with_focus_theme() -> App {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(focus_theme()));
    app
}

fn focus(app: &mut App, switch: Entity, visible: bool) {
    app.insert_resource(InputFocus::from_entity(switch));
    app.insert_resource(InputFocusVisible(visible));
}

fn outline(app: &App, switch: Entity) -> Outline {
    *app.world().get::<Outline>(switch).unwrap()
}

fn tab_index(app: &App, switch: Entity) -> i32 {
    app.world().get::<TabIndex>(switch).unwrap().0
}

fn track(app: &App, switch: Entity) -> Color {
    app.world().get::<BackgroundColor>(switch).unwrap().0
}

fn thumb_entity(app: &App, switch: Entity) -> Entity {
    app.world().get::<Children>(switch).unwrap()[0]
}

fn thumb(app: &App, switch: Entity) -> Color {
    let thumb = thumb_entity(app, switch);
    app.world().get::<BackgroundColor>(thumb).unwrap().0
}

fn thumb_side(app: &App, switch: Entity) -> JustifyContent {
    app.world().get::<Node>(switch).unwrap().justify_content
}
