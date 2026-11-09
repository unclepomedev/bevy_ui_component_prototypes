//! Tests of the tooltip: when it shows and hides, what it looks like, and where it sits.
//!
//! The real placement against the window edge, and the look on screen, are covered by
//! `examples/tooltip_check.rs`.

use crate::support::{bg, border_color, focus, node, passes_through, test_app};
use accesskit::{Node as AccessKitNode, Role};
use bevy::{
    a11y::AccessibilityNode,
    ecs::entity::EntityHashMap,
    input_focus::{InputFocus, InputFocusVisible},
    picking::{Pickable, backend::HitData, hover::HoverMap, pointer::PointerId},
    prelude::*,
    time::TimeUpdateStrategy,
    ui_widgets::popover::{Popover, PopoverSide},
};
use bevy_ui_component_prototypes::{
    Theme, button::icon_button, text_field::text_field, toggle_switch::toggle_switch,
    tooltip::Tooltip,
};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn spawn_button(app: &mut App, tooltip: Tooltip) -> Entity {
    app.world_mut()
        .spawn((icon_button().label("button").build(), tooltip))
        .id()
}

/// The tooltip of `widget` on screen: its child with a `Popover`.
fn popup(app: &mut App, widget: Entity) -> Option<Entity> {
    let children = app.world().get::<Children>(widget)?.to_vec();
    children
        .into_iter()
        .find(|child| app.world().get::<Popover>(*child).is_some())
}

fn popups_in_world(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), With<Popover>>()
        .iter(app.world())
        .count()
}

fn popup_text(app: &App, popup: Entity) -> String {
    let label = app.world().get::<Children>(popup).unwrap()[0];
    app.world().get::<Text>(label).unwrap().0.clone()
}

fn popup_label(app: &App, popup: Entity) -> Entity {
    app.world().get::<Children>(popup).unwrap()[0]
}

/// The pointer rests on `entities` (nearest first), or on nothing.
fn hover(app: &mut App, entities: &[Entity]) {
    let mut hits = EntityHashMap::default();
    for (i, entity) in entities.iter().enumerate() {
        hits.insert(
            *entity,
            HitData::new(Entity::PLACEHOLDER, i as f32 * 0.001, None, None),
        );
    }
    let mut map = HoverMap::default();
    if !entities.is_empty() {
        map.insert(PointerId::Mouse, hits);
    }
    app.insert_resource(map);
}

// ---------------------------------------------------------------------------
// When it shows
// ---------------------------------------------------------------------------

#[test]
fn nothing_shows_until_the_pointer_rests_on_a_widget() {
    let mut app = timed_app(0.1);
    let button = spawn_button(&mut app, Tooltip::new("Save"));

    run(&mut app, 10);
    assert_eq!(popup(&mut app, button), None);
    assert_eq!(popups_in_world(&mut app), 0);
}

#[test]
fn the_tooltip_shows_after_the_delay_of_the_theme() {
    // The probe theme waits 0.4 seconds.
    let mut app = timed_app(0.1);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    hover(&mut app, &[button]);

    run(&mut app, 3);
    assert_eq!(popup(&mut app, button), None);

    run(&mut app, 2);
    let popup = popup(&mut app, button).expect("the tooltip shows after 0.4 seconds");
    assert_eq!(popup_text(&app, popup), "Save");
    assert_eq!(popups_in_world(&mut app), 1);
}

#[test]
fn the_delay_of_a_tooltip_replaces_the_one_of_the_theme() {
    let mut app = timed_app(0.1);
    let button = spawn_button(&mut app, Tooltip::new("Save").delay(0.0));
    hover(&mut app, &[button]);

    run(&mut app, 1);
    assert!(popup(&mut app, button).is_some());
}

#[test]
fn leaving_the_widget_hides_the_tooltip() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    hover(&mut app, &[button]);
    run(&mut app, 2);
    assert!(popup(&mut app, button).is_some());

    hover(&mut app, &[]);
    run(&mut app, 1);
    assert_eq!(popup(&mut app, button), None);
    assert_eq!(popups_in_world(&mut app), 0);
}

#[test]
fn leaving_before_the_delay_shows_nothing() {
    let mut app = timed_app(0.1);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    hover(&mut app, &[button]);
    run(&mut app, 2);

    hover(&mut app, &[]);
    run(&mut app, 10);
    assert_eq!(popup(&mut app, button), None);
}

#[test]
fn the_child_of_a_widget_stands_for_the_widget() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    let label = app.world().get::<Children>(button).unwrap()[0];
    hover(&mut app, &[label]);

    run(&mut app, 2);
    assert!(popup(&mut app, button).is_some());
}

#[test]
fn the_nearest_tooltip_up_the_tree_wins() {
    let mut app = timed_app(0.5);
    let outer = spawn_button(&mut app, Tooltip::new("outer"));
    let inner = app
        .world_mut()
        .spawn((Node::default(), Tooltip::new("inner"), ChildOf(outer)))
        .id();
    hover(&mut app, &[inner]);

    run(&mut app, 2);
    assert_eq!(popup(&mut app, outer), None);
    let popup = popup(&mut app, inner).expect("the inner widget shows its own");
    assert_eq!(popup_text(&app, popup), "inner");
}

#[test]
fn the_camera_drawn_last_wins_over_a_nearer_depth_below_it() {
    let mut app = timed_app(0.5);
    let below = app.world_mut().spawn(Camera2d).id();
    let above = app
        .world_mut()
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                ..default()
            },
        ))
        .id();
    let back = spawn_button(&mut app, Tooltip::new("back"));
    let front = spawn_button(&mut app, Tooltip::new("front"));

    // The widget of the camera above is further away, but its layer is drawn over the other one.
    let mut hits = EntityHashMap::default();
    hits.insert(back, HitData::new(below, 0.0, None, None));
    hits.insert(front, HitData::new(above, 10.0, None, None));
    let mut map = HoverMap::default();
    map.insert(PointerId::Mouse, hits);
    app.insert_resource(map);

    run(&mut app, 2);
    assert_eq!(popup(&mut app, back), None);
    let popup = popup(&mut app, front).expect("the hit of the camera above shows its tooltip");
    assert_eq!(popup_text(&app, popup), "front");
}

#[test]
fn the_next_widget_shows_at_once_after_one_was_shown() {
    // The probe theme warms up for 0.25 seconds; a step is 0.1.
    let mut app = timed_app(0.1);
    let first = spawn_button(&mut app, Tooltip::new("first").delay(0.0));
    let second = spawn_button(&mut app, Tooltip::new("second"));
    hover(&mut app, &[first]);
    run(&mut app, 2);
    assert!(popup(&mut app, first).is_some());

    hover(&mut app, &[second]);
    run(&mut app, 1);
    assert_eq!(popup(&mut app, first), None);
    assert!(popup(&mut app, second).is_some());
}

#[test]
fn the_next_widget_waits_after_the_warmup_is_over() {
    let mut app = timed_app(0.1);
    let first = spawn_button(&mut app, Tooltip::new("first").delay(0.0));
    let second = spawn_button(&mut app, Tooltip::new("second"));
    hover(&mut app, &[first]);
    run(&mut app, 2);

    hover(&mut app, &[]);
    run(&mut app, 5);
    hover(&mut app, &[second]);
    run(&mut app, 2);
    assert_eq!(popup(&mut app, second), None);
    run(&mut app, 3);
    assert!(popup(&mut app, second).is_some());
}

// ---------------------------------------------------------------------------
// Dismissing
// ---------------------------------------------------------------------------

#[test]
fn a_pointer_press_hides_the_tooltip_until_the_widget_is_left() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    hover(&mut app, &[button]);
    run(&mut app, 2);
    assert!(popup(&mut app, button).is_some());

    let mut mouse = ButtonInput::<MouseButton>::default();
    mouse.press(MouseButton::Left);
    app.insert_resource(mouse);
    run(&mut app, 1);
    assert_eq!(popup(&mut app, button), None);

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    run(&mut app, 4);
    assert_eq!(popup(&mut app, button), None, "it stays away while hovered");

    hover(&mut app, &[]);
    run(&mut app, 1);
    hover(&mut app, &[button]);
    run(&mut app, 4);
    assert!(popup(&mut app, button).is_some());
}

#[test]
fn escape_hides_the_tooltip() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    hover(&mut app, &[button]);
    run(&mut app, 2);
    assert!(popup(&mut app, button).is_some());

    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::Escape);
    app.insert_resource(keys);
    run(&mut app, 1);
    assert_eq!(popup(&mut app, button), None);
}

#[test]
fn despawning_the_widget_takes_the_tooltip_with_it() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    hover(&mut app, &[button]);
    run(&mut app, 2);
    assert!(popup(&mut app, button).is_some());

    app.world_mut().entity_mut(button).despawn();
    hover(&mut app, &[]);
    run(&mut app, 2);
    assert_eq!(popups_in_world(&mut app), 0);

    let other = spawn_button(&mut app, Tooltip::new("Other"));
    hover(&mut app, &[other]);
    run(&mut app, 2);
    assert!(popup(&mut app, other).is_some());
}

// ---------------------------------------------------------------------------
// Keyboard focus
// ---------------------------------------------------------------------------

#[test]
fn a_visible_focus_shows_the_tooltip() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    focus(&mut app, button, true);

    run(&mut app, 2);
    assert!(popup(&mut app, button).is_some());

    app.insert_resource(InputFocus::default());
    run(&mut app, 1);
    assert_eq!(popup(&mut app, button), None);
}

#[test]
fn a_focus_a_click_gave_shows_nothing() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    focus(&mut app, button, false);

    run(&mut app, 4);
    assert_eq!(popup(&mut app, button), None);

    app.insert_resource(InputFocusVisible(true));
    run(&mut app, 2);
    assert!(popup(&mut app, button).is_some());
}

#[test]
fn the_pointer_wins_over_the_focus() {
    let mut app = timed_app(0.5);
    let focused = spawn_button(&mut app, Tooltip::new("focused"));
    let hovered = spawn_button(&mut app, Tooltip::new("hovered"));
    focus(&mut app, focused, true);
    hover(&mut app, &[hovered]);

    run(&mut app, 3);
    assert_eq!(popup(&mut app, focused), None);
    assert!(popup(&mut app, hovered).is_some());
}

// ---------------------------------------------------------------------------
// The tooltip on screen
// ---------------------------------------------------------------------------

fn shown(app: &mut App, tooltip: Tooltip) -> (Entity, Entity) {
    let button = spawn_button(app, tooltip);
    hover(app, &[button]);
    run(app, 2);
    let popup = popup(app, button).expect("the tooltip shows");
    (button, popup)
}

#[test]
fn the_tooltip_takes_its_look_from_the_theme() {
    let mut app = timed_app(0.5);
    let (_, popup) = shown(&mut app, Tooltip::new("Save"));

    assert_eq!(bg(&app, popup), Color::srgb(0.61, 0.11, 0.12));
    assert_eq!(
        border_color(&app, popup),
        BorderColor::all(Color::srgb(0.63, 0.93, 0.94))
    );
    let node = node(&app, popup);
    assert_eq!(node.max_width, Val::Px(171.0));
    assert_eq!(node.padding, UiRect::axes(Val::Px(5.0), Val::Px(3.0)));
    assert_eq!(node.border, UiRect::all(Val::Px(2.0)));
    assert_eq!(node.border_radius, BorderRadius::all(Val::Px(3.0)));
    assert_eq!(node.position_type, PositionType::Absolute);

    let label = popup_label(&app, popup);
    let world = app.world();
    assert_eq!(
        world.get::<TextColor>(label).map(|c| c.0),
        Some(Color::srgb(0.62, 0.52, 0.53))
    );
    assert_eq!(
        world.get::<TextFont>(label).map(|f| f.font_size),
        Some(FontSize::Px(11.0))
    );
}

#[test]
fn a_theme_on_the_widget_or_above_it_changes_the_look() {
    let mut app = timed_app(0.5);
    let mut theme = Theme::default();
    theme.tooltip.bg = Color::srgb(0.2, 0.8, 0.2);
    theme.tooltip_metrics.delay = 0.0;
    let host = app.world_mut().spawn((Node::default(), theme)).id();
    let button = app
        .world_mut()
        .spawn((
            icon_button().label("button").build(),
            Tooltip::new("Save"),
            ChildOf(host),
        ))
        .id();
    hover(&mut app, &[button]);

    run(&mut app, 1);
    let popup = popup(&mut app, button).expect("the theme above has no delay");
    assert_eq!(bg(&app, popup), Color::srgb(0.2, 0.8, 0.2));
}

#[test]
fn changing_the_tooltip_changes_what_is_on_screen() {
    let mut app = timed_app(0.5);
    let (button, popup) = shown(&mut app, Tooltip::new("Save"));

    app.world_mut()
        .entity_mut(button)
        .insert(Tooltip::new("Save as").side(PopoverSide::Right));
    run(&mut app, 1);

    assert_eq!(popup_text(&app, popup), "Save as");
    let popover = app.world().get::<Popover>(popup).unwrap();
    let sides: Vec<_> = popover
        .positions
        .iter()
        .map(|placement| placement.side)
        .collect();
    assert_eq!(sides.first(), Some(&PopoverSide::Right));
    assert!(
        sides
            .iter()
            .all(|side| matches!(side, PopoverSide::Right | PopoverSide::Left))
    );
    assert!(sides.contains(&PopoverSide::Left));
    assert!(
        popover
            .positions
            .iter()
            .all(|placement| placement.gap == 13.0)
    );
}

#[test]
fn the_tooltip_sits_the_gap_of_the_theme_above_the_layer_that_holds_its_widget() {
    let mut app = timed_app(0.5);
    let (_, themed) = shown(&mut app, Tooltip::new("Save"));
    assert_eq!(
        app.world().get::<GlobalZIndex>(themed).map(|z| z.0),
        Some(77)
    );

    let mut app = timed_app(0.5);
    let panel = app
        .world_mut()
        .spawn((Node::default(), GlobalZIndex(500)))
        .id();
    let button = app
        .world_mut()
        .spawn((
            icon_button().label("button").build(),
            Tooltip::new("Save"),
            ChildOf(panel),
        ))
        .id();
    hover(&mut app, &[button]);
    run(&mut app, 2);
    let popup = popup(&mut app, button).unwrap();
    assert_eq!(
        app.world().get::<GlobalZIndex>(popup).map(|z| z.0),
        Some(500 + 77)
    );
}

#[test]
fn the_tooltip_lets_the_pointer_through() {
    let mut app = timed_app(0.5);
    let (_, popup) = shown(&mut app, Tooltip::new("Save"));

    let world = app.world();
    assert!(passes_through(world.get::<Pickable>(popup).unwrap()));
    let label = popup_label(&app, popup);
    assert!(passes_through(app.world().get::<Pickable>(label).unwrap()));
}

// ---------------------------------------------------------------------------
// Other widgets, and a screen reader
// ---------------------------------------------------------------------------

#[test]
fn other_widgets_get_a_tooltip_too() {
    let mut app = timed_app(0.5);
    let switch = app
        .world_mut()
        .spawn((toggle_switch().build(), Tooltip::new("Sound")))
        .id();
    let field = app
        .world_mut()
        .spawn((text_field().build(), Tooltip::new("Name")))
        .id();

    hover(&mut app, &[switch]);
    run(&mut app, 2);
    assert!(popup(&mut app, switch).is_some());

    hover(&mut app, &[field]);
    run(&mut app, 2);
    assert_eq!(popup(&mut app, switch), None);
    let popup = popup(&mut app, field).expect("a text field shows it too");
    assert_eq!(popup_text(&app, popup), "Name");
}

#[test]
fn the_text_is_the_description_of_the_widget_and_the_name_of_the_tooltip() {
    let mut app = timed_app(0.5);
    let (button, popup) = shown(&mut app, Tooltip::new("Save"));

    let world = app.world();
    assert_eq!(
        world
            .get::<AccessibilityNode>(button)
            .unwrap()
            .0
            .description(),
        Some("Save")
    );
    assert_eq!(
        world.get::<AccessibilityNode>(popup).unwrap().0.label(),
        Some("Save")
    );
}

#[test]
fn removing_the_tooltip_takes_its_description_back() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    run(&mut app, 1);
    let description = |app: &App| {
        app.world()
            .get::<AccessibilityNode>(button)
            .unwrap()
            .0
            .description()
            .map(str::to_owned)
    };
    assert_eq!(description(&app), Some("Save".to_owned()));

    app.world_mut().entity_mut(button).remove::<Tooltip>();
    run(&mut app, 1);
    assert_eq!(description(&app), None);
}

#[test]
fn a_description_someone_else_wrote_stays_when_the_tooltip_goes() {
    let mut app = timed_app(0.5);
    let button = spawn_button(&mut app, Tooltip::new("Save"));
    run(&mut app, 1);
    app.world_mut()
        .get_mut::<AccessibilityNode>(button)
        .unwrap()
        .0
        .set_description("Written by the app");

    app.world_mut().entity_mut(button).remove::<Tooltip>();
    run(&mut app, 1);
    let world = app.world();
    assert_eq!(
        world
            .get::<AccessibilityNode>(button)
            .unwrap()
            .0
            .description(),
        Some("Written by the app")
    );
}

#[test]
fn a_node_that_arrives_after_the_tooltip_gets_the_description() {
    let mut app = timed_app(0.5);
    let widget = app
        .world_mut()
        .spawn((Node::default(), Tooltip::new("Late")))
        .id();
    run(&mut app, 2);

    app.world_mut()
        .entity_mut(widget)
        .insert(AccessibilityNode::from(AccessKitNode::new(Role::Button)));
    run(&mut app, 1);
    let world = app.world();
    assert_eq!(
        world
            .get::<AccessibilityNode>(widget)
            .unwrap()
            .0
            .description(),
        Some("Late")
    );
}
