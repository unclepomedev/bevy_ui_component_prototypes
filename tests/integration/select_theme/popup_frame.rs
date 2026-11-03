//! Tests of The frame of the popup.

use super::{popup, spawn_fruit, spawn_fruit_under};
use crate::{
    probe::probe_theme,
    support::{bg, passes_through, red, test_app},
};
use bevy::{
    picking::Pickable,
    prelude::*,
    ui::OverrideClip,
    ui_widgets::popover::{Popover, PopoverAlign, PopoverSide},
};
use bevy_ui_component_prototypes::{GlobalTheme, Theme};

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
fn the_popup_is_one_above_the_nearest_ancestor_with_a_layer() {
    let mut app = test_app();
    let outer = app
        .world_mut()
        .spawn((Node::default(), GlobalZIndex(3)))
        .id();
    let panel = app
        .world_mut()
        .spawn((Node::default(), GlobalZIndex(7), ChildOf(outer)))
        .id();
    let root = spawn_fruit_under(&mut app, panel);

    app.update();
    assert_eq!(z_index(&app, popup(&app, root)), 8);

    app.world_mut().entity_mut(panel).insert(GlobalZIndex(20));
    app.update();
    assert_eq!(z_index(&app, popup(&app, root)), 21);
}

#[test]
fn an_ancestor_layer_wins_over_the_theme_value() {
    let mut app = test_app();
    let panel = app
        .world_mut()
        .spawn((Node::default(), GlobalZIndex(7)))
        .id();
    let root = spawn_fruit_under(&mut app, panel);
    app.update();

    let mut theme = probe_theme();
    theme.select_metrics.popup_z_index = 99;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    assert_eq!(z_index(&app, popup(&app, root)), 8);
}

#[test]
fn the_popup_ignores_the_clipping_of_its_ancestors() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    assert!(app.world().get::<OverrideClip>(popup(&app, root)).is_some());
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

/// The probe theme with a red popup.
fn red_popup_theme() -> Theme {
    let mut theme = probe_theme();
    theme.select.popup.bg = red();
    theme
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

fn z_index(app: &App, entity: Entity) -> i32 {
    app.world().get::<GlobalZIndex>(entity).unwrap().0
}

fn scroll_mark(app: &App, root: Entity) -> (Entity, Entity) {
    let world = app.world();
    let track = world.get::<Children>(popup(app, root)).unwrap()[1];
    let thumb = world.get::<Children>(track).unwrap()[0];
    (track, thumb)
}

#[test]
fn the_scroll_mark_is_the_second_child_of_the_popup_and_the_thumb_is_in_it() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let (track, thumb) = scroll_mark(&app, root);
    let world = app.world();
    assert_eq!(world.get::<Children>(popup(&app, root)).unwrap().len(), 2);
    assert_eq!(world.get::<ChildOf>(thumb).unwrap().parent(), track);
}

#[test]
fn the_scroll_mark_lets_the_pointer_through() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let (track, thumb) = scroll_mark(&app, root);
    for part in [track, thumb] {
        assert!(passes_through(app.world().get::<Pickable>(part).unwrap()));
    }
}

#[test]
fn the_scroll_mark_is_hidden_when_there_is_nothing_to_scroll() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    let (track, _) = scroll_mark(&app, root);
    assert_eq!(
        app.world().get::<Node>(track).unwrap().display,
        Display::None
    );
}

fn assert_scroll_mark(app: &App, root: Entity, track_color: Color, thumb_color: Color, width: f32) {
    let (track, thumb) = scroll_mark(app, root);
    assert_eq!(bg(app, track), track_color);
    assert_eq!(bg(app, thumb), thumb_color);
    let node = app.world().get::<Node>(track).unwrap();
    assert_eq!(node.width, Val::Px(width));
}

#[test]
fn the_scroll_mark_takes_the_theme() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    let theme = probe_theme();
    assert_scroll_mark(
        &app,
        root,
        theme.select.scroll_track,
        theme.select.scroll_thumb,
        theme.select_metrics.scroll_indicator_width,
    );
    let (track, _) = scroll_mark(&app, root);
    let node = app.world().get::<Node>(track).unwrap();
    assert_eq!(node.top, theme.select_metrics.popup_padding.top);
    assert_eq!(node.bottom, theme.select_metrics.popup_padding.bottom);
    assert_eq!(node.right, theme.select_metrics.popup_padding.right);
}

#[test]
fn the_scroll_mark_follows_the_global_theme_but_not_a_scoped_one() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let scope = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_fruit_under(&mut app, scope);
    app.update();
    let mut changed = probe_theme();
    changed.select.scroll_track = red();
    changed.select.scroll_thumb = red();
    changed.select_metrics.scroll_indicator_width = 9.0;
    app.insert_resource(GlobalTheme(changed));
    app.update();
    assert_scroll_mark(&app, root, red(), red(), 9.0);
    let theme = probe_theme();
    assert_scroll_mark(
        &app,
        scoped,
        theme.select.scroll_track,
        theme.select.scroll_thumb,
        theme.select_metrics.scroll_indicator_width,
    );
}
