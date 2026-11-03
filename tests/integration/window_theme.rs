//! Behavior tests for window styling and theme resolution.
//!
//! Real pointer picking and layout are covered by `examples/window_check.rs`.

use crate::{
    probe::probe_theme,
    support::{bg, border_color, node, test_app},
};
use bevy::{picking::hover::PickingInteraction, prelude::*};
use bevy_ui_component_prototypes::{
    CloseShape, GlobalTheme, Theme, TitleAlign, WindowContent, WindowMetrics, WindowRect,
    WindowSide, WindowStyle, window,
};

#[derive(Component)]
struct Marker;

fn spawn_window(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(window().title(Text::new("Title")).build())
        .id()
}

// The structure the tests rely on: the title bar and the content, the close button and the title,
// the glyph box, and the two bars of the cross.
fn title_bar(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(window).unwrap()[0]
}

fn content(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(window).unwrap()[1]
}

fn close(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(title_bar(app, window)).unwrap()[0]
}

fn title(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(title_bar(app, window)).unwrap()[1]
}

fn glyph_bars(app: &App, window: Entity) -> Vec<Entity> {
    let glyph_box = app.world().get::<Children>(close(app, window)).unwrap()[0];
    app.world().get::<Children>(glyph_box).unwrap().to_vec()
}

#[test]
fn the_frame_is_styled_after_one_update() {
    let mut app = test_app();
    let window = spawn_window(&mut app);

    app.update();

    let style = probe_theme().window;
    assert_eq!(bg(&app, window), style.bg);
    assert_eq!(border_color(&app, window), BorderColor::all(style.border));
    let frame = node(&app, window);
    assert_eq!(frame.border, UiRect::all(Val::Px(style.shape.border_width)));
    assert_eq!(
        frame.border_radius,
        BorderRadius::all(Val::Px(style.shape.radius))
    );
}

#[test]
fn the_rect_is_written_to_the_node_and_follows_changes() {
    let mut app = test_app();
    let window = app
        .world_mut()
        .spawn(
            window()
                .rect(WindowRect::new(10.0, 20.0, 300.0, 200.0))
                .build(),
        )
        .id();
    app.update();

    let frame = node(&app, window);
    assert_eq!(frame.position_type, PositionType::Absolute);
    assert_eq!(
        (frame.left, frame.top, frame.width, frame.height),
        (px(10), px(20), px(300), px(200))
    );

    app.world_mut()
        .entity_mut(window)
        .insert(WindowRect::new(1.0, 2.0, 3.0, 4.0));
    app.update();
    let frame = node(&app, window);
    assert_eq!(
        (frame.left, frame.top, frame.width, frame.height),
        (px(1), px(2), px(3), px(4))
    );
}

#[test]
fn the_title_bar_takes_its_color_and_size_from_the_theme() {
    let mut app = test_app();
    let window = spawn_window(&mut app);

    app.update();

    let theme = probe_theme();
    let bar = title_bar(&app, window);
    assert_eq!(bg(&app, bar), theme.window.title_bar.bg);
    let bar = node(&app, bar);
    assert_eq!(bar.height, Val::Px(theme.window_metrics.title_height));
    assert_eq!(bar.padding, theme.window_metrics.title_padding);
    assert_eq!(bar.column_gap, Val::Px(theme.window_metrics.title_gap));
}

#[test]
fn the_close_button_is_at_the_side_the_theme_says() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();
    // The probe theme puts it at the left.
    assert_eq!(
        node(&app, title_bar(&app, window)).flex_direction,
        FlexDirection::Row
    );

    let mut theme = probe_theme();
    theme.window.close.side = WindowSide::Right;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    assert_eq!(
        node(&app, title_bar(&app, window)).flex_direction,
        FlexDirection::RowReverse
    );
}

#[test]
fn the_title_takes_its_color_font_size_and_alignment_from_the_theme() {
    let mut app = test_app();
    let window = spawn_window(&mut app);

    app.update();

    let theme = probe_theme();
    let world = app.world();
    let title = title(&app, window);
    assert_eq!(
        world.get::<TextColor>(title).unwrap().0,
        theme.window.title_bar.fg
    );
    let font = world.get::<TextFont>(title).unwrap();
    assert_eq!(font.font_size, theme.window_metrics.title_font_size);
    assert_eq!(font.font, theme.font);
    assert_eq!(
        world.get::<TextLayout>(title).unwrap().justify,
        Justify::Center
    );

    let mut theme = probe_theme();
    theme.window.title_bar.align = TitleAlign::Start;
    app.insert_resource(GlobalTheme(theme));
    app.update();
    assert_eq!(
        app.world().get::<TextLayout>(title).unwrap().justify,
        Justify::Left
    );
}

#[test]
fn the_close_button_and_its_cross_follow_the_pointer() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();
    let close = close(&app, window);
    let bars = glyph_bars(&app, window);
    let style = probe_theme().window.close;

    for (state, bg_color, glyph) in [
        (
            PickingInteraction::None,
            style.bg.normal,
            style.glyph.normal,
        ),
        (
            PickingInteraction::Hovered,
            style.bg.hovered,
            style.glyph.hovered,
        ),
        (
            PickingInteraction::Pressed,
            style.bg.pressed,
            style.glyph.pressed,
        ),
    ] {
        app.world_mut().entity_mut(close).insert(state);
        app.update();
        assert_eq!(bg(&app, close), bg_color);
        assert!(bars.iter().all(|bar| bg(&app, *bar) == glyph));
    }
}

#[test]
fn the_close_button_has_the_size_and_the_corners_the_theme_says() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    let metrics = probe_theme().window_metrics;
    let button = node(&app, close(&app, window));
    assert_eq!(button.width, Val::Px(metrics.close_width));
    assert_eq!(button.height, Val::Px(metrics.close_height));
    // The probe theme has a circle: half of the shorter side.
    assert_eq!(
        button.border_radius,
        BorderRadius::all(Val::Px(metrics.close_width.min(metrics.close_height) / 2.0))
    );

    let mut theme = probe_theme();
    theme.window.close.shape = CloseShape::Square;
    app.insert_resource(GlobalTheme(theme));
    app.update();
    assert_eq!(
        node(&app, close(&app, window)).border_radius,
        BorderRadius::all(Val::Px(0.0))
    );
}

#[test]
fn the_cross_is_two_bars_turned_the_opposite_ways() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    let metrics = probe_theme().window_metrics;
    let bars = glyph_bars(&app, window);
    assert_eq!(bars.len(), 2);
    for bar in &bars {
        let bar = node(&app, *bar);
        assert_eq!(bar.width, Val::Px(metrics.glyph_size));
        assert_eq!(bar.height, Val::Px(metrics.glyph_stroke));
    }
    let turn = |bar: Entity| app.world().get::<UiTransform>(bar).unwrap().rotation;
    assert_eq!(turn(bars[0]), Rot2::degrees(45.0));
    assert_eq!(turn(bars[1]), Rot2::degrees(-45.0));
}

#[test]
fn the_content_area_clips_and_takes_the_bundles_given_to_it() {
    let mut app = test_app();
    let window = app
        .world_mut()
        .spawn(window().content_bundle(Marker).build())
        .id();
    app.update();

    let content = content(&app, window);
    assert!(app.world().get::<Marker>(content).is_some());
    assert!(app.world().get::<WindowContent>(content).is_some());
    assert_eq!(node(&app, content).overflow, Overflow::clip());
    assert!(app.world().get::<Marker>(window).is_none());
}

#[test]
fn the_title_bundle_goes_to_the_title_only() {
    let mut app = test_app();
    let window = app
        .world_mut()
        .spawn(window().title((Text::new("Hint"), Marker)).build())
        .id();
    app.update();

    let title = title(&app, window);
    assert_eq!(app.world().get::<Text>(title).unwrap().0, "Hint");
    assert!(app.world().get::<Marker>(title).is_some());
    assert!(app.world().get::<Marker>(title_bar(&app, window)).is_none());
}

#[test]
fn a_global_theme_change_updates_unscoped_windows_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_window(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = app
        .world_mut()
        .spawn((window().build(), ChildOf(panel)))
        .id();
    app.update();

    let mut swapped = probe_theme();
    swapped.window = WindowStyle::mac();
    swapped.window_metrics = WindowMetrics::mac();
    app.insert_resource(GlobalTheme(swapped));
    app.update();

    assert_eq!(bg(&app, unscoped), WindowStyle::mac().bg);
    assert_eq!(bg(&app, scoped), probe_theme().window.bg);
    assert_eq!(
        node(&app, title_bar(&app, unscoped)).height,
        Val::Px(WindowMetrics::mac().title_height)
    );
}

#[test]
fn the_default_theme_has_a_window_look() {
    let theme = Theme::default();

    assert_eq!(theme.window.close.side, WindowStyle::windows().close.side);
    assert_eq!(theme.window_metrics, WindowMetrics::windows());
}
