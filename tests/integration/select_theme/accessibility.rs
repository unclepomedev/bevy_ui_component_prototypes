//! Tests of The name for screen readers.

use super::{
    Fruit, Marker, fruit_select, list, menu_event, popup, rows, shown, spawn_fruit,
    spawn_preselected, trigger, trigger_node,
};
use crate::support::test_app;
use accesskit::{HasPopup, NodeId, Role};
use bevy::{a11y::AccessibilityNode, prelude::*, ui_widgets::MenuAction};
use bevy_ui_component_prototypes::select::{SelectedIndex, SelectedOption, select};

fn named_fruit(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((fruit_select(), AccessibleLabel::new("Fruit")))
        .id()
}

#[test]
fn the_name_given_to_the_select_is_the_name_of_the_trigger() {
    let mut app = test_app();
    let root = named_fruit(&mut app);
    app.update();
    assert_eq!(trigger_name(&app, root), Some("Fruit".to_string()));
    assert!(app.world().get::<AccessibilityNode>(root).is_none());
}

#[test]
fn a_select_without_a_name_has_a_trigger_without_one() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    assert_eq!(trigger_name(&app, root), None);
    assert!(
        app.world()
            .get::<AccessibleLabel>(trigger(&app, root))
            .is_none()
    );
}

#[test]
fn a_name_that_is_changed_changes_the_name_of_the_trigger() {
    let mut app = test_app();
    let root = named_fruit(&mut app);
    app.update();
    app.world_mut()
        .entity_mut(root)
        .insert(AccessibleLabel::new("Berry"));
    app.update();
    assert_eq!(trigger_name(&app, root), Some("Berry".to_string()));
    assert!(app.world().get::<AccessibilityNode>(root).is_none());
}

#[test]
fn a_name_that_is_added_later_reaches_the_trigger() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    app.world_mut()
        .entity_mut(root)
        .insert(AccessibleLabel::new("Fruit"));
    app.update();
    assert_eq!(trigger_name(&app, root), Some("Fruit".to_string()));
}

#[test]
fn a_name_that_is_removed_goes_from_the_trigger_too() {
    let mut app = test_app();
    let root = named_fruit(&mut app);
    app.update();
    app.world_mut().entity_mut(root).remove::<AccessibleLabel>();
    app.update();
    assert_eq!(trigger_name(&app, root), None);
}

#[test]
fn the_trigger_is_a_closed_combo_box_that_opens_a_list() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    let node = trigger_node(&app, root);
    assert_eq!(node.role(), Role::ComboBox);
    assert_eq!(node.has_popup(), Some(HasPopup::Listbox));
    assert_eq!(node.is_expanded(), Some(false));
}

#[test]
fn the_expanded_state_follows_the_list_opening_and_closing() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(trigger_node(&app, root).is_expanded(), Some(true));

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(trigger_node(&app, root).is_expanded(), Some(false));

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(trigger_node(&app, root).is_expanded(), Some(true));
}

#[test]
fn the_name_is_added_to_the_combo_box_without_changing_what_it_is() {
    let mut app = test_app();
    let root = named_fruit(&mut app);

    app.update();

    let node = trigger_node(&app, root);
    assert_eq!(node.label(), Some("Fruit"));
    assert_eq!(node.role(), Role::ComboBox);
    assert_eq!(node.is_expanded(), Some(false));
}

#[test]
fn the_closed_list_is_out_of_the_tree_of_a_screen_reader() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let node = popup_node(&app, root);
    assert_eq!(node.role(), Role::GenericContainer);
    assert!(node.is_hidden());
}

#[test]
fn the_list_is_in_the_tree_while_it_is_open() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert!(!popup_node(&app, root).is_hidden());

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert!(popup_node(&app, root).is_hidden());

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert!(!popup_node(&app, root).is_hidden());
}

#[test]
fn the_trigger_says_which_list_it_opens() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let list = list(&app, root);
    assert_eq!(
        trigger_node(&app, root).controls(),
        [NodeId(list.to_bits())]
    );
}

#[test]
fn the_placeholder_is_read_while_nothing_is_chosen() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let node = trigger_node(&app, root);
    assert_eq!(node.placeholder(), Some("Pick one"));
    assert_eq!(node.value(), None);
}

#[test]
fn the_chosen_option_is_the_value_of_the_trigger() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Banana,
    });
    app.update();
    let node = trigger_node(&app, root);
    assert_eq!(node.value(), Some("Banana"));
    assert_eq!(node.placeholder(), None);

    app.world_mut()
        .entity_mut(root)
        .remove::<SelectedOption<Fruit>>();
    app.update();
    let node = trigger_node(&app, root);
    assert_eq!(node.value(), None);
    assert_eq!(node.placeholder(), Some("Pick one"));
}

#[test]
fn a_text_that_changes_changes_what_is_read() {
    let mut app = test_app();
    let root = spawn_preselected(&mut app, Fruit::Banana);
    app.update();

    let label = shown(&app, root).unwrap();
    app.world_mut().get_mut::<Text>(label).unwrap().0 = "Berry".to_string();
    app.update();

    assert_eq!(trigger_node(&app, root).value(), Some("Berry"));
}

#[test]
fn a_position_past_the_last_option_is_read_as_the_placeholder() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(9) });
    app.update();

    let node = trigger_node(&app, root);
    assert_eq!(node.value(), None);
    assert_eq!(node.placeholder(), Some("Pick one"));
}

#[test]
fn the_rows_are_named_by_the_text_of_their_labels() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let names: Vec<Option<String>> = rows(&app, root)
        .iter()
        .map(|row| {
            app.world()
                .get::<AccessibilityNode>(*row)
                .unwrap()
                .label()
                .map(str::to_owned)
        })
        .collect();
    assert_eq!(
        names,
        [
            Some("Apple".to_string()),
            Some("Banana".to_string()),
            Some("Cherry".to_string())
        ]
    );
}

#[test]
fn a_label_without_a_text_is_not_read() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .option(Fruit::Apple, Marker(1))
                .selected(Fruit::Apple)
                .build(),
        )
        .id();

    app.update();

    assert_eq!(trigger_node(&app, root).value(), None);
    let row = rows(&app, root)[0];
    assert_eq!(
        app.world().get::<AccessibilityNode>(row).unwrap().label(),
        None
    );
}

fn popup_node(app: &App, root: Entity) -> &AccessibilityNode {
    app.world()
        .get::<AccessibilityNode>(popup(app, root))
        .unwrap()
}

fn trigger_name(app: &App, root: Entity) -> Option<String> {
    app.world()
        .get::<AccessibilityNode>(trigger(app, root))
        .unwrap()
        .label()
        .map(str::to_owned)
}
