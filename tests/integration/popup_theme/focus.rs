//! Tests of the keyboard and the screen reader: where Tab goes, and what a popup is called.

use super::{
    buttons::{buttons, spawn_confirm_popup},
    panel,
};
use crate::support::test_app;
use bevy::{
    a11y::AccessibilityNode,
    ecs::system::RunSystemOnce,
    input_focus::{
        AcquireFocus, InputFocus, on_window_acquire_focus_clear,
        tab_navigation::{NavAction, TabGroup, TabIndex, TabNavigation, acquire_focus_tab_index},
    },
    prelude::*,
};
use bevy_ui_component_prototypes::{ButtonVariant, PopupChoice, popup};

/// The layer of the popup the example puts in front, well above the one of the theme.
const FRONT_Z: i32 = 1000;

fn label_of(app: &App, entity: Entity) -> Option<String> {
    app.world()
        .get::<AccessibleLabel>(entity)
        .map(|label| label.0.clone())
}

/// Without the visibility plugins nothing is ever propagated, and Tab navigation skips hidden entities.
fn show_all(app: &mut App, entities: &[Entity]) {
    for entity in entities {
        app.world_mut()
            .entity_mut(*entity)
            .insert(InheritedVisibility::VISIBLE);
    }
}

fn navigate(app: &mut App, from: Entity, action: NavAction) -> Entity {
    app.world_mut()
        .run_system_once(move |navigation: TabNavigation| {
            navigation.navigate(&InputFocus::from_entity(from), action)
        })
        .unwrap()
        .expect("tab navigation finds a target")
}

#[test]
fn a_popup_is_a_modal_dialog_named_by_its_message() {
    let mut app = test_app();
    let root = spawn_confirm_popup(&mut app);

    let node = app.world().get::<AccessibilityNode>(root).unwrap();
    assert_eq!(node.role(), accesskit::Role::Dialog);
    assert!(node.is_modal());
    assert_eq!(label_of(&app, root).as_deref(), Some("Delete?"));
}

#[test]
fn a_button_is_named_by_its_label() {
    let mut app = test_app();
    let root = spawn_confirm_popup(&mut app);

    let buttons = buttons(&app, root);
    assert_eq!(label_of(&app, buttons[0]).as_deref(), Some("Delete"));
    assert_eq!(label_of(&app, buttons[1]).as_deref(), Some("Keep"));
}

#[test]
fn a_popup_is_a_modal_tab_group() {
    let mut app = test_app();
    let root = spawn_confirm_popup(&mut app);

    assert!(app.world().get::<TabGroup>(root).unwrap().modal);
}

#[test]
fn a_popup_takes_the_focus_when_spawned() {
    let mut app = test_app();
    let behind = app.world_mut().spawn_empty().id();
    app.insert_resource(InputFocus::from_entity(behind));

    let root = spawn_confirm_popup(&mut app);

    assert_eq!(
        app.world().resource::<InputFocus>().get(),
        Some(panel(&app, root))
    );
}

#[test]
fn the_focus_goes_to_the_popup_spawned_last_even_when_it_is_behind() {
    let mut app = test_app();
    app.insert_resource(InputFocus::default());
    let front = app
        .world_mut()
        .spawn((
            popup()
                .message("In front")
                .button("Yes", PopupChoice::Confirm, ButtonVariant::Default)
                .button("No", PopupChoice::Cancel, ButtonVariant::Default)
                .build(),
            GlobalZIndex(FRONT_Z),
        ))
        .id();
    let behind = spawn_confirm_popup(&mut app);
    let [front_yes, front_no] = buttons(&app, front)[..] else {
        panic!("two buttons");
    };
    let [delete, keep] = buttons(&app, behind)[..] else {
        panic!("two buttons");
    };
    let behind_panel = panel(&app, behind);
    let shown = [
        front,
        panel(&app, front),
        app.world().get::<ChildOf>(front_yes).unwrap().parent(),
        front_yes,
        front_no,
        behind,
        behind_panel,
        app.world().get::<ChildOf>(delete).unwrap().parent(),
        delete,
        keep,
    ];
    show_all(&mut app, &shown);
    app.update();

    let layer = |e: Entity| app.world().get::<GlobalZIndex>(e).unwrap().0;
    assert!(layer(front) > layer(behind));
    assert_eq!(
        app.world().resource::<InputFocus>().get(),
        Some(behind_panel)
    );
    // The popup that has the focus is a modal tab group: Tab never reaches the one in front.
    assert_eq!(navigate(&mut app, behind_panel, NavAction::Next), delete);
    assert_eq!(navigate(&mut app, delete, NavAction::Next), keep);
    assert_eq!(navigate(&mut app, keep, NavAction::Next), delete);
}

#[test]
fn a_popup_spawns_without_the_focus_resource() {
    let mut app = test_app();

    let root = spawn_confirm_popup(&mut app);
    app.update();

    assert!(app.world().get_entity(root).is_ok());
}

#[test]
fn tab_cycles_through_the_buttons_of_the_popup_and_never_reaches_what_is_behind() {
    let mut app = test_app();
    let background = app.world_mut().spawn(TabGroup::default()).id();
    let behind = app
        .world_mut()
        .spawn((TabIndex(0), ChildOf(background)))
        .id();
    let root = spawn_confirm_popup(&mut app);
    let panel = panel(&app, root);
    let [delete, keep] = buttons(&app, root)[..] else {
        panic!("two buttons");
    };
    let row = app.world().get::<ChildOf>(delete).unwrap().parent();
    let shown = [behind, root, panel, row, delete, keep];
    show_all(&mut app, &shown);
    app.update();

    // The focus starts on the panel, which is not a button: Tab goes to the first one.
    assert_eq!(navigate(&mut app, panel, NavAction::Next), delete);
    assert_eq!(navigate(&mut app, delete, NavAction::Next), keep);
    assert_eq!(navigate(&mut app, keep, NavAction::Next), delete);
    assert_eq!(navigate(&mut app, delete, NavAction::Previous), keep);
}

// ---------------------------------------------------------------------------
// Pressing on the popup
// ---------------------------------------------------------------------------

/// An app that resolves a press like the ones of `DefaultPlugins` do: a target with a `TabIndex` takes the
/// focus, and a press that finds none clears it at the window.
fn pressable_app() -> (App, Entity) {
    let mut app = test_app();
    app.init_resource::<InputFocus>()
        .add_observer(acquire_focus_tab_index)
        .add_observer(on_window_acquire_focus_clear);
    let window = app.world_mut().spawn(Window::default()).id();
    (app, window)
}

fn press(app: &mut App, window: Entity, target: Entity) {
    app.world_mut().trigger(AcquireFocus {
        focused_entity: target,
        window,
    });
    app.update();
}

fn focused(app: &App) -> Option<Entity> {
    app.world().resource::<InputFocus>().get()
}

#[test]
fn a_press_on_the_scrim_or_the_message_keeps_the_focus_in_the_popup() {
    let (mut app, window) = pressable_app();
    let root = spawn_confirm_popup(&mut app);
    let panel = panel(&app, root);
    let message = app.world().get::<Children>(panel).unwrap()[0];

    for target in [root, panel, message] {
        app.world_mut().resource_mut::<InputFocus>().clear();
        press(&mut app, window, target);
        assert_eq!(focused(&app), Some(panel));
    }
}

#[test]
fn a_press_on_a_button_focuses_the_button() {
    let (mut app, window) = pressable_app();
    let root = spawn_confirm_popup(&mut app);
    let keep = buttons(&app, root)[1];

    press(&mut app, window, keep);

    assert_eq!(focused(&app), Some(keep));
}

#[test]
fn a_press_outside_the_popup_still_clears_the_focus() {
    let (mut app, window) = pressable_app();
    spawn_confirm_popup(&mut app);
    let outside = app.world_mut().spawn_empty().id();

    press(&mut app, window, outside);

    assert_eq!(focused(&app), None);
}
