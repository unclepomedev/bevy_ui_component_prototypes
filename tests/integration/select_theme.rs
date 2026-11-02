//! Behavior tests for the select: structure, the label that is shown, the look of the trigger,
//! the popup and its rows, opening and closing, and theme resolution.
//! The pointer and the keys are covered by `examples/select_check.rs`.

use crate::probe::probe_theme;
use accesskit::{HasPopup, NodeId, Role};
use bevy::{
    a11y::AccessibilityNode,
    input_focus::{
        InputFocus, InputFocusVisible,
        tab_navigation::{NavAction, TabIndex},
    },
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{InteractionDisabled, Selected},
    ui_widgets::{
        ActiveDescendant, ListBox, ListItem, MenuAction, MenuButton, MenuEvent, MenuFocusState,
        MenuPopup, ReselectListRow, ScrollArea, ValueChange,
        popover::{Popover, PopoverAlign, PopoverSide},
    },
};
use bevy_ui_component_prototypes::{
    GlobalTheme, Select, SelectedIndex, SelectedOption, Theme, UiComponentPrototypesPlugin, select,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

#[derive(Component, Clone)]
struct Marker(u8);

// ---------------------------------------------------------------------------
// Structure
// ---------------------------------------------------------------------------

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
    assert_eq!(world.get::<Children>(popup).map(|c| c.len()), Some(1));
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

// ---------------------------------------------------------------------------
// The selected row
// ---------------------------------------------------------------------------

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
fn a_position_past_the_last_option_selects_no_row() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(9) });
    app.update();

    assert_eq!(selected_rows(&app, root), [false, false, false]);
}

// ---------------------------------------------------------------------------
// The label that is shown
// ---------------------------------------------------------------------------

#[test]
fn nothing_chosen_shows_the_placeholder() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

#[test]
fn without_a_placeholder_nothing_is_shown() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .option(Fruit::Apple, text("Apple"))
                .build(),
        )
        .id();

    app.update();

    assert_eq!(shown_text(&app, root), None);
}

#[test]
fn the_initial_choice_shows_its_label_after_one_update() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .placeholder(text("Pick one"))
                .options([
                    (Fruit::Apple, text("Apple")),
                    (Fruit::Banana, text("Banana")),
                ])
                .selected(Fruit::Banana)
                .build(),
        )
        .id();

    app.update();

    assert_eq!(index(&app, root), Some(1));
    assert_eq!(shown_text(&app, root), Some("Banana".to_string()));
}

#[test]
fn changing_the_index_replaces_the_label() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    let placeholder = shown(&app, root).unwrap();

    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(2) });
    app.update();

    assert_eq!(shown_text(&app, root), Some("Cherry".to_string()));
    assert!(app.world().get_entity(placeholder).is_err());
    assert_eq!(children_of(&app, content(&app, root)), 1);
}

#[test]
fn labels_of_different_types_can_be_mixed_and_land_on_the_label_entity() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .option(Fruit::Apple, text("Apple"))
                .option(Fruit::Banana, (Text::new("Banana"), Marker(7)))
                .selected(Fruit::Banana)
                .build(),
        )
        .id();

    app.update();

    let label = shown(&app, root).unwrap();
    assert_eq!(app.world().get::<Marker>(label).map(|m| m.0), Some(7));
    // Neither the trigger nor the root gets it.
    assert!(app.world().get::<Marker>(trigger(&app, root)).is_none());
    assert!(app.world().get::<Marker>(root).is_none());
}

#[test]
fn a_value_chooses_its_option_and_removing_it_shows_the_placeholder_again() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Banana,
    });
    app.update();
    assert_eq!(index(&app, root), Some(1));
    assert_eq!(shown_text(&app, root), Some("Banana".to_string()));

    app.world_mut()
        .entity_mut(root)
        .remove::<SelectedOption<Fruit>>();
    app.update();
    assert_eq!(index(&app, root), None);
    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

#[test]
fn a_value_that_is_not_an_option_shows_the_placeholder() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .placeholder(text("Pick one"))
                .option(Fruit::Apple, text("Apple"))
                .build(),
        )
        .id();
    app.update();

    app.world_mut().entity_mut(root).insert(SelectedOption {
        value: Fruit::Cherry,
    });
    app.update();

    assert_eq!(index(&app, root), None);
    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

#[test]
fn a_row_of_another_select_is_not_a_choice_of_this_one() {
    let mut app = app_that_logs_requests();
    let first = spawn_fruit(&mut app);
    let second = spawn_fruit(&mut app);
    let first_trigger = trigger(&app, first);
    let first_list = list(&app, first);
    let foreign_row = rows(&app, second)[1];
    menu_event(&mut app, first_trigger, MenuAction::Toggle);

    report_row(&mut app, first_list, foreign_row);

    assert!(app.world().resource::<Requests>().0.is_empty());
    assert_eq!(visibility(&app, popup(&app, first)), Visibility::Inherited);
}

#[test]
fn a_reselect_of_a_row_of_another_select_does_not_close_this_one() {
    let mut app = test_app();
    let first = spawn_fruit(&mut app);
    let second = spawn_fruit(&mut app);
    let first_trigger = trigger(&app, first);
    let first_list = list(&app, first);
    let foreign_row = rows(&app, second)[0];
    menu_event(&mut app, first_trigger, MenuAction::Toggle);

    app.world_mut().trigger(ReselectListRow {
        source: first_list,
        row: foreign_row,
    });
    app.update();

    assert_eq!(visibility(&app, popup(&app, first)), Visibility::Inherited);
}

#[test]
fn a_preselected_option_is_a_selected_option_like_any_other() {
    let mut app = test_app();
    let root = spawn_preselected(&mut app, Fruit::Banana);

    assert_eq!(
        app.world().get::<SelectedOption<Fruit>>(root),
        Some(&SelectedOption {
            value: Fruit::Banana
        })
    );
    assert_eq!(index(&app, root), Some(1));
}

#[test]
fn a_preselected_select_can_be_cleared_by_removing_the_value() {
    let mut app = test_app();
    let root = spawn_preselected(&mut app, Fruit::Banana);
    app.update();

    app.world_mut()
        .entity_mut(root)
        .remove::<SelectedOption<Fruit>>();
    app.update();

    assert_eq!(index(&app, root), None);
    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

#[test]
fn a_preselected_value_that_is_not_an_option_chooses_nothing() {
    let mut app = test_app();
    let root = app
        .world_mut()
        .spawn(
            select::<Fruit>()
                .placeholder(text("Pick one"))
                .option(Fruit::Apple, text("Apple"))
                .selected(Fruit::Cherry)
                .build(),
        )
        .id();
    app.update();

    assert!(app.world().get::<SelectedOption<Fruit>>(root).is_none());
    assert_eq!(index(&app, root), None);
    assert_eq!(shown_text(&app, root), Some("Pick one".to_string()));
}

// ---------------------------------------------------------------------------
// Styling
// ---------------------------------------------------------------------------

#[test]
fn the_trigger_and_its_label_are_styled_after_one_update() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let style = probe_theme().select.trigger;
    let metrics = probe_theme().select_metrics.trigger;
    let trigger = trigger(&app, root);
    assert_eq!(bg(&app, trigger), style.bg.normal);
    assert_eq!(
        *app.world().get::<BorderColor>(trigger).unwrap(),
        BorderColor::all(style.border.normal)
    );
    let node = app.world().get::<Node>(trigger).unwrap();
    assert_eq!(node.height, metrics.height);
    assert_eq!(node.padding, metrics.padding);
    assert_eq!(node.border, UiRect::all(Val::Px(style.shape.border_width)));

    let label = shown(&app, root).unwrap();
    assert_eq!(label_color(&app, label), style.fg.normal);
    let font = app.world().get::<TextFont>(label).unwrap();
    assert_eq!(font.font, probe_theme().font);
    assert_eq!(font.font_size, metrics.font_size);
}

#[test]
fn the_chevron_takes_the_color_of_the_text() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    assert_eq!(
        *app.world().get::<BorderColor>(arrow(&app, root)).unwrap(),
        BorderColor::all(probe_theme().select.trigger.fg.normal)
    );
}

#[test]
fn a_label_that_replaces_another_is_styled_in_the_same_update() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(0) });
    app.update();

    let label = shown(&app, root).unwrap();
    assert_eq!(
        label_color(&app, label),
        probe_theme().select.trigger.fg.normal
    );
}

#[test]
fn hover_and_press_change_the_colors_of_the_trigger_and_its_text() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let style = probe_theme().select.trigger;
    app.update();
    let trigger = trigger(&app, root);

    for (state, bg_color, fg_color) in [
        (
            PickingInteraction::Hovered,
            style.bg.hovered,
            style.fg.hovered,
        ),
        (
            PickingInteraction::Pressed,
            style.bg.pressed,
            style.fg.pressed,
        ),
        (PickingInteraction::None, style.bg.normal, style.fg.normal),
    ] {
        app.world_mut().entity_mut(trigger).insert(state);
        app.update();
        assert_eq!(bg(&app, trigger), bg_color);
        let label = shown(&app, root).unwrap();
        assert_eq!(label_color(&app, label), fg_color);
    }
}

#[test]
fn interaction_disabled_grays_the_trigger_and_removal_restores_it() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let style = probe_theme().select.trigger;
    app.update();
    let trigger = trigger(&app, root);

    app.world_mut()
        .entity_mut(trigger)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(bg(&app, trigger), style.bg.disabled);
    assert_eq!(
        label_color(&app, shown(&app, root).unwrap()),
        style.fg.disabled
    );

    app.world_mut()
        .entity_mut(trigger)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(bg(&app, trigger), style.bg.normal);
}

#[test]
fn visible_focus_draws_the_ring_on_the_trigger() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    let trigger = trigger(&app, root);

    app.insert_resource(InputFocus::from_entity(trigger));
    app.insert_resource(InputFocusVisible(true));
    app.update();

    let style = probe_theme().select.trigger;
    assert_eq!(bg(&app, trigger), style.bg.focused);
    assert_eq!(outline(&app, trigger).color, style.focus_ring.color);
}

// ---------------------------------------------------------------------------
// The frame of the popup
// ---------------------------------------------------------------------------

#[test]
fn the_popup_is_styled_after_one_update() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let style = probe_theme().select.popup;
    let popup = popup(&app, root);
    assert_eq!(bg(&app, popup), style.bg);
    assert_eq!(
        *app.world().get::<BorderColor>(popup).unwrap(),
        BorderColor::all(style.border)
    );
    let node = app.world().get::<Node>(popup).unwrap();
    assert_eq!(node.border, UiRect::all(Val::Px(style.shape.border_width)));
    assert_eq!(
        node.border_radius,
        BorderRadius::all(Val::Px(style.shape.radius))
    );
    assert_eq!(node.padding, probe_theme().select_metrics.popup_padding);
}

#[test]
fn global_theme_change_updates_unscoped_popups_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_fruit(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_fruit_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_popup_theme()));
    app.update();

    assert_eq!(bg(&app, popup(&app, unscoped)), red());
    assert_eq!(bg(&app, popup(&app, scoped)), probe_theme().select.popup.bg);
}

#[test]
fn scoped_theme_change_updates_only_popups_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_fruit(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_fruit_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_popup_theme();
    app.update();

    assert_eq!(bg(&app, popup(&app, inside)), red());
    assert_eq!(
        bg(&app, popup(&app, outside)),
        probe_theme().select.popup.bg
    );
}

#[test]
fn a_popup_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_popup_theme()).id();
    app.update();

    let late = spawn_fruit_under(&mut app, panel);
    app.update();

    assert_eq!(bg(&app, popup(&app, late)), red());
}

#[test]
fn a_theme_change_reshapes_the_popup() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    let mut theme = probe_theme();
    theme.select.popup.shape.border_width = 11.0;
    theme.select.popup.shape.radius = 12.0;
    theme.select_metrics.popup_padding = UiRect::all(Val::Px(13.0));
    app.insert_resource(GlobalTheme(theme));
    app.update();

    let node = app.world().get::<Node>(popup(&app, root)).unwrap();
    assert_eq!(node.border, UiRect::all(Val::Px(11.0)));
    assert_eq!(node.border_radius, BorderRadius::all(Val::Px(12.0)));
    assert_eq!(node.padding, UiRect::all(Val::Px(13.0)));
}

#[test]
fn the_popup_goes_below_the_select_or_above_it_when_there_is_no_room() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let gap = probe_theme().select_metrics.popup_gap;
    assert_eq!(
        placements(&app, root),
        [
            (PopoverSide::Bottom, PopoverAlign::Start, gap),
            (PopoverSide::Top, PopoverAlign::Start, gap),
        ]
    );
}

#[test]
fn a_theme_change_moves_the_popup_away_from_the_select() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    let mut theme = probe_theme();
    theme.select_metrics.popup_gap = 21.0;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    assert!(
        placements(&app, root)
            .iter()
            .all(|(_, _, gap)| *gap == 21.0)
    );
}

#[test]
fn the_popup_takes_its_place_in_the_stacking_order_from_the_theme() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    let popup = popup(&app, root);
    assert_eq!(z_index(&app, popup), 42);

    let mut theme = probe_theme();
    theme.select_metrics.popup_z_index = 7;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    assert_eq!(z_index(&app, popup), 7);
}

#[test]
fn the_popup_is_as_wide_as_the_select_at_least() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    let popup = popup(&app, root);
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().min_width,
        Val::Percent(100.0)
    );
}

// ---------------------------------------------------------------------------
// The name for screen readers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// The rows of the popup
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Opening and closing
// ---------------------------------------------------------------------------

#[test]
fn toggling_opens_the_popup_and_toggling_again_closes_it() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let popup = popup(&app, root);

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(visibility(&app, popup), Visibility::Inherited);
    assert_eq!(
        focus_state(&app, popup),
        MenuFocusState::Opening(NavAction::First)
    );

    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(visibility(&app, popup), Visibility::Hidden);
    assert_eq!(focus_state(&app, popup), MenuFocusState::Closed);
}

#[test]
fn opening_asks_for_the_end_of_the_list_the_key_names() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);

    menu_event(&mut app, trigger, MenuAction::Open(NavAction::Last));

    assert_eq!(
        focus_state(&app, popup(&app, root)),
        MenuFocusState::Opening(NavAction::Last)
    );
}

#[test]
fn opening_an_open_select_leaves_it_as_it_is() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    menu_event(&mut app, trigger, MenuAction::Open(NavAction::Last));

    menu_event(&mut app, trigger, MenuAction::Open(NavAction::First));

    assert_eq!(
        focus_state(&app, popup(&app, root)),
        MenuFocusState::Opening(NavAction::Last)
    );
}

#[test]
fn close_all_from_the_popup_closes_an_open_select() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let popup = popup(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);
    assert_eq!(visibility(&app, popup), Visibility::Inherited);

    menu_event(&mut app, popup, MenuAction::CloseAll);

    assert_eq!(visibility(&app, popup), Visibility::Hidden);
    assert_eq!(focus_state(&app, popup), MenuFocusState::Closed);
}

#[test]
fn close_all_on_a_closed_select_changes_nothing() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let popup = popup(&app, root);

    menu_event(&mut app, popup, MenuAction::CloseAll);

    assert_eq!(visibility(&app, popup), Visibility::Hidden);
}

#[test]
fn focus_root_puts_the_focus_back_on_the_trigger_and_shows_the_indicator() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let list = list(&app, root);
    app.insert_resource(InputFocus::from_entity(list));
    // A click on the trigger hid it.
    app.insert_resource(InputFocusVisible(false));

    menu_event(&mut app, list, MenuAction::FocusRoot);

    assert_eq!(app.world().resource::<InputFocus>().get(), Some(trigger));
    assert!(app.world().resource::<InputFocusVisible>().0);
}

#[test]
fn without_the_input_focus_plugins_focus_root_does_nothing() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);

    menu_event(&mut app, trigger, MenuAction::FocusRoot);

    assert!(app.world().get_resource::<InputFocus>().is_none());
}

#[derive(Resource, Default)]
struct Seen(Vec<Entity>);

#[test]
fn the_event_stops_at_the_select() {
    let mut app = test_app();
    app.init_resource::<Seen>()
        .add_observer(|event: On<MenuEvent>, mut seen: ResMut<Seen>| {
            seen.0.push(event.event_target());
        });
    let outer = app.world_mut().spawn_empty().id();
    let root = spawn_fruit_under(&mut app, outer);
    let trigger = trigger(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);

    let seen = &app.world().resource::<Seen>().0;
    assert!(seen.contains(&root));
    assert!(!seen.contains(&outer));
}

// ---------------------------------------------------------------------------
// Choosing
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, Fruit)>);

fn app_that_logs_requests() -> App {
    let mut app = test_app();
    app.init_resource::<Requests>().add_observer(
        |change: On<ValueChange<Fruit>>, mut log: ResMut<Requests>| {
            log.0.push((change.source, change.value));
        },
    );
    app
}

fn report_row(app: &mut App, source: Entity, row: Entity) {
    app.world_mut().trigger(ValueChange::<Entity> {
        source,
        value: row,
        is_final: true,
    });
    app.update();
}

#[test]
fn a_row_the_list_reports_becomes_one_request_for_its_value() {
    let mut app = app_that_logs_requests();
    // Inside another entity, so that an event that travels up would be seen more than once.
    let outer = app.world_mut().spawn_empty().id();
    let root = spawn_fruit_under(&mut app, outer);
    let list = list(&app, root);
    let rows = rows(&app, root);

    report_row(&mut app, list, rows[1]);
    report_row(&mut app, list, rows[2]);

    assert_eq!(
        app.world().resource::<Requests>().0,
        [(root, Fruit::Banana), (root, Fruit::Cherry)]
    );
}

#[test]
fn a_request_never_changes_the_choice_by_itself() {
    let mut app = app_that_logs_requests();
    let root = spawn_fruit(&mut app);
    let list = list(&app, root);
    let row = rows(&app, root)[2];

    report_row(&mut app, list, row);

    assert_eq!(index(&app, root), None);
    assert_eq!(selected_rows(&app, root), [false, false, false]);
}

#[test]
fn a_change_that_does_not_come_from_a_list_of_a_select_is_ignored() {
    let mut app = app_that_logs_requests();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let row = rows(&app, root)[0];

    report_row(&mut app, trigger, row);

    assert!(app.world().resource::<Requests>().0.is_empty());
}

#[test]
fn an_entity_that_is_not_a_row_of_the_list_is_ignored() {
    let mut app = app_that_logs_requests();
    let root = spawn_fruit(&mut app);
    let list = list(&app, root);
    let trigger = trigger(&app, root);

    report_row(&mut app, list, trigger);

    assert!(app.world().resource::<Requests>().0.is_empty());
}

#[test]
fn choosing_an_option_closes_the_list_and_gives_the_focus_back_quietly() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let list = list(&app, root);
    let popup = popup(&app, root);
    let row = rows(&app, root)[1];
    menu_event(&mut app, trigger, MenuAction::Toggle);
    app.insert_resource(InputFocus::from_entity(list));
    // A click hid it.
    app.insert_resource(InputFocusVisible(false));

    report_row(&mut app, list, row);

    assert_eq!(visibility(&app, popup), Visibility::Hidden);
    assert_eq!(focus_state(&app, popup), MenuFocusState::Closed);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(trigger));
    assert!(!app.world().resource::<InputFocusVisible>().0);
}

#[test]
fn choosing_the_chosen_option_again_closes_the_list_too() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let list = list(&app, root);
    let popup = popup(&app, root);
    let row = rows(&app, root)[0];
    menu_event(&mut app, trigger, MenuAction::Toggle);
    app.insert_resource(InputFocus::from_entity(list));

    app.world_mut()
        .trigger(ReselectListRow { source: list, row });
    app.update();

    assert_eq!(visibility(&app, popup), Visibility::Hidden);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(trigger));
}

#[test]
fn a_choice_made_while_the_list_is_closed_leaves_the_focus_alone() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let list = list(&app, root);
    let row = rows(&app, root)[0];
    app.insert_resource(InputFocus::from_entity(list));

    report_row(&mut app, list, row);

    assert_eq!(app.world().resource::<InputFocus>().get(), Some(list));
}

// ---------------------------------------------------------------------------
// The highlight under the pointer
// ---------------------------------------------------------------------------

#[test]
fn the_row_the_pointer_comes_to_becomes_the_highlighted_one() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);

    point_at(&mut app, rows[1], PickingInteraction::Hovered);
    assert_eq!(highlighted(&app, root), Some(rows[1]));

    // The pointer goes to another row, and the highlight goes with it.
    point_at(&mut app, rows[1], PickingInteraction::None);
    point_at(&mut app, rows[2], PickingInteraction::Hovered);
    assert_eq!(highlighted(&app, root), Some(rows[2]));
}

#[test]
fn the_highlight_stays_where_it_was_when_the_pointer_leaves() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);
    point_at(&mut app, rows[1], PickingInteraction::Hovered);

    point_at(&mut app, rows[1], PickingInteraction::None);

    assert_eq!(highlighted(&app, root), Some(rows[1]));
}

#[test]
fn a_pressed_row_is_highlighted_too() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);

    point_at(&mut app, rows[0], PickingInteraction::Pressed);

    assert_eq!(highlighted(&app, root), Some(rows[0]));
}

#[test]
fn a_closed_list_does_not_follow_the_pointer() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let rows = rows(&app, root);

    point_at(&mut app, rows[1], PickingInteraction::Hovered);

    assert_eq!(highlighted(&app, root), None);
}

#[test]
fn a_disabled_row_is_not_highlighted_by_the_pointer() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let trigger = trigger(&app, root);
    let rows = rows(&app, root);
    menu_event(&mut app, trigger, MenuAction::Toggle);
    app.world_mut()
        .entity_mut(rows[1])
        .insert(InteractionDisabled);

    point_at(&mut app, rows[1], PickingInteraction::Hovered);

    assert_eq!(highlighted(&app, root), None);
}

// ---------------------------------------------------------------------------
// Theme resolution
// ---------------------------------------------------------------------------

#[test]
fn global_theme_change_updates_unscoped_selects_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_fruit(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_fruit_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_trigger_theme()));
    app.update();

    assert_eq!(bg(&app, trigger(&app, unscoped)), red());
    assert_eq!(
        bg(&app, trigger(&app, scoped)),
        probe_theme().select.trigger.bg.normal
    );
}

#[test]
fn scoped_theme_change_updates_only_selects_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_fruit(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_fruit_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_trigger_theme();
    app.update();

    assert_eq!(bg(&app, trigger(&app, inside)), red());
    assert_eq!(
        bg(&app, trigger(&app, outside)),
        probe_theme().select.trigger.bg.normal
    );
}

#[test]
fn a_select_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_trigger_theme()).id();
    app.update();

    let late = spawn_fruit_under(&mut app, panel);
    app.update();

    assert_eq!(bg(&app, trigger(&app, late)), red());
}

#[test]
fn a_theme_change_swaps_the_font_of_the_label() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.insert_resource(GlobalTheme(Theme {
        font: FontSource::sans_serif(),
        ..probe_theme()
    }));
    app.update();

    let label = shown(&app, root).unwrap();
    assert_eq!(
        app.world().get::<TextFont>(label).unwrap().font,
        FontSource::sans_serif()
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app.insert_resource(GlobalTheme(probe_theme()));
    app
}

fn text(name: &'static str) -> impl Bundle + Clone {
    Text::new(name)
}

fn fruit_select() -> impl Bundle {
    select::<Fruit>()
        .placeholder(text("Pick one"))
        .option(Fruit::Apple, text("Apple"))
        .option(Fruit::Banana, text("Banana"))
        .option(Fruit::Cherry, text("Cherry"))
        .build()
}

fn spawn_fruit(app: &mut App) -> Entity {
    app.world_mut().spawn(fruit_select()).id()
}

fn spawn_preselected(app: &mut App, value: Fruit) -> Entity {
    app.world_mut()
        .spawn(
            select::<Fruit>()
                .placeholder(text("Pick one"))
                .options([
                    (Fruit::Apple, text("Apple")),
                    (Fruit::Banana, text("Banana")),
                    (Fruit::Cherry, text("Cherry")),
                ])
                .selected(value)
                .build(),
        )
        .id()
}

fn spawn_fruit_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((fruit_select(), ChildOf(parent)))
        .id()
}

fn red() -> Color {
    Color::srgb(0.9, 0.1, 0.1)
}

/// The probe theme with a red trigger.
fn red_trigger_theme() -> Theme {
    let mut theme = probe_theme();
    theme.select.trigger.bg.normal = red();
    theme
}

/// The probe theme with a red popup.
fn red_popup_theme() -> Theme {
    let mut theme = probe_theme();
    theme.select.popup.bg = red();
    theme
}

/// The probe theme with red rows.
fn red_row_theme() -> Theme {
    let mut theme = probe_theme();
    theme.select.option_bg.unchecked.normal = red();
    theme
}

fn trigger(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(root).unwrap()[0]
}

fn popup(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(root).unwrap()[1]
}

fn list(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(popup(app, root)).unwrap()[0]
}

fn placements(app: &App, root: Entity) -> Vec<(PopoverSide, PopoverAlign, f32)> {
    app.world()
        .get::<Popover>(popup(app, root))
        .unwrap()
        .positions
        .iter()
        .map(|placement| (placement.side, placement.align, placement.gap))
        .collect()
}

fn rows(app: &App, root: Entity) -> Vec<Entity> {
    app.world()
        .get::<Children>(list(app, root))
        .unwrap()
        .to_vec()
}

fn selected_rows(app: &App, root: Entity) -> Vec<bool> {
    rows(app, root)
        .iter()
        .map(|row| app.world().get::<Selected>(*row).is_some())
        .collect()
}

fn row_label(app: &App, row: Entity) -> Entity {
    app.world().get::<Children>(row).unwrap()[0]
}

fn content(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(trigger(app, root)).unwrap()[0]
}

fn arrow(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(trigger(app, root)).unwrap()[1]
}

fn children_of(app: &App, entity: Entity) -> usize {
    app.world().get::<Children>(entity).map_or(0, |c| c.len())
}

/// The label that is shown now: the chosen option's, or the placeholder.
fn shown(app: &App, root: Entity) -> Option<Entity> {
    app.world()
        .get::<Children>(content(app, root))
        .and_then(|children| children.first().copied())
}

fn shown_text(app: &App, root: Entity) -> Option<String> {
    shown(app, root).map(|label| app.world().get::<Text>(label).unwrap().0.clone())
}

fn popup_node(app: &App, root: Entity) -> &AccessibilityNode {
    app.world()
        .get::<AccessibilityNode>(popup(app, root))
        .unwrap()
}

fn trigger_node(app: &App, root: Entity) -> &AccessibilityNode {
    app.world()
        .get::<AccessibilityNode>(trigger(app, root))
        .unwrap()
}

fn trigger_name(app: &App, root: Entity) -> Option<String> {
    app.world()
        .get::<AccessibilityNode>(trigger(app, root))
        .unwrap()
        .label()
        .map(str::to_owned)
}

fn index(app: &App, root: Entity) -> Option<usize> {
    app.world().get::<SelectedIndex>(root).unwrap().index
}

fn bg(app: &App, entity: Entity) -> Color {
    app.world().get::<BackgroundColor>(entity).unwrap().0
}

fn outline(app: &App, entity: Entity) -> Outline {
    *app.world().get::<Outline>(entity).unwrap()
}

fn label_color(app: &App, label: Entity) -> Color {
    app.world().get::<TextColor>(label).unwrap().0
}

fn point_at(app: &mut App, row: Entity, interaction: PickingInteraction) {
    app.world_mut().entity_mut(row).insert(interaction);
    app.update();
}

fn highlighted(app: &App, root: Entity) -> Option<Entity> {
    app.world()
        .get::<ActiveDescendant>(list(app, root))
        .unwrap()
        .0
}

fn menu_event(app: &mut App, source: Entity, action: MenuAction) {
    app.world_mut().trigger(MenuEvent { source, action });
    app.update();
}

fn visibility(app: &App, entity: Entity) -> Visibility {
    *app.world().get::<Visibility>(entity).unwrap()
}

fn focus_state(app: &App, popup: Entity) -> MenuFocusState {
    app.world().get::<MenuFocusState>(popup).unwrap().clone()
}

fn z_index(app: &App, entity: Entity) -> i32 {
    app.world().get::<GlobalZIndex>(entity).unwrap().0
}
