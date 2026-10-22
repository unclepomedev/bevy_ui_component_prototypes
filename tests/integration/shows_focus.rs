//! Behavior tests for `ShowsFocus`: the marker that tells which widget holds the visible focus indicator.

use bevy::{
    input_focus::{InputFocus, InputFocusVisible},
    prelude::*,
    ui::InteractionDisabled,
};
use bevy_ui_component_prototypes::{
    FocusIndicator, ShowsFocus, UiComponentPrototypesPlugin, icon_button, slider, toggle_switch,
};

#[test]
fn a_focused_widget_with_a_visible_indicator_is_marked() {
    let mut app = test_app();
    let widget = spawn_button(&mut app);

    focus(&mut app, widget, true);
    app.update();

    assert!(marked(&app, widget));
}

// Every widget of the crate, not only buttons.
#[test]
fn buttons_switches_and_sliders_are_all_marked() {
    let mut app = test_app();
    let widgets = [
        app.world_mut().spawn(icon_button("b").build()).id(),
        app.world_mut().spawn(toggle_switch().build()).id(),
        app.world_mut().spawn(slider().build()).id(),
    ];

    for widget in widgets {
        focus(&mut app, widget, true);
        app.update();
        assert!(marked(&app, widget));
    }
}

// A pointer click hides the indicator while the entity keeps the focus.
#[test]
fn the_marker_follows_the_indicator_being_hidden_and_shown() {
    let mut app = test_app();
    let widget = spawn_button(&mut app);
    focus(&mut app, widget, true);
    app.update();

    app.insert_resource(InputFocusVisible(false));
    app.update();
    assert!(!marked(&app, widget));

    app.insert_resource(InputFocusVisible(true));
    app.update();
    assert!(marked(&app, widget));
}

#[test]
fn moving_the_focus_moves_the_marker() {
    let mut app = test_app();
    let first = spawn_button(&mut app);
    let second = spawn_button(&mut app);
    focus(&mut app, first, true);
    app.update();

    focus(&mut app, second, true);
    app.update();

    assert!(!marked(&app, first));
    assert!(marked(&app, second));
}

#[test]
fn clearing_the_focus_removes_the_marker() {
    let mut app = test_app();
    let widget = spawn_button(&mut app);
    focus(&mut app, widget, true);
    app.update();

    app.insert_resource(InputFocus::default());
    app.update();

    assert!(!marked(&app, widget));
}

// The ring is never shown on a disabled widget, and neither is the marker.
#[test]
fn a_disabled_widget_is_not_marked_and_is_again_once_enabled() {
    let mut app = test_app();
    let widget = spawn_button(&mut app);
    focus(&mut app, widget, true);
    app.update();

    app.world_mut()
        .entity_mut(widget)
        .insert(InteractionDisabled);
    app.update();
    assert!(!marked(&app, widget));

    app.world_mut()
        .entity_mut(widget)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(marked(&app, widget));
}

// The marker is about the crate's widgets; any other focused entity is none of its business.
#[test]
fn an_entity_that_is_not_a_widget_of_the_crate_is_not_marked() {
    let mut app = test_app();
    let other = app.world_mut().spawn_empty().id();

    focus(&mut app, other, true);
    app.update();

    assert!(!marked(&app, other));
}

#[test]
fn a_despawned_owner_does_not_break_the_next_update() {
    let mut app = test_app();
    let first = spawn_button(&mut app);
    let second = spawn_button(&mut app);
    focus(&mut app, first, true);
    app.update();

    app.world_mut().despawn(first);
    focus(&mut app, second, true);
    app.update();

    assert!(marked(&app, second));
}

#[test]
fn without_the_input_focus_plugins_nothing_is_marked() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    let widget = spawn_button(&mut app);

    app.update();

    assert!(!marked(&app, widget));
}

#[test]
fn an_always_visible_indicator_marks_the_focus_even_after_a_click_hides_it() {
    let mut app = test_app();
    app.insert_resource(FocusIndicator::Always);
    let widget = spawn_button(&mut app);

    focus(&mut app, widget, false);
    app.update();

    assert!(marked(&app, widget));
}

#[test]
fn going_back_to_auto_hides_the_marker_again() {
    let mut app = test_app();
    app.insert_resource(FocusIndicator::Always);
    let widget = spawn_button(&mut app);
    focus(&mut app, widget, false);
    app.update();

    app.insert_resource(FocusIndicator::Auto);
    app.update();

    assert!(!marked(&app, widget));
}

#[test]
fn an_always_visible_indicator_still_skips_disabled_and_unfocused_widgets() {
    let mut app = test_app();
    app.insert_resource(FocusIndicator::Always);
    let disabled = app
        .world_mut()
        .spawn((icon_button("b").build(), InteractionDisabled))
        .id();
    let other = spawn_button(&mut app);

    focus(&mut app, disabled, false);
    app.update();

    assert!(!marked(&app, disabled));
    assert!(!marked(&app, other));
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app
}

fn spawn_button(app: &mut App) -> Entity {
    app.world_mut().spawn(icon_button("button").build()).id()
}

fn focus(app: &mut App, entity: Entity, visible: bool) {
    app.insert_resource(InputFocus::from_entity(entity));
    app.insert_resource(InputFocusVisible(visible));
}

fn marked(app: &App, entity: Entity) -> bool {
    app.world().get::<ShowsFocus>(entity).is_some()
}
