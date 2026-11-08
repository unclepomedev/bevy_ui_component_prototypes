//! Tests of the life of a toast.

use super::{alive, label, label_color, run, set_elapsed, spawn_toast, timed_app};
use crate::{
    probe::probe_theme,
    support::{bg, border_color},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme,
    toast::{Toast, toast, toast_host},
};

#[test]
fn elapsed_counts_the_seconds_since_the_toast_appeared() {
    let mut app = timed_app(0.25);
    let toast = spawn_toast(&mut app, "Saved");

    run(&mut app, 3);

    assert_eq!(app.world().get::<Toast>(toast).unwrap().elapsed, 0.75);
}

#[test]
fn a_toast_lives_for_the_fades_and_the_duration_of_its_theme() {
    // The probe theme: 0.5 in, 2.0 held, 0.25 out.
    let mut app = timed_app(0.25);
    let toast = spawn_toast(&mut app, "Saved");

    run(&mut app, 10);
    assert!(alive(&app, toast));

    run(&mut app, 1);
    assert!(!alive(&app, toast));
}

#[test]
fn the_duration_of_the_builder_wins_over_the_theme() {
    let mut app = timed_app(0.25);
    let short = app
        .world_mut()
        .spawn(toast().text("Short").duration(1.0).build())
        .id();
    let themed = spawn_toast(&mut app, "Themed");

    run(&mut app, 6);
    assert!(alive(&app, short));
    assert!(alive(&app, themed));

    run(&mut app, 1);
    assert!(!alive(&app, short));
    assert!(alive(&app, themed));
}

#[test]
fn a_toast_set_ahead_is_over_at_once() {
    let mut app = timed_app(0.25);
    let toast = spawn_toast(&mut app, "Saved");

    app.world_mut().entity_mut(toast).insert(Toast {
        elapsed: 100.0,
        duration: None,
    });
    run(&mut app, 1);

    assert!(!alive(&app, toast));
}

#[test]
fn the_theme_of_the_scope_decides_how_long_a_toast_lives() {
    let mut app = timed_app(0.25);
    let panel = app.world_mut().spawn(quick_theme()).id();
    let inside = app
        .world_mut()
        .spawn((toast().text("In").build(), ChildOf(panel)))
        .id();
    let outside = spawn_toast(&mut app, "Out");

    run(&mut app, 2);

    assert!(!alive(&app, inside));
    assert!(alive(&app, outside));
}

#[test]
fn a_toast_goes_on_while_the_game_time_is_paused() {
    let mut app = timed_app(0.25);
    app.world_mut().resource_mut::<Time<Virtual>>().pause();
    let toast = spawn_toast(&mut app, "Saved");

    run(&mut app, 11);

    assert!(!alive(&app, toast));
}

#[test]
fn despawning_a_toast_takes_its_text_with_it() {
    let mut app = timed_app(0.25);
    let toast = spawn_toast(&mut app, "Saved");
    let text = label(&app, toast);

    run(&mut app, 11);

    assert!(!alive(&app, text));
}

#[test]
fn a_toast_fades_in_through_the_alpha_of_every_color() {
    let mut app = timed_app(0.25);
    let toast = spawn_toast(&mut app, "Saved");

    // Half of the fade-in (0.5 seconds in the probe theme).
    run(&mut app, 1);

    let style = probe_theme().toasts.base;
    assert_eq!(bg(&app, toast), style.bg.with_alpha(0.5));
    assert_eq!(
        border_color(&app, toast),
        BorderColor::all(style.border.with_alpha(0.5))
    );
    assert_eq!(label_color(&app, toast), style.fg.with_alpha(0.5));
}

#[test]
fn a_toast_is_invisible_before_it_starts() {
    let mut app = timed_app(0.25);
    let toast = spawn_toast(&mut app, "Saved");
    set_elapsed(&mut app, toast, -0.25);

    run(&mut app, 1);

    assert_eq!(bg(&app, toast).alpha(), 0.0);
    assert_eq!(label_color(&app, toast).alpha(), 0.0);
}

#[test]
fn a_toast_fades_out_through_the_alpha_of_every_color() {
    let mut app = timed_app(0.25);
    let toast = spawn_toast(&mut app, "Saved");
    // One update later it is at 2.625: half of the fade-out (0.25 seconds from 2.5).
    set_elapsed(&mut app, toast, 2.375);

    run(&mut app, 1);

    let style = probe_theme().toasts.base;
    assert_eq!(bg(&app, toast), style.bg.with_alpha(0.5));
    assert_eq!(label_color(&app, toast), style.fg.with_alpha(0.5));
}

#[test]
fn the_fade_keeps_the_alpha_the_theme_gives() {
    let mut app = timed_app(0.25);
    let mut theme = probe_theme();
    theme.toasts.base.bg = Color::srgba(0.2, 0.3, 0.4, 0.5);
    app.insert_resource(GlobalTheme(theme));
    let toast = spawn_toast(&mut app, "Saved");

    run(&mut app, 1);

    assert_eq!(bg(&app, toast), Color::srgba(0.2, 0.3, 0.4, 0.25));
}

#[test]
fn a_toast_slides_in_from_the_start_its_host_gives() {
    let mut app = timed_app(0.25);
    let host = app
        .world_mut()
        .spawn(toast_host().slide(Val2::percent(100, 0)).build())
        .id();
    let toast = app
        .world_mut()
        .spawn((toast().text("Saved").build(), ChildOf(host)))
        .id();

    // Halfway through the fade-in: 1/8 of the way is left (ease-out).
    run(&mut app, 1);
    assert_eq!(translation(&app, toast), Val2::percent(12.5, 0));

    run(&mut app, 1);
    assert_eq!(translation(&app, toast), Val2::percent(0, 0));
}

#[test]
fn a_toast_without_a_host_stays_in_place() {
    let mut app = timed_app(0.25);
    let toast = spawn_toast(&mut app, "Saved");

    run(&mut app, 1);

    assert_eq!(translation(&app, toast), Val2::ZERO);
}

/// The probe theme where a toast is a half second of hold and nothing else.
fn quick_theme() -> Theme {
    let mut theme = probe_theme();
    theme.toast.fade_in = 0.0;
    theme.toast.fade_out = 0.0;
    theme.toast.default_duration = 0.5;
    theme
}

fn translation(app: &App, toast: Entity) -> Val2 {
    app.world().get::<UiTransform>(toast).unwrap().translation
}
