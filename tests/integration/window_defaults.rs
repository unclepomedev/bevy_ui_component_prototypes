//! Behavior tests for `WindowDefaultsPlugin`: the stack of windows, their layers, and `RaiseWindow`.

use crate::support::test_app;
use bevy::{
    camera::NormalizedRenderTarget,
    picking::{
        backend::HitData,
        events::{Pointer, PointerDrag, PointerPress},
        pointer::{Location, PointerButton, PointerId},
    },
    prelude::*,
    ui::ComputedNode,
    ui_widgets::Activate,
};
use bevy_ui_component_prototypes::{
    ActiveWindow, CloseWindow, MoveWindow, RaiseWindow, WindowContent, WindowDefaultsPlugin,
    WindowLimits, WindowRect, WindowStack, window,
};

fn app_with_defaults(plugin: WindowDefaultsPlugin) -> App {
    let mut app = test_app();
    app.add_plugins(plugin);
    app
}

fn spawn_window(app: &mut App) -> Entity {
    app.world_mut().spawn(window().build()).id()
}

fn layer(app: &App, window: Entity) -> Option<i32> {
    app.world().get::<GlobalZIndex>(window).map(|z| z.0)
}

fn is_active(app: &App, window: Entity) -> bool {
    app.world().get::<ActiveWindow>(window).is_some()
}

#[test]
fn windows_get_layers_in_the_order_they_were_spawned() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b, c] = [(); 3].map(|_| spawn_window(&mut app));
    app.update();
    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
    assert_eq!(layer(&app, c), Some(30));
}

#[test]
fn the_base_and_the_step_come_from_the_plugin() {
    let mut app = app_with_defaults(WindowDefaultsPlugin {
        z_base: 100,
        z_step: 5,
    });
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    assert_eq!(layer(&app, a), Some(100));
    assert_eq!(layer(&app, b), Some(105));
}

#[test]
fn only_the_top_window_is_active() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    assert!(!is_active(&app, a));
    assert!(is_active(&app, b));
}

#[test]
fn raising_a_window_brings_it_to_the_top() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b, c] = [(); 3].map(|_| spawn_window(&mut app));
    app.update();

    app.world_mut().trigger(RaiseWindow { window: a });
    app.update();

    assert_eq!(layer(&app, b), Some(10));
    assert_eq!(layer(&app, c), Some(20));
    assert_eq!(layer(&app, a), Some(30));
    assert!(is_active(&app, a));
    assert!(!is_active(&app, c));
}

#[test]
fn raising_the_top_window_changes_nothing() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    let before = app.world().resource::<WindowStack>().clone();

    app.world_mut().trigger(RaiseWindow { window: b });
    app.update();

    assert_eq!(app.world().resource::<WindowStack>(), &before);
    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
}

#[test]
fn raising_an_entity_that_is_not_a_window_is_ignored() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let a = spawn_window(&mut app);
    let stranger = app.world_mut().spawn_empty().id();
    app.update();

    app.world_mut().trigger(RaiseWindow { window: stranger });
    app.update();

    assert_eq!(layer(&app, stranger), None);
    assert_eq!(layer(&app, a), Some(10));
}

#[test]
fn a_despawned_window_leaves_no_gap() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b, c] = [(); 3].map(|_| spawn_window(&mut app));
    app.update();

    app.world_mut().despawn(b);
    app.update();

    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, c), Some(20));
    assert!(!app.world().resource::<WindowStack>().contains(b));
}

#[test]
fn the_active_window_moves_down_when_the_top_one_is_despawned() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();

    app.world_mut().despawn(b);
    app.update();

    assert!(is_active(&app, a));
}

#[test]
fn a_window_spawned_later_goes_on_the_top() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let a = spawn_window(&mut app);
    app.update();

    let b = spawn_window(&mut app);
    app.update();

    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
    assert!(is_active(&app, b));
    assert!(!is_active(&app, a));
}

#[test]
fn without_the_plugin_nothing_is_applied() {
    let mut app = test_app();
    let a = spawn_window(&mut app);
    app.update();

    app.world_mut().trigger(RaiseWindow { window: a });
    app.update();

    assert_eq!(layer(&app, a), None);
    assert!(!is_active(&app, a));
}

/// The pointer presses on `target`.
fn press(app: &mut App, target: Entity) {
    let location = Location {
        target: NormalizedRenderTarget::None {
            width: 1,
            height: 1,
        },
        position: Vec2::ZERO,
    };
    let camera = app.world_mut().spawn_empty().id();
    app.world_mut().trigger(PointerPress {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, location),
        button: PointerButton::Primary,
        hit: HitData::new(camera, 0.0, None, None),
        count: 1,
    });
    app.update();
}

fn content_of(app: &App, window: Entity) -> Entity {
    let children = app.world().get::<Children>(window).unwrap();
    children
        .iter()
        .find(|child| app.world().get::<WindowContent>(*child).is_some())
        .unwrap()
}

#[test]
fn pressing_on_a_window_brings_it_to_the_front() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();

    press(&mut app, a);

    assert_eq!(layer(&app, a), Some(20));
    assert_eq!(layer(&app, b), Some(10));
    assert!(is_active(&app, a));
}

#[test]
fn pressing_on_something_inside_a_window_brings_the_window_to_the_front() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    app.update();
    let content = content_of(&app, a);
    let inside = app.world_mut().spawn(ChildOf(content)).id();

    press(&mut app, inside);

    assert_eq!(layer(&app, a), Some(20));
    assert_eq!(layer(&app, b), Some(10));
}

#[test]
fn pressing_outside_the_windows_changes_nothing() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window(&mut app));
    let outside = app.world_mut().spawn_empty().id();
    app.update();

    press(&mut app, outside);

    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
}

fn rect(app: &App, window: Entity) -> WindowRect {
    *app.world().get::<WindowRect>(window).unwrap()
}

fn title_bar(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(window).unwrap()[0]
}

fn close_button(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(title_bar(app, window)).unwrap()[0]
}

/// A window of 200 x 100 at (100, 100) in a node of 800 x 600.
fn window_in_area(app: &mut App) -> Entity {
    let area = app
        .world_mut()
        .spawn(ComputedNode {
            size: Vec2::new(800.0, 600.0),
            inverse_scale_factor: 1.0,
            ..default()
        })
        .id();
    let window = app
        .world_mut()
        .spawn((
            window()
                .rect(WindowRect::new(100.0, 100.0, 200.0, 100.0))
                .build(),
            WindowLimits {
                visible_width: 50.0,
                visible_height: 30.0,
            },
            ChildOf(area),
        ))
        .id();
    app.update();
    window
}

fn drag(app: &mut App, target: Entity, button: PointerButton, delta: Vec2) {
    let location = Location {
        target: NormalizedRenderTarget::None {
            width: 1,
            height: 1,
        },
        position: Vec2::ZERO,
    };
    app.world_mut().trigger(PointerDrag {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, location),
        button,
        distance: delta,
        delta,
    });
    app.update();
}

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

    assert!(app.world().get::<bevy::ui_widgets::Button>(close).is_some());
}
