//! Tests of The rows of the popup.

use super::{
    label_color, list, menu_event, point_at, row_label, rows, spawn_fruit, spawn_fruit_under,
    trigger,
};
use crate::probe::probe_theme;
use crate::support::{bg, red, test_app};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{ActiveDescendant, MenuAction, ScrollArea};
use bevy_ui_component_prototypes::{GlobalTheme, SelectedIndex, Theme};

#[test]
fn the_list_shows_as_many_rows_as_the_theme_says_and_scrolls_beyond() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    let list = list(&app, root);
    assert!(app.world().get::<ScrollArea>(list).is_some());
    let node = app.world().get::<Node>(list).unwrap();
    assert_eq!(node.overflow, Overflow::scroll_y());
    assert_eq!(node.max_height, Val::Px(37.0 * 2.0));
}

#[test]
fn a_theme_change_changes_how_many_rows_the_list_shows() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    let mut theme = probe_theme();
    theme.select_metrics.option_height = 10.0;
    theme.select_metrics.max_visible_options = 5;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    let node = app.world().get::<Node>(list(&app, root)).unwrap();
    assert_eq!(node.max_height, Val::Px(50.0));
}

#[test]
fn a_list_shows_at_least_one_row() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    let mut theme = probe_theme();
    theme.select_metrics.max_visible_options = 0;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    let node = app.world().get::<Node>(list(&app, root)).unwrap();
    assert_eq!(node.max_height, Val::Px(37.0));
}

#[test]
fn the_rows_do_not_shrink_when_the_list_is_shorter_than_all_of_them() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    for row in rows(&app, root) {
        assert_eq!(app.world().get::<Node>(row).unwrap().flex_shrink, 0.0);
    }
}

#[test]
fn the_rows_and_their_labels_are_styled_after_one_update() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let style = probe_theme().select;
    let metrics = probe_theme().select_metrics;
    let row = rows(&app, root)[0];
    assert_eq!(bg(&app, row), style.option_bg.unchecked.normal);
    let node = app.world().get::<Node>(row).unwrap();
    assert_eq!(node.height, Val::Px(metrics.option_height));
    assert_eq!(node.padding, metrics.option_padding);

    let label = row_label(&app, row);
    assert_eq!(label_color(&app, label), style.option_fg.unchecked.normal);
    let font = app.world().get::<TextFont>(label).unwrap();
    assert_eq!(font.font, probe_theme().font);
    assert_eq!(font.font_size, metrics.trigger.font_size);
}

#[test]
fn the_selected_row_takes_the_checked_colors() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(1) });
    app.update();

    let style = probe_theme().select;
    let rows = rows(&app, root);
    assert_eq!(bg(&app, rows[1]), style.option_bg.checked.normal);
    assert_eq!(
        label_color(&app, row_label(&app, rows[1])),
        style.option_fg.checked.normal
    );
    assert_eq!(bg(&app, rows[0]), style.option_bg.unchecked.normal);

    // The choice moves, and the old row goes back.
    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(2) });
    app.update();
    assert_eq!(bg(&app, rows[1]), style.option_bg.unchecked.normal);
    assert_eq!(bg(&app, rows[2]), style.option_bg.checked.normal);
}

#[test]
fn a_press_changes_the_colors_of_a_row_and_its_text_and_release_restores_them() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let style = probe_theme().select;
    app.update();
    let row = rows(&app, root)[0];

    point_at(&mut app, row, PickingInteraction::Pressed);
    assert_eq!(bg(&app, row), style.option_bg.unchecked.pressed);
    assert_eq!(
        label_color(&app, row_label(&app, row)),
        style.option_fg.unchecked.pressed
    );

    point_at(&mut app, row, PickingInteraction::None);
    assert_eq!(bg(&app, row), style.option_bg.unchecked.normal);
}

#[test]
fn the_pointer_over_a_row_of_an_open_list_colors_it_as_the_highlighted_one() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    let style = probe_theme().select;
    menu_event(&mut app, trigger, MenuAction::Toggle);

    point_at(&mut app, rows[1], PickingInteraction::Hovered);

    assert_eq!(bg(&app, rows[1]), style.option_bg.unchecked.focused);
    assert_eq!(
        label_color(&app, row_label(&app, rows[1])),
        style.option_fg.unchecked.focused
    );
    assert_eq!(bg(&app, rows[0]), style.option_bg.unchecked.normal);
}

#[test]
fn a_disabled_row_is_grayed_and_removal_restores_it() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let style = probe_theme().select;
    app.update();
    let row = rows(&app, root)[0];

    app.world_mut().entity_mut(row).insert(InteractionDisabled);
    app.update();
    assert_eq!(bg(&app, row), style.option_bg.unchecked.disabled);
    assert_eq!(
        label_color(&app, row_label(&app, row)),
        style.option_fg.unchecked.disabled
    );

    app.world_mut()
        .entity_mut(row)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(bg(&app, row), style.option_bg.unchecked.normal);
}

#[test]
fn the_highlight_follows_the_active_row_of_the_list() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let style = probe_theme().select;
    app.update();
    let list = list(&app, root);
    let rows = rows(&app, root);

    app.world_mut()
        .entity_mut(list)
        .insert(ActiveDescendant(Some(rows[0])));
    app.update();
    assert_eq!(bg(&app, rows[0]), style.option_bg.unchecked.focused);
    assert_eq!(bg(&app, rows[1]), style.option_bg.unchecked.normal);

    app.world_mut()
        .entity_mut(list)
        .insert(ActiveDescendant(Some(rows[1])));
    app.update();
    assert_eq!(bg(&app, rows[0]), style.option_bg.unchecked.normal);
    assert_eq!(bg(&app, rows[1]), style.option_bg.unchecked.focused);

    app.world_mut()
        .entity_mut(list)
        .insert(ActiveDescendant(None));
    app.update();
    assert_eq!(bg(&app, rows[1]), style.option_bg.unchecked.normal);
}

#[test]
fn global_theme_change_updates_unscoped_rows_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_fruit(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_fruit_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_row_theme()));
    app.update();

    assert_eq!(bg(&app, rows(&app, unscoped)[0]), red());
    assert_eq!(
        bg(&app, rows(&app, scoped)[0]),
        probe_theme().select.option_bg.unchecked.normal
    );
}

#[test]
fn a_theme_change_resizes_the_rows_and_their_text() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    let mut theme = probe_theme();
    theme.select_metrics.option_height = 51.0;
    theme.select_metrics.trigger.font_size = FontSize::Px(52.0);
    app.insert_resource(GlobalTheme(theme));
    app.update();

    let row = rows(&app, root)[0];
    assert_eq!(app.world().get::<Node>(row).unwrap().height, Val::Px(51.0));
    let font = app.world().get::<TextFont>(row_label(&app, row)).unwrap();
    assert_eq!(font.font_size, FontSize::Px(52.0));
}

/// The probe theme with red rows.
fn red_row_theme() -> Theme {
    let mut theme = probe_theme();
    theme.select.option_bg.unchecked.normal = red();
    theme
}
