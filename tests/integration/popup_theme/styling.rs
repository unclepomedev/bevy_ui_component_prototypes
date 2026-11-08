//! Tests of styling and theme resolution.

use super::{message, panel, spawn_popup};
use crate::{
    probe::probe_theme,
    support::{bg, border_color, red, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::GlobalTheme;

#[test]
fn a_popup_is_styled_after_one_update() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");

    app.update();

    let theme = probe_theme();
    let (style, metrics) = (theme.popup, theme.popup_metrics);
    let panel = panel(&app, root);
    assert_eq!(bg(&app, root), style.scrim);
    assert_eq!(bg(&app, panel), style.bg);
    assert_eq!(border_color(&app, panel), BorderColor::all(style.border));

    let node = app.world().get::<Node>(panel).unwrap();
    assert_eq!(node.min_width, metrics.min_width);
    assert_eq!(node.max_width, metrics.max_width);
    assert_eq!(node.padding, metrics.padding);
    assert_eq!(node.row_gap, metrics.row_gap);
    assert_eq!(node.border, UiRect::all(Val::Px(style.shape.border_width)));
    assert_eq!(
        node.border_radius,
        BorderRadius::all(Val::Px(style.shape.radius))
    );

    let message = message(&app, root);
    let world = app.world();
    assert_eq!(world.get::<TextColor>(message).unwrap().0, style.fg);
    let font = world.get::<TextFont>(message).unwrap();
    assert_eq!(font.font_size, metrics.font_size);
    assert_eq!(font.font, probe_theme().font);
}

#[test]
fn a_theme_on_the_popup_wins_over_the_global_theme() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");
    let mut theme = probe_theme();
    theme.popup.scrim = red();
    theme.popup.fg = red();
    app.world_mut().entity_mut(root).insert(theme);

    app.update();

    assert_eq!(bg(&app, root), red());
    assert_eq!(
        app.world().get::<TextColor>(message(&app, root)).unwrap().0,
        red()
    );
    assert_eq!(bg(&app, panel(&app, root)), probe_theme().popup.bg);
}

#[test]
fn a_change_of_the_global_theme_restyles_the_popup() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Quit?");
    app.update();

    let mut theme = probe_theme();
    theme.popup.bg = red();
    theme.popup_metrics.padding = UiRect::all(Val::Px(3.0));
    app.insert_resource(GlobalTheme(theme));
    app.update();

    let panel = panel(&app, root);
    assert_eq!(bg(&app, panel), red());
    assert_eq!(
        app.world().get::<Node>(panel).unwrap().padding,
        UiRect::all(Val::Px(3.0))
    );
}

#[test]
fn a_popup_under_a_theme_takes_it_over_the_global_one() {
    let mut app = test_app();
    let mut scoped = probe_theme();
    scoped.popup.bg = red();
    let scope = app.world_mut().spawn(scoped).id();
    let root = spawn_popup(&mut app, "Quit?");
    app.world_mut().entity_mut(root).insert(ChildOf(scope));

    app.update();

    assert_eq!(bg(&app, panel(&app, root)), red());
}
