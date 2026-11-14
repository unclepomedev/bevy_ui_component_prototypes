//! Tests of The selected row.

use super::{Fruit, selected_rows, spawn_fruit, text};
use crate::support::test_app;
use bevy_ui_component_prototypes::select::{SelectedOption, select};

#[test]
fn the_row_of_the_initial_choice_is_selected() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .options([
                    (Fruit::Apple, text("Apple")),
                    (Fruit::Banana, text("Banana")),
                    (Fruit::Cherry, text("Cherry")),
                ])
                .selected(Fruit::Banana)
                .build(),
        )
        .id();

    app.update();

    assert_eq!(selected_rows(&app, root), [false, true, false]);
}

#[test]
fn with_nothing_chosen_no_row_is_selected() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    assert_eq!(selected_rows(&app, root), [false, false, false]);
}

#[test]
fn the_mark_follows_the_choice_and_goes_with_it() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Apple,
    });
    app.update();
    assert_eq!(selected_rows(&app, root), [true, false, false]);

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Cherry,
    });
    app.update();
    assert_eq!(selected_rows(&app, root), [false, false, true]);

    app.world_mut()
        .entity_mut(root)
        .remove::<SelectedOption<Fruit>>();
    app.update();
    assert_eq!(selected_rows(&app, root), [false, false, false]);
}

#[test]
fn a_value_that_is_not_an_option_selects_no_row() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Durian,
    });
    app.update();

    assert_eq!(selected_rows(&app, root), [false, false, false]);
}
