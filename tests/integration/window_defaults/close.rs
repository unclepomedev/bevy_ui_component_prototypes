//! Closing a window: by the request and by the close button.

use super::*;
use bevy::ui_widgets::Button as WidgetButton;

#[test]
fn a_close_request_despawns_the_window_and_the_others_move_down() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b, c] = [(); 3].map(|_| spawn_window(&mut app));
    app.update();

    app.world_mut().trigger(CloseWindow { window: c });
    app.update();

    assert!(app.world().get_entity(c).is_err());
    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
    assert!(is_active(&app, b));
}

#[test]
fn a_close_request_for_something_that_is_not_a_window_is_ignored() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let stranger = app.world_mut().spawn_empty().id();
    app.update();

    app.world_mut().trigger(CloseWindow { window: stranger });
    app.update();

    assert!(app.world().get_entity(stranger).is_ok());
}

#[test]
fn a_close_request_is_not_applied_without_the_plugin() {
    let mut app = test_app();
    let a = spawn_window(&mut app);
    app.update();

    app.world_mut().trigger(CloseWindow { window: a });
    app.update();

    assert!(app.world().get_entity(a).is_ok());
}

#[test]
fn activating_the_close_button_closes_the_window() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    let close = close_button(&app, a);

    app.world_mut().trigger(Activate { entity: close });
    app.update();

    assert!(app.world().get_entity(a).is_err());
    assert!(app.world().get_entity(b).is_ok());
}

#[test]
fn the_close_button_is_a_request_the_app_can_answer_itself() {
    #[derive(Resource, Default)]
    struct Asked(Vec<Entity>);

    let mut app = test_app();
    app.init_resource::<Asked>().add_observer(
        |close: On<CloseWindow>, mut asked: ResMut<Asked>| asked.0.push(close.window),
    );
    let a = spawn_window(&mut app);
    app.update();
    let close = close_button(&app, a);

    app.world_mut().trigger(Activate { entity: close });
    app.update();

    assert_eq!(app.world().resource::<Asked>().0, [a]);
    assert!(app.world().get_entity(a).is_ok());
}

#[test]
fn the_close_button_is_a_button_for_the_assistive_technology() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let a = spawn_window(&mut app);
    app.update();
    let close = close_button(&app, a);

    assert!(app.world().get::<WidgetButton>(close).is_some());
}
