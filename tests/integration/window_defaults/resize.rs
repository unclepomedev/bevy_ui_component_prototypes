//! Resizing a window: by the request and by dragging a grip.

use super::*;

/// The order of the grips among the children of a window, after the title bar and the content.
const GRIPS: [WindowEdge; 8] = [
    WindowEdge::Top,
    WindowEdge::Bottom,
    WindowEdge::Left,
    WindowEdge::Right,
    WindowEdge::TopLeft,
    WindowEdge::TopRight,
    WindowEdge::BottomLeft,
    WindowEdge::BottomRight,
];

fn grip(app: &App, window: Entity, edge: WindowEdge) -> Entity {
    let index = GRIPS.iter().position(|e| *e == edge).unwrap();
    app.world().get::<Children>(window).unwrap()[2 + index]
}

#[test]
fn a_resize_request_moves_the_edge_by_the_delta() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);

    app.world_mut().trigger(ResizeWindow {
        window,
        edge: WindowEdge::BottomRight,
        delta: Vec2::new(30.0, 20.0),
    });
    app.update();

    assert_eq!(
        rect(&app, window),
        WindowRect::new(100.0, 100.0, 230.0, 120.0)
    );
}

#[test]
fn a_resize_stops_at_the_minimum_size() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);

    app.world_mut().trigger(ResizeWindow {
        window,
        edge: WindowEdge::Left,
        delta: Vec2::new(5000.0, 0.0),
    });
    app.update();

    // The right edge was at 300, and the window may not be narrower than 80.
    assert_eq!(
        rect(&app, window),
        WindowRect::new(220.0, 100.0, 80.0, 100.0)
    );
}

#[test]
fn a_resize_request_is_not_applied_without_the_plugin() {
    let mut app = test_app();
    let window = window_in_area(&mut app);

    app.world_mut().trigger(ResizeWindow {
        window,
        edge: WindowEdge::Right,
        delta: Vec2::new(30.0, 0.0),
    });
    app.update();

    assert_eq!(rect(&app, window).width, 200.0);
}

#[test]
fn dragging_a_grip_with_the_primary_button_resizes_the_window() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);

    let right = grip(&app, window, WindowEdge::Right);
    drag(
        &mut app,
        right,
        PointerButton::Primary,
        Vec2::new(10.0, 99.0),
    );
    let top_left = grip(&app, window, WindowEdge::TopLeft);
    drag(
        &mut app,
        top_left,
        PointerButton::Primary,
        Vec2::new(-10.0, -5.0),
    );

    assert_eq!(
        rect(&app, window),
        WindowRect::new(90.0, 95.0, 220.0, 105.0)
    );
}

#[test]
fn dragging_a_grip_with_another_button_does_not_resize_the_window() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);

    let right = grip(&app, window, WindowEdge::Right);
    drag(
        &mut app,
        right,
        PointerButton::Secondary,
        Vec2::new(10.0, 0.0),
    );

    assert_eq!(rect(&app, window).width, 200.0);
}

#[test]
fn dragging_a_grip_does_not_move_the_window() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);

    let bottom = grip(&app, window, WindowEdge::Bottom);
    drag(
        &mut app,
        bottom,
        PointerButton::Primary,
        Vec2::new(40.0, 10.0),
    );

    let rect = rect(&app, window);
    assert_eq!((rect.x, rect.y), (100.0, 100.0));
}

#[test]
fn the_grip_drag_is_a_request_the_app_can_answer_itself() {
    #[derive(Resource, Default)]
    struct Asked(Vec<(WindowEdge, Vec2)>);

    let mut app = test_app();
    app.init_resource::<Asked>().add_observer(
        |resize: On<ResizeWindow>, mut asked: ResMut<Asked>| {
            asked.0.push((resize.edge, resize.delta))
        },
    );
    let window = window_in_area(&mut app);

    let left = grip(&app, window, WindowEdge::Left);
    drag(&mut app, left, PointerButton::Primary, Vec2::new(7.0, 3.0));

    assert_eq!(
        app.world().resource::<Asked>().0,
        [(WindowEdge::Left, Vec2::new(7.0, 3.0))]
    );
    assert_eq!(rect(&app, window).x, 100.0);
}

#[test]
fn pressing_on_a_grip_brings_the_window_to_the_front() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();

    let grip = grip(&app, a, WindowEdge::Top);
    press(&mut app, grip);

    assert_eq!(layer(&app, a), Some(20));
    assert_eq!(layer(&app, b), Some(10));
}
