//! Behavior tests for the scroll indicator on its own: when it is shown, where the thumb is, and theme
//! resolution. The sizes the layout would give are written to the scroll area by the tests.
//! The select, which has one in its popup, is covered in `select_theme`.

use crate::{
    probe::probe_theme,
    support::{bg, node, passes_through, red, test_app},
};
use bevy::{
    prelude::*,
    ui::{ComputedNode, ScrollPosition},
    ui_widgets::ScrollArea,
};
use bevy_ui_component_prototypes::{GlobalTheme, scroll_indicator};

/// A node that has a scroll area and an indicator in it, the area first.
struct Spot {
    area: Entity,
    track: Entity,
    thumb: Entity,
}

fn spawn_spot_under(app: &mut App, parent: Option<Entity>) -> Spot {
    let mut wrapper = app.world_mut().spawn(Node::default());
    if let Some(parent) = parent {
        wrapper.insert(ChildOf(parent));
    }
    let wrapper = wrapper.id();
    let area = app
        .world_mut()
        .spawn((
            Node {
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollArea,
            ChildOf(wrapper),
        ))
        .id();
    let track = app
        .world_mut()
        .spawn((scroll_indicator(), ChildOf(wrapper)))
        .id();
    let thumb = app.world().get::<Children>(track).unwrap()[0];
    Spot { area, track, thumb }
}

fn spawn_spot(app: &mut App) -> Spot {
    spawn_spot_under(app, None)
}

/// What the layout would have measured: the area shows `visible` of `content`, scrolled by `offset`.
fn measure(app: &mut App, area: Entity, visible: f32, content: f32, offset: f32) {
    app.world_mut().entity_mut(area).insert((
        ComputedNode {
            size: Vec2::new(100.0, visible),
            content_size: Vec2::new(100.0, content),
            inverse_scale_factor: 1.0,
            ..default()
        },
        ScrollPosition(Vec2::new(0.0, offset)),
    ));
}

fn display(app: &App, track: Entity) -> Display {
    node(app, track).display
}

#[test]
fn nothing_to_scroll_shows_no_track() {
    let mut app = test_app();
    let spot = spawn_spot(&mut app);
    measure(&mut app, spot.area, 100.0, 100.0, 0.0);

    app.update();

    assert_eq!(display(&app, spot.track), Display::None);
}

#[test]
fn content_that_does_not_fit_shows_the_track_and_the_thumb_at_the_start() {
    let mut app = test_app();
    let spot = spawn_spot(&mut app);
    measure(&mut app, spot.area, 100.0, 400.0, 0.0);

    app.update();

    assert_eq!(display(&app, spot.track), Display::Flex);
    let thumb = node(&app, spot.thumb);
    assert_eq!(thumb.top, Val::Px(0.0));
    // The track is the height of the area less the inset of the theme at both ends: 100 - 2 * 3. The thumb
    // is the share that is visible, a quarter of it.
    assert_eq!(thumb.height, Val::Px(23.5));
}

#[test]
fn the_thumb_moves_with_the_scroll_position() {
    let mut app = test_app();
    let spot = spawn_spot(&mut app);
    measure(&mut app, spot.area, 100.0, 400.0, 150.0);

    app.update();

    // Halfway through the 300 that can be scrolled: half of the 70.5 the thumb can move on a track of 94.
    assert_eq!(node(&app, spot.thumb).top, Val::Px(35.25));
}

#[test]
fn at_the_end_the_thumb_ends_where_the_track_ends() {
    let mut app = test_app();
    let spot = spawn_spot(&mut app);
    measure(&mut app, spot.area, 100.0, 400.0, 300.0);

    app.update();

    let thumb = node(&app, spot.thumb);
    let (Val::Px(top), Val::Px(height)) = (thumb.top, thumb.height) else {
        panic!("the thumb is placed in pixels");
    };
    // The track is 94 long.
    assert!((top + height - 94.0).abs() < 1e-4);
}

#[test]
fn the_thumb_follows_a_change_of_the_size() {
    let mut app = test_app();
    let spot = spawn_spot(&mut app);
    measure(&mut app, spot.area, 100.0, 400.0, 0.0);
    app.update();

    measure(&mut app, spot.area, 200.0, 400.0, 0.0);
    app.update();

    // The track is 194 now: half of it is the share that is visible.
    assert_eq!(node(&app, spot.thumb).height, Val::Px(97.0));
    measure(&mut app, spot.area, 400.0, 400.0, 0.0);
    app.update();
    assert_eq!(display(&app, spot.track), Display::None);
}

#[test]
fn an_indicator_without_a_scroll_area_next_to_it_stays_hidden() {
    let mut app = test_app();
    let wrapper = app.world_mut().spawn(Node::default()).id();
    let track = app
        .world_mut()
        .spawn((scroll_indicator(), ChildOf(wrapper)))
        .id();

    app.update();

    assert_eq!(display(&app, track), Display::None);
}

#[test]
fn the_indicator_takes_the_colors_the_size_and_the_place_of_the_theme() {
    let mut app = test_app();
    let spot = spawn_spot(&mut app);
    measure(&mut app, spot.area, 100.0, 400.0, 0.0);

    app.update();

    let theme = probe_theme();
    assert_eq!(bg(&app, spot.track), theme.scroll_indicator.track);
    assert_eq!(bg(&app, spot.thumb), theme.scroll_indicator.thumb);
    let metrics = theme.scroll_indicator_metrics;
    let track = node(&app, spot.track);
    assert_eq!(track.width, Val::Px(metrics.width));
    let inset = Val::Px(metrics.inset);
    assert_eq!(
        (track.top, track.bottom, track.right),
        (inset, inset, inset)
    );
    assert_eq!(track.position_type, PositionType::Absolute);
}

#[test]
fn the_indicator_follows_the_global_theme_but_not_a_scoped_one() {
    let mut app = test_app();
    let spot = spawn_spot(&mut app);
    let scope = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_spot_under(&mut app, Some(scope));
    for spot in [&spot, &scoped] {
        measure(&mut app, spot.area, 100.0, 400.0, 0.0);
    }
    app.update();

    let mut changed = probe_theme();
    changed.scroll_indicator.track = red();
    changed.scroll_indicator.thumb = red();
    changed.scroll_indicator_metrics.width = 9.0;
    app.insert_resource(GlobalTheme(changed));
    app.update();

    assert_eq!(bg(&app, spot.track), red());
    assert_eq!(bg(&app, spot.thumb), red());
    assert_eq!(node(&app, spot.track).width, Val::Px(9.0));
    let theme = probe_theme();
    assert_eq!(bg(&app, scoped.track), theme.scroll_indicator.track);
    assert_eq!(
        node(&app, scoped.track).width,
        Val::Px(theme.scroll_indicator_metrics.width)
    );
}

#[test]
fn the_indicator_lets_the_pointer_through() {
    let mut app = test_app();
    let spot = spawn_spot(&mut app);
    for part in [spot.track, spot.thumb] {
        assert!(passes_through(app.world().get::<Pickable>(part).unwrap()));
    }
}
