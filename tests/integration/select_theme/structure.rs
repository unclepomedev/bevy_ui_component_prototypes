//! Tests of Structure.

use super::{Fruit, Marker, list, popup, row_label, rows, spawn_fruit, trigger};
use crate::support::{outline, test_app};
use bevy::{
    input_focus::tab_navigation::TabIndex,
    prelude::*,
    ui_widgets::{ListBox, ListItem, MenuButton, MenuPopup},
};
use bevy_ui_component_prototypes::select::{Select, select};

#[test]
fn a_select_is_a_root_with_a_trigger_that_takes_the_focus() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    let world = app.world();
    assert!(world.get::<Select>(root).is_some());
    assert_eq!(world.get::<Children>(root).map(|c| c.len()), Some(2));
    let trigger = trigger(&app, root);
    assert!(world.get::<MenuButton>(trigger).is_some());
    assert_eq!(world.get::<TabIndex>(trigger).map(|t| t.0), Some(0));
    // `Outline::default()` would be a visible white line; the ring must start hidden.
    assert_eq!(outline(&app, trigger).color, Color::NONE);
    assert_eq!(world.get::<Children>(trigger).map(|c| c.len()), Some(2));
}

#[test]
fn the_popup_is_a_hidden_menu_popup_next_to_the_trigger() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    let world = app.world();
    let popup = popup(&app, root);
    assert!(world.get::<MenuPopup>(popup).is_some());
    assert_eq!(world.get::<Visibility>(popup), Some(&Visibility::Hidden));
    assert_eq!(world.get::<Children>(popup).map(|c| c.len()), Some(2));
}

#[test]
fn the_list_can_take_the_focus() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    let world = app.world();
    let list = list(&app, root);
    assert!(world.get::<ListBox>(list).is_some());
    assert_eq!(world.get::<TabIndex>(list).map(|t| t.0), Some(0));
}

#[test]
fn the_list_has_a_row_per_option_in_order_with_its_label() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    let rows = rows(&app, root);
    assert!(
        rows.iter()
            .all(|row| app.world().get::<ListItem>(*row).is_some())
    );
    let names: Vec<String> = rows
        .iter()
        .map(|row| {
            let label = row_label(&app, *row);
            app.world().get::<Text>(label).unwrap().0.clone()
        })
        .collect();
    assert_eq!(names, ["Apple", "Banana", "Cherry"]);
}

#[test]
fn a_label_bundle_lands_on_the_label_of_its_row_only() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .option(Fruit::Apple, (Text::new("Apple"), Marker(7)))
                .build(),
        )
        .id();

    let row = rows(&app, root)[0];
    let label = row_label(&app, row);
    assert_eq!(app.world().get::<Marker>(label).map(|m| m.0), Some(7));
    assert!(app.world().get::<Marker>(row).is_none());
    assert!(app.world().get::<Marker>(list(&app, root)).is_none());
}
