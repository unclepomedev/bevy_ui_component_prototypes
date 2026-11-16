//! Behavior tests for window styling and theme resolution.
//!
//! Real pointer picking and layout are covered by `examples/window_check.rs`.

use crate::{
    probe::probe_theme,
    support::{bg, border_color, focus, node, test_app},
};
use bevy::{
    input_focus::{
        InputFocusVisible,
        tab_navigation::{TabGroup, TabIndex},
    },
    picking::hover::PickingInteraction,
    prelude::*,
};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme,
    window::{
        ActiveWindow, CloseShape, Maximized, TitleAlign, WindowContent, WindowMetrics, WindowRect,
        WindowSide, WindowStyle, window,
    },
};

#[derive(Component)]
struct Marker;

/// A window that is the active one: it has the active look.
fn spawn_window(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((window().title(Text::new("Title")).build(), ActiveWindow))
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

/// The box of the title is the second child of the title bar, and the title is in it.
fn title_box(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(title_bar(app, window)).unwrap()[1]
}

fn title(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(title_box(app, window)).unwrap()[0]
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

#[test]
fn a_window_without_the_active_mark_has_the_inactive_colors() {
    let mut app = test_app();
    let window = app
        .world_mut()
        .spawn(window().title(Text::new("Title")).build())
        .id();

    app.update();

    let theme = probe_theme();
    assert_eq!(
        bg(&app, title_bar(&app, window)),
        theme.window.title_bar.inactive_bg
    );
    assert_eq!(
        app.world().get::<TextColor>(title(&app, window)).unwrap().0,
        theme.window.title_bar.inactive_fg
    );
}

#[test]
fn gaining_and_losing_the_active_mark_swaps_the_colors() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();
    let theme = probe_theme();
    let bar_bg = |app: &App| bg(app, title_bar(app, window));
    let title_fg = |app: &App| app.world().get::<TextColor>(title(app, window)).unwrap().0;
    assert_eq!(bar_bg(&app), theme.window.title_bar.bg);
    assert_eq!(title_fg(&app), theme.window.title_bar.fg);

    app.world_mut().entity_mut(window).remove::<ActiveWindow>();
    app.update();
    assert_eq!(bar_bg(&app), theme.window.title_bar.inactive_bg);
    assert_eq!(title_fg(&app), theme.window.title_bar.inactive_fg);

    app.world_mut().entity_mut(window).insert(ActiveWindow);
    app.update();
    assert_eq!(bar_bg(&app), theme.window.title_bar.bg);
    assert_eq!(title_fg(&app), theme.window.title_bar.fg);
}

#[test]
fn the_title_bar_follows_the_inside_of_the_rounded_frame() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    // The probe theme: a radius of 6 and a border of 3.
    let radius = node(&app, title_bar(&app, window)).border_radius;
    assert_eq!(radius, BorderRadius::top(px(3.0)));

    let mut theme = probe_theme();
    theme.window.shape.radius = 0.0;
    app.insert_resource(GlobalTheme(theme));
    app.update();
    let radius = node(&app, title_bar(&app, window)).border_radius;
    assert_eq!(radius, BorderRadius::top(px(0.0)));
}

// The grips follow the content, in the order of `WindowEdge::ALL`: top, bottom, left, right, top left,
// top right, bottom left, bottom right.
fn grip(app: &App, window: Entity, index: usize) -> Entity {
    app.world().get::<Children>(window).unwrap()[2 + index]
}

#[test]
fn a_window_has_eight_invisible_grips_after_its_content() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    assert_eq!(app.world().get::<Children>(window).unwrap().len(), 2 + 8);
    for index in 0..8 {
        let grip = grip(&app, window, index);
        assert_eq!(bg(&app, grip), Color::NONE);
        assert_eq!(node(&app, grip).position_type, PositionType::Absolute);
    }
}

#[test]
fn the_grips_straddle_the_outer_edge_of_the_frame() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    // The probe theme: a border of 3 and a grip of 11. A strip begins 5.5 outside the frame, so 8.5 from the
    // padding box, and the strips of the sides begin past the corners: 5.5 - 3 = 2.5.
    let left = node(&app, grip(&app, window, 2));
    assert_eq!(left.left, Val::Px(-8.5));
    assert_eq!(left.width, Val::Px(11.0));
    assert_eq!((left.top, left.bottom), (Val::Px(2.5), Val::Px(2.5)));
    assert_eq!(left.right, Val::Auto);

    let bottom = node(&app, grip(&app, window, 1));
    assert_eq!(bottom.bottom, Val::Px(-8.5));
    assert_eq!(bottom.height, Val::Px(11.0));
    assert_eq!((bottom.left, bottom.right), (Val::Px(2.5), Val::Px(2.5)));

    let corner = node(&app, grip(&app, window, 4));
    assert_eq!((corner.left, corner.top), (Val::Px(-8.5), Val::Px(-8.5)));
    assert_eq!(
        (corner.width, corner.height),
        (Val::Px(11.0), Val::Px(11.0))
    );

    let corner = node(&app, grip(&app, window, 7));
    assert_eq!(
        (corner.right, corner.bottom),
        (Val::Px(-8.5), Val::Px(-8.5))
    );
}

#[test]
fn the_grips_follow_the_theme() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    let mut theme = probe_theme();
    theme.window_metrics.resize_grip = 20.0;
    theme.window.shape.border_width = 1.0;
    app.insert_resource(GlobalTheme(theme));
    app.update();

    let right = node(&app, grip(&app, window, 3));
    assert_eq!(right.width, Val::Px(20.0));
    assert_eq!(right.right, Val::Px(-11.0));
    assert_eq!(right.top, Val::Px(9.0));
}

#[test]
fn a_node_given_to_the_content_sets_its_layout_but_it_still_clips_and_fills_the_window() {
    let mut app = test_app();
    let window = app
        .world_mut()
        .spawn(
            window()
                .content_bundle(Node {
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .build(),
        )
        .id();
    app.update();

    let content = node(&app, content(&app, window));
    assert_eq!(content.padding, UiRect::all(Val::Px(8.0)));
    assert_eq!(content.flex_direction, FlexDirection::Row);
    assert_eq!(content.overflow, Overflow::clip());
    assert_eq!(content.flex_grow, 1.0);
    assert_eq!(
        (content.min_width, content.min_height),
        (Val::Px(0.0), Val::Px(0.0))
    );
}

#[test]
fn the_title_is_clipped_by_its_box_and_the_content_can_be_smaller_than_what_is_in_it() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    let title_box = node(&app, title_box(&app, window));
    assert_eq!(title_box.overflow, Overflow::clip_x());
    assert_eq!(title_box.min_width, Val::Px(0.0));
    assert_eq!(title_box.flex_grow, 1.0);
    assert_eq!(node(&app, title(&app, window)).width, Val::Percent(100.0));

    let content = node(&app, content(&app, window));
    assert_eq!(
        (content.min_width, content.min_height),
        (Val::Px(0.0), Val::Px(0.0))
    );
}

// Keyboard.

#[test]
fn the_close_button_is_last_in_the_tab_order() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    let close = close(&app, window);
    assert_eq!(
        app.world().get::<TabIndex>(close),
        Some(&TabIndex(i32::MAX))
    );
}

#[test]
fn each_window_is_a_tab_group() {
    let mut app = test_app();
    let window = spawn_window(&mut app);
    app.update();

    let group = app.world().get::<TabGroup>(window).unwrap();
    assert!(!group.modal);
}

#[test]
fn the_close_button_shows_the_focus_ring_of_the_theme_while_it_shows_focus() {
    let mut app = test_app();
    let theme = probe_theme();
    let ring = theme.window.close.focus_ring;
    app.insert_resource(GlobalTheme(theme));
    let window = spawn_window(&mut app);
    app.update();
    let close = close(&app, window);
    let outline = |app: &App| *app.world().get::<Outline>(close).unwrap();

    assert_eq!(outline(&app).color, Color::NONE);

    focus(&mut app, close, true);
    app.update();
    app.update();
    let shown = outline(&app);
    assert_eq!(
        (shown.color, shown.width, shown.offset),
        (ring.color, ring.width, ring.offset)
    );

    app.insert_resource(InputFocusVisible(false));
    app.update();
    app.update();
    assert_eq!(outline(&app).color, Color::NONE);
}

// The maximize button.

/// A window with a maximize button, active.
fn spawn_maximizable(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            window().title(Text::new("Title")).maximizable().build(),
            ActiveWindow,
        ))
        .id()
}

/// It follows the close button in the title bar, and the title after both.
fn maximize(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(title_bar(app, window)).unwrap()[1]
}

fn maximize_glyph(app: &App, window: Entity) -> (Entity, Entity) {
    let glyph_box = app.world().get::<Children>(maximize(app, window)).unwrap()[0];
    let boxes = app.world().get::<Children>(glyph_box).unwrap();
    (boxes[0], boxes[1])
}

#[test]
fn a_maximize_button_is_only_there_if_asked_for() {
    let mut app = test_app();
    let plain = spawn_window(&mut app);
    let maximizable = spawn_maximizable(&mut app);
    app.update();

    let buttons = |window| {
        app.world()
            .get::<Children>(title_bar(&app, window))
            .unwrap()
            .len()
    };
    // The close button and the title box, and the maximize button between them.
    assert_eq!(buttons(plain), 2);
    assert_eq!(buttons(maximizable), 3);
    // The title is still the last one.
    assert!(
        app.world()
            .get::<Text>(title_of(&app, maximizable))
            .is_some()
    );
}

/// The title is in its box, which is the last child of the title bar.
fn title_of(app: &App, window: Entity) -> Entity {
    let bar = app.world().get::<Children>(title_bar(app, window)).unwrap();
    let title_box = bar[bar.len() - 1];
    app.world().get::<Children>(title_box).unwrap()[0]
}

#[test]
fn the_maximize_button_follows_the_pointer() {
    let mut app = test_app();
    let window = spawn_maximizable(&mut app);
    app.update();
    let button = maximize(&app, window);
    let (front, back) = maximize_glyph(&app, window);
    let style = probe_theme().window.maximize;

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
        app.world_mut().entity_mut(button).insert(state);
        app.update();
        assert_eq!(bg(&app, button), bg_color);
        for part in [front, back] {
            assert_eq!(border_color(&app, part), BorderColor::all(glyph));
        }
    }
    // The close button is not touched by the pointer on the other one.
    assert_eq!(
        bg(&app, close(&app, window)),
        probe_theme().window.close.bg.normal
    );
}

#[test]
fn the_maximize_button_has_the_size_of_the_close_button_and_its_own_corners() {
    let mut app = test_app();
    let window = spawn_maximizable(&mut app);
    app.update();

    let metrics = probe_theme().window_metrics;
    let button = node(&app, maximize(&app, window));
    assert_eq!(button.width, Val::Px(metrics.close_width));
    assert_eq!(button.height, Val::Px(metrics.close_height));
    // The probe theme has a circle for the close button and a square for this one.
    assert_eq!(button.border_radius, BorderRadius::all(Val::Px(0.0)));

    let mut theme = probe_theme();
    theme.window.maximize.shape = CloseShape::Circle;
    app.insert_resource(GlobalTheme(theme));
    app.update();
    assert_eq!(
        node(&app, maximize(&app, window)).border_radius,
        BorderRadius::all(Val::Px(metrics.close_width.min(metrics.close_height) / 2.0))
    );
}

#[test]
fn the_glyph_is_one_box_and_becomes_two_when_the_window_is_maximized() {
    let mut app = test_app();
    let window = spawn_maximizable(&mut app);
    app.update();
    let (front, back) = maximize_glyph(&app, window);
    let size = probe_theme().window_metrics.glyph_size;

    assert_eq!(node(&app, front).width, Val::Px(size));
    assert_eq!(node(&app, front).height, Val::Px(size));
    assert_eq!(node(&app, back).display, Display::None);

    app.world_mut().entity_mut(window).insert(Maximized {
        restore: WindowRect::default(),
    });
    app.update();
    let (front_node, back_node) = (node(&app, front), node(&app, back));
    assert_eq!(back_node.display, Display::Flex);
    let Val::Px(side) = front_node.width else {
        panic!("the side of the box is in pixels");
    };
    assert!(side < size);
    assert_eq!(front_node.height, Val::Px(side));
    // The back box is at the upper right and the front one at the lower left.
    assert_eq!(front_node.top, Val::Px(size - side));
    assert_eq!(back_node.left, Val::Px(size - side));
    assert_eq!(back_node.top, Val::Px(0.0));

    app.world_mut().entity_mut(window).remove::<Maximized>();
    app.update();
    assert_eq!(node(&app, back).display, Display::None);
    assert_eq!(node(&app, front).width, Val::Px(size));
}

#[test]
fn the_grips_are_not_shown_while_the_window_is_maximized() {
    let mut app = test_app();
    let window = spawn_maximizable(&mut app);
    app.update();
    let grips: Vec<Entity> = (0..8).map(|index| grip(&app, window, index)).collect();
    assert!(
        grips
            .iter()
            .all(|g| node(&app, *g).display != Display::None)
    );

    app.world_mut().entity_mut(window).insert(Maximized {
        restore: WindowRect::default(),
    });
    app.update();
    assert!(
        grips
            .iter()
            .all(|g| node(&app, *g).display == Display::None)
    );

    app.world_mut().entity_mut(window).remove::<Maximized>();
    app.update();
    assert!(
        grips
            .iter()
            .all(|g| node(&app, *g).display != Display::None)
    );
}

#[test]
fn the_maximize_button_is_second_to_last_in_the_tab_order() {
    let mut app = test_app();
    let window = spawn_maximizable(&mut app);
    app.update();

    assert_eq!(
        app.world().get::<TabIndex>(maximize(&app, window)),
        Some(&TabIndex(i32::MAX - 1))
    );
}

#[test]
fn the_maximize_button_shows_its_focus_ring_while_it_shows_focus() {
    let mut app = test_app();
    let ring = probe_theme().window.maximize.focus_ring;
    let window = spawn_maximizable(&mut app);
    app.update();
    let button = maximize(&app, window);
    let outline = |app: &App| *app.world().get::<Outline>(button).unwrap();

    assert_eq!(outline(&app).color, Color::NONE);

    focus(&mut app, button, true);
    app.update();
    app.update();
    let shown = outline(&app);
    assert_eq!(
        (shown.color, shown.width, shown.offset),
        (ring.color, ring.width, ring.offset)
    );
}

// A window inside the content of another one.

/// An inner window in the content of an outer one, both with a maximize button.
fn nested_windows(app: &mut App) -> (Entity, Entity) {
    let outer = spawn_maximizable(app);
    let content = content(app, outer);
    let inner = app
        .world_mut()
        .spawn((
            window().maximizable().build(),
            ActiveWindow,
            ChildOf(content),
        ))
        .id();
    app.update();
    (outer, inner)
}

#[test]
fn maximizing_the_outer_window_leaves_the_parts_of_the_inner_one_alone() {
    let mut app = test_app();
    let (outer, inner) = nested_windows(&mut app);
    let size = probe_theme().window_metrics.glyph_size;

    app.world_mut().entity_mut(outer).insert(Maximized {
        restore: WindowRect::default(),
    });
    app.update();

    for index in 0..8 {
        assert_ne!(node(&app, grip(&app, inner, index)).display, Display::None);
    }
    let (front, back) = maximize_glyph(&app, inner);
    assert_eq!(node(&app, front).width, Val::Px(size));
    assert_eq!(node(&app, back).display, Display::None);
}

#[test]
fn the_pointer_on_a_button_of_the_inner_window_does_not_style_the_outer_one() {
    let mut app = test_app();
    let (outer, inner) = nested_windows(&mut app);
    let style = probe_theme().window;

    for button in [close(&app, inner), maximize(&app, inner)] {
        app.world_mut()
            .entity_mut(button)
            .insert(PickingInteraction::Hovered);
    }
    app.update();

    assert_eq!(bg(&app, close(&app, outer)), style.close.bg.normal);
    assert_eq!(bg(&app, maximize(&app, outer)), style.maximize.bg.normal);
    assert_eq!(bg(&app, close(&app, inner)), style.close.bg.hovered);
}
