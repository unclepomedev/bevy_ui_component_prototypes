//! Maximizing a window: by the request and by the maximize button, and going back.

use super::*;
use bevy_ui_component_prototypes::window::{Maximizable, Maximized, ToggleMaximizeWindow};

const AREA: Vec2 = Vec2::new(800.0, 600.0);
const START: WindowRect = WindowRect::new(100.0, 100.0, 200.0, 100.0);

fn limits() -> WindowLimits {
    WindowLimits {
        visible_width: 50.0,
        visible_height: 30.0,
        min_width: 80.0,
        min_height: 40.0,
    }
}

/// A node of 800 x 600, and the area is its size.
fn spawn_area(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(ComputedNode {
            size: AREA,
            inverse_scale_factor: 1.0,
            ..default()
        })
        .id()
}

fn set_area(app: &mut App, area: Entity, size: Vec2) {
    app.world_mut().get_mut::<ComputedNode>(area).unwrap().size = size;
}

/// A window of 200 x 100 at (100, 100) in `area`, built with or without `maximizable`.
fn spawn_in(app: &mut App, area: Entity, maximizable: bool) -> Entity {
    let builder = window().rect(START);
    let builder = if maximizable {
        builder.maximizable()
    } else {
        builder
    };
    app.world_mut()
        .spawn((builder.build(), limits(), ChildOf(area)))
        .id()
}

fn maximizable_in_area(app: &mut App) -> (Entity, Entity) {
    let area = spawn_area(app);
    let window = spawn_in(app, area, true);
    app.update();
    (area, window)
}

fn toggle(app: &mut App, window: Entity) {
    app.world_mut().trigger(ToggleMaximizeWindow { window });
    app.update();
}

fn is_maximized(app: &App, window: Entity) -> bool {
    app.world().get::<Maximized>(window).is_some()
}

/// The maximize button is the second child of the title bar, after the close button.
fn maximize_button(app: &App, window: Entity) -> Entity {
    app.world().get::<Children>(title_bar(app, window)).unwrap()[1]
}

#[test]
fn a_window_is_not_maximizable_unless_it_is_built_so() {
    let mut app = test_app();
    let plain = spawn_window(&mut app);
    let maximizable = app.world_mut().spawn(window().maximizable().build()).id();

    assert_eq!(
        app.world().get::<Maximizable>(plain),
        Some(&Maximizable(false))
    );
    assert_eq!(
        app.world().get::<Maximizable>(maximizable),
        Some(&Maximizable(true))
    );
}

#[test]
fn a_maximize_request_fills_the_node_and_keeps_where_the_window_was() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (_, window) = maximizable_in_area(&mut app);

    toggle(&mut app, window);

    assert_eq!(rect(&app, window), WindowRect::new(0.0, 0.0, 800.0, 600.0));
    assert_eq!(
        app.world().get::<Maximized>(window),
        Some(&Maximized { restore: START })
    );
}

#[test]
fn a_second_request_sets_the_window_back_exactly() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (_, window) = maximizable_in_area(&mut app);

    toggle(&mut app, window);
    toggle(&mut app, window);

    assert_eq!(rect(&app, window), START);
    assert!(!is_maximized(&app, window));
}

#[test]
fn a_window_goes_back_to_the_size_it_was_resized_to() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (_, window) = maximizable_in_area(&mut app);
    app.world_mut().trigger(ResizeWindow {
        window,
        edge: WindowEdge::BottomRight,
        delta: Vec2::new(50.0, 40.0),
    });
    app.update();
    let before = rect(&app, window);

    toggle(&mut app, window);
    toggle(&mut app, window);

    assert_eq!(rect(&app, window), before);
}

#[test]
fn a_window_that_is_not_maximizable_ignores_the_request() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let area = spawn_area(&mut app);
    let window = spawn_in(&mut app, area, false);
    app.update();

    toggle(&mut app, window);

    assert_eq!(rect(&app, window), START);
    assert!(!is_maximized(&app, window));
}

#[test]
fn a_window_without_a_node_to_fill_is_not_maximized() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let window = app
        .world_mut()
        .spawn((window().rect(START).maximizable().build(), limits()))
        .id();
    app.update();

    toggle(&mut app, window);

    assert_eq!(rect(&app, window), START);
    assert!(!is_maximized(&app, window));
}

#[test]
fn a_request_for_something_that_is_not_a_window_is_ignored() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let stranger = app.world_mut().spawn_empty().id();
    app.update();

    toggle(&mut app, stranger);

    assert!(!is_maximized(&app, stranger));
}

#[test]
fn a_maximize_request_is_not_applied_without_the_plugin() {
    let mut app = test_app();
    let (_, window) = maximizable_in_area(&mut app);

    toggle(&mut app, window);

    assert_eq!(rect(&app, window), START);
    assert!(!is_maximized(&app, window));
}

#[test]
fn a_maximized_window_is_not_moved_or_resized() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (_, window) = maximizable_in_area(&mut app);
    toggle(&mut app, window);
    let full = rect(&app, window);

    app.world_mut().trigger(MoveWindow {
        window,
        delta: Vec2::new(30.0, 30.0),
    });
    app.world_mut().trigger(ResizeWindow {
        window,
        edge: WindowEdge::Left,
        delta: Vec2::new(30.0, 0.0),
    });
    app.update();
    let bar = title_bar(&app, window);
    drag(&mut app, bar, PointerButton::Primary, Vec2::new(30.0, 30.0));

    assert_eq!(rect(&app, window), full);
}

#[test]
fn a_window_that_went_back_can_be_moved_again() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (_, window) = maximizable_in_area(&mut app);
    toggle(&mut app, window);
    toggle(&mut app, window);

    app.world_mut().trigger(MoveWindow {
        window,
        delta: Vec2::new(30.0, 20.0),
    });
    app.update();

    assert_eq!(
        rect(&app, window),
        WindowRect::new(130.0, 120.0, 200.0, 100.0)
    );
}

#[test]
fn a_maximized_window_follows_the_size_of_its_node() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (area, window) = maximizable_in_area(&mut app);
    toggle(&mut app, window);

    set_area(&mut app, area, Vec2::new(1024.0, 768.0));
    app.update();

    assert_eq!(rect(&app, window), WindowRect::new(0.0, 0.0, 1024.0, 768.0));
}

#[test]
fn a_maximized_window_keeps_the_minimum_size() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (area, window) = maximizable_in_area(&mut app);
    toggle(&mut app, window);

    set_area(&mut app, area, Vec2::new(60.0, 30.0));
    app.update();

    assert_eq!(rect(&app, window), WindowRect::new(0.0, 0.0, 80.0, 40.0));
}

#[test]
fn a_window_goes_back_into_a_node_that_became_smaller() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (area, window) = maximizable_in_area(&mut app);
    toggle(&mut app, window);

    set_area(&mut app, area, Vec2::new(150.0, 120.0));
    app.update();
    toggle(&mut app, window);

    // The same size, with its top edge at the visible height above the bottom of the node.
    let back = rect(&app, window);
    assert_eq!((back.width, back.height), (START.width, START.height));
    assert_eq!(back.y, 90.0);
    assert!(back.x <= 100.0);
}

#[test]
fn the_maximize_button_asks_for_the_window_to_be_maximized() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (_, window) = maximizable_in_area(&mut app);
    let button = maximize_button(&app, window);

    app.world_mut().trigger(Activate { entity: button });
    app.update();
    assert!(is_maximized(&app, window));

    app.world_mut().trigger(Activate { entity: button });
    app.update();
    assert!(!is_maximized(&app, window));
    assert_eq!(rect(&app, window), START);
}

#[test]
fn the_maximize_button_is_a_request_the_app_can_answer_itself() {
    #[derive(Resource, Default)]
    struct Asked(Vec<Entity>);

    let mut app = test_app();
    app.init_resource::<Asked>().add_observer(
        |toggle: On<ToggleMaximizeWindow>, mut asked: ResMut<Asked>| asked.0.push(toggle.window),
    );
    let (_, window) = maximizable_in_area(&mut app);
    let button = maximize_button(&app, window);

    app.world_mut().trigger(Activate { entity: button });
    app.update();

    assert_eq!(app.world().resource::<Asked>().0, [window]);
    assert!(!is_maximized(&app, window));
}

#[test]
fn a_maximized_window_can_be_closed() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let (_, window) = maximizable_in_area(&mut app);
    toggle(&mut app, window);

    app.world_mut().trigger(CloseWindow { window });
    app.update();

    assert!(app.world().get_entity(window).is_err());
}

// Layers.

fn with_maximized_z(z: Option<i32>) -> WindowDefaultsPlugin {
    WindowDefaultsPlugin {
        maximized_z: z,
        ..WindowDefaultsPlugin::default()
    }
}

#[test]
fn without_a_maximized_layer_the_place_in_the_stack_stays() {
    let mut app = app_with_defaults(with_maximized_z(None));
    let area = spawn_area(&mut app);
    let [a, b] = [(); 2].map(|_| spawn_in(&mut app, area, true));
    app.update();

    toggle(&mut app, a);

    assert_eq!(layer(&app, a), Some(10));
    assert_eq!(layer(&app, b), Some(20));
}

#[test]
fn a_maximized_window_goes_to_the_layer_it_was_given_and_back() {
    let mut app = app_with_defaults(with_maximized_z(Some(500)));
    let area = spawn_area(&mut app);
    let [a, b] = [(); 2].map(|_| spawn_in(&mut app, area, true));
    app.update();

    toggle(&mut app, a);
    assert_eq!(layer(&app, a), Some(500));
    // The one that is not maximized is in front of it in the stack, and behind it in the layers.
    assert_eq!(layer(&app, b), Some(20));
    assert!(is_active(&app, b));

    toggle(&mut app, a);
    assert_eq!(layer(&app, a), Some(10));
}

#[test]
fn the_maximized_layers_follow_the_stack_a_step_apart() {
    let mut app = app_with_defaults(with_maximized_z(Some(500)));
    let area = spawn_area(&mut app);
    let [a, b, c] = [(); 3].map(|_| spawn_in(&mut app, area, true));
    app.update();

    toggle(&mut app, c);
    toggle(&mut app, a);

    assert_eq!(layer(&app, a), Some(500));
    assert_eq!(layer(&app, c), Some(510));
    assert_eq!(layer(&app, b), Some(20));
}

#[test]
fn a_maximized_window_goes_back_even_if_it_is_no_longer_maximizable() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let area = spawn_area(&mut app);
    let window = spawn_in(&mut app, area, false);
    app.world_mut().entity_mut(window).insert((
        Maximized { restore: START },
        WindowRect::new(0.0, 0.0, 800.0, 600.0),
    ));
    app.update();

    toggle(&mut app, window);

    assert_eq!(rect(&app, window), START);
    assert!(!is_maximized(&app, window));
}
