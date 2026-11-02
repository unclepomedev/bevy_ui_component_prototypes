//! Tests of Theme resolution.

use super::{shown, spawn_fruit, spawn_fruit_under, trigger};
use crate::{
    probe::probe_theme,
    support::{bg, red, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{GlobalTheme, Theme};

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

/// The probe theme with a red trigger.
fn red_trigger_theme() -> Theme {
    let mut theme = probe_theme();
    theme.select.trigger.bg.normal = red();
    theme
}
