//! Moving a window: by the request and by dragging the title bar.

use super::*;

#[test]
fn a_move_request_moves_the_window_by_the_delta() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);

    app.world_mut().trigger(MoveWindow {
        window,
        delta: Vec2::new(30.0, -20.0),
    });
    app.update();

    assert_eq!(
        rect(&app, window),
        WindowRect::new(130.0, 80.0, 200.0, 100.0)
    );
}

#[test]
fn a_move_stops_at_the_limits() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);

    app.world_mut().trigger(MoveWindow {
        window,
        delta: Vec2::new(5000.0, -5000.0),
    });
    app.update();

    // 800 - 50 at the right; the top edge at 0.
    assert_eq!(
        rect(&app, window),
        WindowRect::new(750.0, 0.0, 200.0, 100.0)
    );
}

#[test]
fn a_move_without_a_parent_that_is_laid_out_is_not_limited() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = spawn_window(&mut app);
    app.update();

    app.world_mut().trigger(MoveWindow {
        window,
        delta: Vec2::new(-5000.0, -5000.0),
    });
    app.update();

    assert_eq!(rect(&app, window).x, -5000.0);
}

#[test]
fn a_move_request_is_not_applied_without_the_plugin() {
    let mut app = test_app();
    let window = window_in_area(&mut app);

    app.world_mut().trigger(MoveWindow {
        window,
        delta: Vec2::new(30.0, 0.0),
    });
    app.update();

    assert_eq!(rect(&app, window).x, 100.0);
}

#[test]
fn dragging_the_title_bar_with_the_primary_button_moves_the_window() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);
    let bar = title_bar(&app, window);

    drag(&mut app, bar, PointerButton::Primary, Vec2::new(10.0, 5.0));
    drag(&mut app, bar, PointerButton::Primary, Vec2::new(10.0, 5.0));

    assert_eq!(rect(&app, window).x, 120.0);
    assert_eq!(rect(&app, window).y, 110.0);
}

#[test]
fn dragging_with_another_button_does_not_move_the_window() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);
    let bar = title_bar(&app, window);

    drag(
        &mut app,
        bar,
        PointerButton::Secondary,
        Vec2::new(10.0, 5.0),
    );

    assert_eq!(rect(&app, window).x, 100.0);
}

#[test]
fn a_drag_that_began_on_the_close_button_does_not_move_the_window() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);
    let close = close_button(&app, window);

    drag(
        &mut app,
        close,
        PointerButton::Primary,
        Vec2::new(10.0, 5.0),
    );

    assert_eq!(rect(&app, window).x, 100.0);
}

#[test]
fn a_drag_in_the_content_does_not_move_the_window() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = window_in_area(&mut app);
    let content = content_of(&app, window);

    drag(
        &mut app,
        content,
        PointerButton::Primary,
        Vec2::new(10.0, 5.0),
    );

    assert_eq!(rect(&app, window).x, 100.0);
}

#[test]
fn the_title_bar_drag_is_a_request_the_app_can_answer_itself() {
    #[derive(Resource, Default)]
    struct Asked(Vec<Vec2>);

    let mut app = test_app();
    app.init_resource::<Asked>()
        .add_observer(|mv: On<MoveWindow>, mut asked: ResMut<Asked>| asked.0.push(mv.delta));
    let window = window_in_area(&mut app);
    let bar = title_bar(&app, window);

    drag(&mut app, bar, PointerButton::Primary, Vec2::new(7.0, 3.0));

    assert_eq!(app.world().resource::<Asked>().0, [Vec2::new(7.0, 3.0)]);
    assert_eq!(rect(&app, window).x, 100.0);
}
