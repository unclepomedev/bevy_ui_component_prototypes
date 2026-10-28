//! Behavior tests for the toast: structure and the pointer.
//!
//! Colors, sizes and the theme come with the style sync; the slide and the real screen are covered by
//! `examples/toast_check.rs`.

use crate::probe::probe_theme;
use bevy::{picking::Pickable, prelude::*, time::TimeUpdateStrategy};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, Toast, ToastHost, ToastStack, ToastVariant, ToastVariantId,
    UiComponentPrototypesPlugin, toast, toast_host,
};
use std::time::Duration;

#[test]
fn a_toast_carries_its_structure_and_starts_unseen() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");

    let world = app.world();
    assert_eq!(
        world.get::<Toast>(toast),
        Some(&Toast {
            elapsed: 0.0,
            duration: None
        })
    );
    assert_eq!(
        world.get::<ToastVariant>(toast),
        Some(&ToastVariant::Default)
    );
    assert!(world.get::<UiTransform>(toast).is_some());
    assert_eq!(world.get::<Children>(toast).map(|c| c.len()), Some(1));
    assert_eq!(text(&app, toast), "Saved");
    // A default `TextColor` is white: the text must not show before the first style sync.
    assert_eq!(label_color(&app, toast), Color::NONE);
    assert_eq!(
        world.get::<BackgroundColor>(toast).map(|c| c.0),
        Some(Color::NONE)
    );
}

#[test]
fn the_builder_options_reach_the_components() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(
            toast()
                .text("Done")
                .variant(ToastVariantId::SUCCESS)
                .duration(2.5)
                .build(),
        )
        .id();

    let world = app.world();
    assert_eq!(world.get::<Toast>(toast).unwrap().duration, Some(2.5));
    assert_eq!(
        world.get::<ToastVariant>(toast),
        Some(&ToastVariant::Named(ToastVariantId::SUCCESS))
    );
}

#[test]
fn the_text_keeps_its_line_breaks_and_wraps() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "first\nsecond");

    assert_eq!(text(&app, toast), "first\nsecond");
    // Unlike a button label, a toast wraps at the width the theme gives it.
    assert_eq!(label_layout(&app, toast).linebreak, LineBreak::WordBoundary);
}

#[test]
fn a_toast_and_its_text_let_the_pointer_through() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");

    let world = app.world();
    assert!(passes_through(world.get::<Pickable>(toast).unwrap()));
    assert!(passes_through(
        world.get::<Pickable>(label(&app, toast)).unwrap()
    ));
}

// ---------------------------------------------------------------------------
// The host
// ---------------------------------------------------------------------------

#[test]
fn a_host_has_the_defaults_and_lets_the_pointer_through() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();

    let world = app.world();
    assert_eq!(
        world.get::<ToastHost>(host),
        Some(&ToastHost {
            max: 5,
            stack: ToastStack::NewestAtBottom,
            slide: Val2::ZERO,
        })
    );
    assert!(world.get::<Node>(host).is_some());
    assert!(passes_through(world.get::<Pickable>(host).unwrap()));
}

#[test]
fn the_host_options_reach_the_component() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn(
            toast_host()
                .max(2)
                .stack(ToastStack::NewestAtTop)
                .slide(Val2::percent(100, 0))
                .build(),
        )
        .id();

    assert_eq!(
        app.world().get::<ToastHost>(host),
        Some(&ToastHost {
            max: 2,
            stack: ToastStack::NewestAtTop,
            slide: Val2::percent(100, 0),
        })
    );
}

// ---------------------------------------------------------------------------
// The stacking direction
// ---------------------------------------------------------------------------

#[test]
fn the_newest_toast_goes_to_the_end_the_stack_names() {
    // The newest toast is the last child.
    for (stack, direction) in [
        (ToastStack::NewestAtBottom, FlexDirection::Column),
        (ToastStack::NewestAtTop, FlexDirection::ColumnReverse),
    ] {
        let mut app = test_app();
        let host = app
            .world_mut()
            .spawn(toast_host().stack(stack).build())
            .id();

        app.update();

        assert_eq!(flex_direction(&app, host), direction, "{stack:?}");
    }
}

#[test]
fn changing_the_stack_changes_the_direction() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();
    app.update();

    app.world_mut().entity_mut(host).insert(ToastHost {
        stack: ToastStack::NewestAtTop,
        ..default()
    });
    app.update();

    assert_eq!(flex_direction(&app, host), FlexDirection::ColumnReverse);
}

// A `Node` spawned together with the host replaces the one the host requires.
#[test]
fn the_apps_node_keeps_its_own_fields_and_gets_the_direction() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn((
            toast_host().stack(ToastStack::NewestAtTop).build(),
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                top: px(24),
                align_items: AlignItems::FlexEnd,
                ..default()
            },
        ))
        .id();

    app.update();

    let node = app.world().get::<Node>(host).unwrap();
    assert_eq!(node.flex_direction, FlexDirection::ColumnReverse);
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.align_items, AlignItems::FlexEnd);
}

#[test]
fn a_node_inserted_later_gets_the_direction_back() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn(toast_host().stack(ToastStack::NewestAtTop).build())
        .id();
    app.update();

    app.world_mut().entity_mut(host).insert(Node::default());
    app.update();

    assert_eq!(flex_direction(&app, host), FlexDirection::ColumnReverse);
}

// ---------------------------------------------------------------------------
// Styling and theme resolution
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// The life of a toast
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app.insert_resource(GlobalTheme(probe_theme()));
    app
}

fn spawn_toast(app: &mut App, text: &str) -> Entity {
    app.world_mut().spawn(toast().text(text).build()).id()
}

fn passes_through(pickable: &Pickable) -> bool {
    !pickable.should_block_lower && !pickable.is_hoverable
}

fn label(app: &App, toast: Entity) -> Entity {
    app.world().get::<Children>(toast).unwrap()[0]
}

fn text(app: &App, toast: Entity) -> String {
    app.world()
        .get::<Text>(label(app, toast))
        .unwrap()
        .0
        .clone()
}

fn label_color(app: &App, toast: Entity) -> Color {
    app.world().get::<TextColor>(label(app, toast)).unwrap().0
}

fn label_layout(app: &App, toast: Entity) -> TextLayout {
    *app.world().get::<TextLayout>(label(app, toast)).unwrap()
}

fn flex_direction(app: &App, host: Entity) -> FlexDirection {
    app.world().get::<Node>(host).unwrap().flex_direction
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

fn red() -> Color {
    Color::srgb(0.9, 0.1, 0.1)
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

fn bg(app: &App, toast: Entity) -> Color {
    app.world().get::<BackgroundColor>(toast).unwrap().0
}

fn border_color(app: &App, toast: Entity) -> BorderColor {
    *app.world().get::<BorderColor>(toast).unwrap()
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

/// An app whose clock moves by `step` seconds every update.
fn timed_app(step: f32) -> App {
    let mut app = test_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        step,
    )));
    // The first update only sets the clock; there is no delta before the second.
    app.update();
    app
}

fn run(app: &mut App, updates: usize) {
    for _ in 0..updates {
        app.update();
    }
}

fn alive(app: &App, entity: Entity) -> bool {
    app.world().get_entity(entity).is_ok()
}

/// The probe theme where a toast is a half second of hold and nothing else.
fn quick_theme() -> Theme {
    let mut theme = probe_theme();
    theme.toast.fade_in = 0.0;
    theme.toast.fade_out = 0.0;
    theme.toast.default_duration = 0.5;
    theme
}

/// Inside the hold of the probe theme (0.5 to 2.5): fully visible.
const HELD: f32 = 1.0;

fn settle(app: &mut App, toast: Entity) -> Entity {
    app.world_mut().entity_mut(toast).insert(Toast {
        elapsed: HELD,
        duration: None,
    });
    toast
}

fn spawn_settled_toast(app: &mut App, text: &str) -> Entity {
    let toast = spawn_toast(app, text);
    settle(app, toast)
}

fn set_elapsed(app: &mut App, toast: Entity, elapsed: f32) {
    app.world_mut().entity_mut(toast).insert(Toast {
        elapsed,
        duration: None,
    });
}

fn translation(app: &App, toast: Entity) -> Val2 {
    app.world().get::<UiTransform>(toast).unwrap().translation
}
