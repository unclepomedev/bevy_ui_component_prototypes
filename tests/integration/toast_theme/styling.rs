//! Tests of styling and theme resolution.

use super::{label, label_color, settle, spawn_toast, text};
use crate::{
    probe::probe_theme,
    support::{bg, border_color, red, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, ToastVariant, ToastVariantId, toast, toast_host,
};

#[test]
fn a_toast_is_styled_after_one_update() {
    let mut app = test_app();
    let toast = spawn_settled_toast(&mut app, "Saved");

    app.update();

    let style = probe_theme().toasts.base;
    assert_eq!(bg(&app, toast), style.bg);
    assert_eq!(border_color(&app, toast), BorderColor::all(style.border));
    assert_eq!(label_color(&app, toast), style.fg);
    let node = app.world().get::<Node>(toast).unwrap();
    assert_eq!(node.border, UiRect::all(Val::Px(style.shape.border_width)));
    assert_eq!(
        node.border_radius,
        BorderRadius::all(Val::Px(style.shape.radius))
    );
}

#[test]
fn a_named_variant_takes_its_style() {
    let mut app = test_app();
    let toast = spawn_success(&mut app);

    app.update();

    let style = probe_theme().toasts.named[&ToastVariantId::SUCCESS];
    assert_eq!(bg(&app, toast), style.bg);
    assert_eq!(label_color(&app, toast), style.fg);
}

#[test]
fn a_name_the_theme_lacks_falls_back_to_the_base_style() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(
            toast()
                .text("Saved")
                .variant(ToastVariantId::new("gold"))
                .build(),
        )
        .id();
    settle(&mut app, toast);

    app.update();

    assert_eq!(bg(&app, toast), probe_theme().toasts.base.bg);
}

#[test]
fn a_toast_without_text_has_an_empty_one_to_fill_in_later() {
    let mut app = test_app();
    let toast = app.world_mut().spawn(toast().build()).id();

    assert_eq!(text(&app, toast), "");
}

#[test]
fn changing_the_variant_of_a_toast_restyles_it() {
    let mut app = test_app();
    let toast = spawn_settled_toast(&mut app, "Saved");
    app.update();

    app.world_mut()
        .entity_mut(toast)
        .insert(ToastVariant::Named(ToastVariantId::DANGER));
    app.update();

    let style = probe_theme().toasts.named[&ToastVariantId::DANGER];
    assert_eq!(bg(&app, toast), style.bg);
    assert_eq!(label_color(&app, toast), style.fg);
}

#[test]
fn global_theme_change_updates_unscoped_toasts_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_success(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_success_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_success_theme()));
    app.update();

    assert_eq!(bg(&app, unscoped), red());
    assert_eq!(bg(&app, scoped), success_bg());
}

#[test]
fn scoped_theme_change_updates_only_toasts_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_success(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_success_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_success_theme();
    app.update();

    assert_eq!(bg(&app, inside), red());
    assert_eq!(bg(&app, outside), success_bg());
}

#[test]
fn toast_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_success_theme()).id();
    app.update();

    let late = spawn_success_under(&mut app, panel);
    app.update();

    assert_eq!(bg(&app, late), red());
}

#[test]
fn the_size_and_the_text_follow_the_theme() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");

    app.update();

    let metrics = probe_theme().toast;
    let node = app.world().get::<Node>(toast).unwrap();
    assert_eq!(node.min_width, metrics.min_width);
    assert_eq!(node.max_width, metrics.max_width);
    assert_eq!(node.padding, metrics.padding);
    let font = label_font(&app, toast);
    assert_eq!(font.font, probe_theme().font);
    assert_eq!(font.font_smoothing, FontSmoothing::AntiAliased);
    assert_eq!(font.font_size, metrics.font_size);
}

#[test]
fn a_theme_change_resizes_the_toast_and_swaps_its_font() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");
    app.update();

    let mut theme = Theme {
        font: FontSource::sans_serif(),
        font_smoothing: FontSmoothing::None,
        ..probe_theme()
    };
    theme.toast.max_width = Val::Px(321.0);
    theme.toast.font_size = FontSize::Px(31.0);
    app.insert_resource(GlobalTheme(theme.clone()));
    app.update();

    assert_eq!(
        app.world().get::<Node>(toast).unwrap().max_width,
        Val::Px(321.0)
    );
    let font = label_font(&app, toast);
    assert_eq!(font.font, theme.font);
    assert_eq!(font.font_smoothing, FontSmoothing::None);
    assert_eq!(font.font_size, FontSize::Px(31.0));
}

#[test]
fn a_host_takes_the_row_gap_of_its_theme() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();

    app.update();

    assert_eq!(row_gap(&app, host), probe_theme().toast.row_gap);
}

#[test]
fn the_row_gap_follows_the_global_theme_but_not_a_scope() {
    let mut app = test_app();
    let unscoped = app.world_mut().spawn(toast_host().build()).id();
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = app
        .world_mut()
        .spawn((toast_host().build(), ChildOf(panel)))
        .id();
    app.update();

    app.insert_resource(GlobalTheme(wide_gap_theme()));
    app.update();

    assert_eq!(row_gap(&app, unscoped), wide_gap());
    assert_eq!(row_gap(&app, scoped), probe_theme().toast.row_gap);
}

#[test]
fn a_scoped_theme_change_moves_the_row_gap_of_the_hosts_inside_it() {
    let mut app = test_app();
    let outside = app.world_mut().spawn(toast_host().build()).id();
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = app
        .world_mut()
        .spawn((toast_host().build(), ChildOf(panel)))
        .id();
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = wide_gap_theme();
    app.update();

    assert_eq!(row_gap(&app, inside), wide_gap());
    assert_eq!(row_gap(&app, outside), probe_theme().toast.row_gap);
}

#[test]
fn a_node_inserted_later_gets_the_row_gap_back() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();
    app.update();

    app.world_mut().entity_mut(host).insert(Node::default());
    app.update();

    assert_eq!(row_gap(&app, host), probe_theme().toast.row_gap);
}

fn spawn_success(app: &mut App) -> Entity {
    let toast = app
        .world_mut()
        .spawn(
            toast()
                .text("Done")
                .variant(ToastVariantId::SUCCESS)
                .build(),
        )
        .id();
    settle(app, toast)
}

fn spawn_success_under(app: &mut App, parent: Entity) -> Entity {
    let toast = app
        .world_mut()
        .spawn((
            toast()
                .text("Done")
                .variant(ToastVariantId::SUCCESS)
                .build(),
            ChildOf(parent),
        ))
        .id();
    settle(app, toast)
}

fn success_bg() -> Color {
    probe_theme().toasts.named[&ToastVariantId::SUCCESS].bg
}

/// The probe theme with a red background for the success toasts.
fn red_success_theme() -> Theme {
    let mut theme = probe_theme();
    theme
        .toasts
        .named
        .get_mut(&ToastVariantId::SUCCESS)
        .unwrap()
        .bg = red();
    theme
}

fn row_gap(app: &App, host: Entity) -> Val {
    app.world().get::<Node>(host).unwrap().row_gap
}

fn wide_gap() -> Val {
    Val::Px(31.0)
}

/// The probe theme with another gap between the toasts.
fn wide_gap_theme() -> Theme {
    let mut theme = probe_theme();
    theme.toast.row_gap = wide_gap();
    theme
}

fn label_font(app: &App, toast: Entity) -> TextFont {
    app.world()
        .get::<TextFont>(label(app, toast))
        .unwrap()
        .clone()
}

fn spawn_settled_toast(app: &mut App, text: &str) -> Entity {
    let toast = spawn_toast(app, text);
    settle(app, toast)
}
