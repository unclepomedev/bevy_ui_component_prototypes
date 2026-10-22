//! Behavior tests for the slider: styling, the position of the fill and the thumb, theme resolution,
//! focus, tab order, and the controlled-value wiring on top of Bevy's `Slider`.
//!
//! Real pointer picking, layout, and dragging are covered by `examples/slider_check.rs`.

use accesskit::Role;
use bevy::{
    a11y::AccessibilityNode,
    input_focus::{InputFocus, InputFocusVisible, tab_navigation::TabIndex},
    picking::hover::Hovered,
    prelude::*,
    ui::{InteractionDisabled, Pressed},
    ui_widgets::{
        SetSliderValue, Slider, SliderDragState, SliderOrientation, SliderPlugin, SliderRange,
        SliderStep, SliderValue, SliderValueChange, TrackClick, ValueChange,
    },
};
use bevy_ui_component_prototypes::{
    GlobalTheme, SliderEditing, SliderSegments, SliderShape, StateColors, Theme,
    UiComponentPrototypesPlugin, slider,
};

// ---------------------------------------------------------------------------
// Behavior wiring
// ---------------------------------------------------------------------------

#[test]
fn slider_carries_the_bevy_slider_behavior_components() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);

    let world = app.world();
    let behavior = world.get::<Slider>(slider).unwrap();
    // Clicking the rail has to move the value there, and the width must not decide the orientation.
    assert_eq!(behavior.track_click, TrackClick::Snap);
    assert_eq!(behavior.orientation, SliderOrientation::Horizontal);
    assert_eq!(
        world
            .get::<AccessibilityNode>(slider)
            .map(|node| node.role()),
        Some(Role::Slider)
    );
    assert!(world.get::<Hovered>(slider).is_some());
    assert_eq!(world.get::<TabIndex>(slider).map(|t| t.0), Some(0));
    // `Outline::default()` would be a visible white line; the ring must start hidden.
    assert_eq!(outline(&app, slider).color, Color::NONE);
}

#[test]
fn the_builder_defaults_to_a_unit_range_and_one_percent_keys() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);

    let world = app.world();
    assert_eq!(
        world.get::<SliderRange>(slider),
        Some(&SliderRange::default())
    );
    assert_eq!(world.get::<SliderValue>(slider), Some(&SliderValue(0.0)));
    assert_eq!(world.get::<SliderStep>(slider), Some(&SliderStep(0.01)));
}

#[test]
fn the_builder_options_reach_the_bevy_components() {
    let mut app = test_app();
    let slider = app
        .world_mut()
        .spawn(slider().range(10.0, 30.0).value(15.0).step(5.0).build())
        .id();

    let world = app.world();
    assert_eq!(
        world.get::<SliderRange>(slider),
        Some(&SliderRange::new(10.0, 30.0))
    );
    assert_eq!(world.get::<SliderValue>(slider), Some(&SliderValue(15.0)));
    assert_eq!(world.get::<SliderStep>(slider), Some(&SliderStep(5.0)));
}

// A step is a key increment: without one, a wide range must not move by a whole unit per key press.
#[test]
fn the_default_step_scales_with_the_range() {
    let mut app = test_app();
    let slider = app
        .world_mut()
        .spawn(slider().range(0.0, 200.0).build())
        .id();

    assert_eq!(
        app.world().get::<SliderStep>(slider),
        Some(&SliderStep(2.0))
    );
}

// ---------------------------------------------------------------------------
// Initial styling and the position of the parts
// ---------------------------------------------------------------------------

#[test]
fn a_slider_is_styled_after_one_update() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);

    app.update();

    let style = Theme::default().slider;
    assert_eq!(rail(&app, slider), style.rail.normal);
    assert_eq!(fill(&app, slider), style.fill.normal);
    assert_eq!(thumb(&app, slider), style.thumb.normal);
    assert_eq!(fill_width(&app, slider), Val::Percent(0.0));
    assert_eq!(thumb_left(&app, slider), Val::Percent(0.0));
}

#[test]
fn the_initial_value_places_the_fill_and_the_thumb_after_one_update() {
    let mut app = test_app();
    let slider = app.world_mut().spawn(slider().value(0.5).build()).id();

    app.update();

    assert_eq!(fill_width(&app, slider), Val::Percent(50.0));
    assert_eq!(thumb_left(&app, slider), Val::Percent(50.0));
}

#[test]
fn the_position_follows_the_value_and_the_range() {
    let mut app = test_app();
    let slider = app
        .world_mut()
        .spawn(slider().range(100.0, 300.0).value(150.0).build())
        .id();
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(25.0));

    app.world_mut()
        .entity_mut(slider)
        .insert(SliderValue(300.0));
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(100.0));
    assert_eq!(fill_width(&app, slider), Val::Percent(100.0));

    // The range changing under a fixed value moves the thumb as well.
    app.world_mut()
        .entity_mut(slider)
        .insert(SliderRange::new(100.0, 500.0));
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(50.0));
}

// A value outside the range must not push the thumb out of the rail.
#[test]
fn the_position_never_leaves_the_rail() {
    let mut app = test_app();
    let slider = app.world_mut().spawn(slider().value(7.0).build()).id();
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(100.0));

    app.world_mut().entity_mut(slider).insert(SliderValue(-3.0));
    app.update();
    assert_eq!(thumb_left(&app, slider), Val::Percent(0.0));
}

// ---------------------------------------------------------------------------
// Shape
// ---------------------------------------------------------------------------

#[test]
fn the_default_shape_is_a_thin_rail_and_a_round_thumb() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);

    app.update();

    assert_eq!(node(&app, slider).height, Val::Px(20.0));
    assert_eq!(node(&app, rail_part(&app, slider)).height, Val::Px(6.0));
    assert_eq!(node(&app, rail_part(&app, slider)).top, Val::Px(7.0));
    assert_eq!(node(&app, thumb_part(&app, slider)).width, Val::Px(20.0));
    assert_eq!(node(&app, thumb_part(&app, slider)).height, Val::Px(20.0));
    assert_eq!(
        node(&app, thumb_part(&app, slider)).border_radius,
        BorderRadius::all(Val::Px(10.0))
    );
    assert_eq!(node(&app, travel_part(&app, slider)).right, Val::Px(20.0));
}

#[test]
fn a_custom_shape_reaches_every_part_after_one_update() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(squared_theme()));
    let slider = spawn_slider(&mut app);

    app.update();

    let square = BorderRadius::all(Val::Px(0.0));
    // The slider is as tall as its tallest part: here the rail.
    assert_eq!(node(&app, slider).height, Val::Px(10.0));
    assert_eq!(node(&app, rail_part(&app, slider)).height, Val::Px(10.0));
    assert_eq!(node(&app, rail_part(&app, slider)).border_radius, square);
    assert_eq!(node(&app, fill_part(&app, slider)).height, Val::Px(10.0));
    assert_eq!(node(&app, fill_part(&app, slider)).border_radius, square);
    // No thumb: it takes no room, and the fill may use the whole width.
    assert_eq!(node(&app, thumb_part(&app, slider)).width, Val::Px(0.0));
    assert_eq!(node(&app, thumb_part(&app, slider)).height, Val::Px(0.0));
    assert_eq!(node(&app, travel_part(&app, slider)).right, Val::Px(0.0));
}

#[test]
fn a_global_theme_change_reshapes_unscoped_sliders_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_slider(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let scoped = spawn_slider_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(squared_theme()));
    app.update();

    assert_eq!(node(&app, unscoped).height, Val::Px(10.0));
    assert_eq!(node(&app, scoped).height, Val::Px(20.0));
}

#[test]
fn a_scoped_theme_change_reshapes_only_the_sliders_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_slider(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let inside = spawn_slider_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = squared_theme();
    app.update();

    assert_eq!(node(&app, inside).height, Val::Px(10.0));
    assert_eq!(node(&app, outside).height, Val::Px(20.0));
}

// ---------------------------------------------------------------------------
// Segments
// ---------------------------------------------------------------------------

#[test]
fn a_segmented_rail_and_fill_are_blocks_drawn_by_gradients() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let slider = app.world_mut().spawn(slider().value(0.8).build()).id();

    app.update();

    let style = blocks_theme().slider;
    // The plain backgrounds are out of the way, so that the gaps show through.
    assert_eq!(rail(&app, slider), Color::NONE);
    assert_eq!(fill(&app, slider), Color::NONE);
    let rail_stops = stops(&app, rail_part(&app, slider));
    assert_eq!(rail_stops.len(), 38);
    assert_eq!(painted_blocks(&rail_stops, style.rail.normal), 10);
    assert_eq!(
        painted_blocks(&stops(&app, fill_part(&app, slider)), style.fill.normal),
        8
    );
}

#[test]
fn the_lit_blocks_follow_the_value() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let slider = app.world_mut().spawn(slider().value(0.8).build()).id();
    app.update();

    app.world_mut().entity_mut(slider).insert(SliderValue(0.3));
    app.update();

    let fill_color = blocks_theme().slider.fill.normal;
    assert_eq!(
        painted_blocks(&stops(&app, fill_part(&app, slider)), fill_color),
        3
    );
}

// The fill spans the whole bar and lights blocks; it does not grow, and nothing is left of the thumb.
#[test]
fn a_segmented_fill_spans_the_bar_and_there_is_no_thumb() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let slider = app.world_mut().spawn(slider().value(0.4).build()).id();

    app.update();

    assert_eq!(
        node(&app, fill_part(&app, slider)).width,
        Val::Percent(100.0)
    );
    assert_eq!(node(&app, thumb_part(&app, slider)).width, Val::Px(0.0));
    assert_eq!(node(&app, travel_part(&app, slider)).right, Val::Px(0.0));
}

#[test]
fn colors_of_the_blocks_follow_the_state() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let slider = app.world_mut().spawn(slider().value(0.5).build()).id();
    app.update();

    app.world_mut().entity_mut(slider).insert(Hovered(true));
    app.update();

    let style = blocks_theme().slider;
    assert_eq!(
        painted_blocks(&stops(&app, fill_part(&app, slider)), style.fill.hovered),
        5
    );
}

#[test]
fn a_plain_slider_has_no_gradients_and_keeps_its_colors() {
    let mut app = test_app();
    let slider = app.world_mut().spawn(slider().value(0.5).build()).id();

    app.update();

    let style = Theme::default().slider;
    assert!(stops(&app, rail_part(&app, slider)).is_empty());
    assert!(stops(&app, fill_part(&app, slider)).is_empty());
    assert_eq!(rail(&app, slider), style.rail.normal);
    assert_eq!(fill(&app, slider), style.fill.normal);
    assert_eq!(
        node(&app, fill_part(&app, slider)).width,
        Val::Percent(50.0)
    );
}

#[test]
fn swapping_between_segmented_and_plain_themes_switches_the_look_back_and_forth() {
    let mut app = test_app();
    let slider = app.world_mut().spawn(slider().value(0.5).build()).id();
    app.update();

    app.insert_resource(GlobalTheme(blocks_theme()));
    app.update();
    assert_eq!(stops(&app, rail_part(&app, slider)).len(), 38);
    assert_eq!(fill(&app, slider), Color::NONE);

    app.insert_resource(GlobalTheme(Theme::default()));
    app.update();
    assert!(stops(&app, rail_part(&app, slider)).is_empty());
    assert!(stops(&app, fill_part(&app, slider)).is_empty());
    assert_eq!(fill(&app, slider), Theme::default().slider.fill.normal);
    assert_eq!(
        node(&app, fill_part(&app, slider)).width,
        Val::Percent(50.0)
    );
}

// ---------------------------------------------------------------------------
// Interaction states
// ---------------------------------------------------------------------------

#[test]
fn hovering_the_slider_or_its_thumb_changes_the_colors() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = Theme::default().slider;
    app.update();

    // `Hovered` already includes the descendants, so it covers the pickable thumb.
    app.world_mut().entity_mut(slider).insert(Hovered(true));
    app.update();
    assert_eq!(fill(&app, slider), style.fill.hovered);

    app.world_mut().entity_mut(slider).insert(Hovered(false));
    app.update();
    assert_eq!(fill(&app, slider), style.fill.normal);
}

#[test]
fn a_rail_click_shows_the_pressed_colors_until_release() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = Theme::default().slider;
    app.update();

    app.world_mut().entity_mut(slider).insert(Pressed);
    app.update();
    assert_eq!(fill(&app, slider), style.fill.pressed);

    // Removal is not a `Changed`; it has to be picked up through `RemovedComponents`.
    app.world_mut().entity_mut(slider).remove::<Pressed>();
    app.update();
    assert_eq!(fill(&app, slider), style.fill.normal);
}

// Pressing the thumb itself inserts no `Pressed`: only the drag state tells.
#[test]
fn dragging_shows_the_pressed_colors_until_the_drag_ends() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = Theme::default().slider;
    app.update();

    app.world_mut()
        .get_mut::<SliderDragState>(slider)
        .unwrap()
        .dragging = true;
    app.update();
    assert_eq!(fill(&app, slider), style.fill.pressed);

    app.world_mut()
        .get_mut::<SliderDragState>(slider)
        .unwrap()
        .dragging = false;
    app.update();
    assert_eq!(fill(&app, slider), style.fill.normal);
}

#[test]
fn interaction_disabled_grays_the_slider_and_removal_restores_it() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = Theme::default().slider;
    app.update();

    app.world_mut()
        .entity_mut(slider)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(rail(&app, slider), style.rail.disabled);
    assert_eq!(fill(&app, slider), style.fill.disabled);
    assert_eq!(thumb(&app, slider), style.thumb.disabled);

    app.world_mut()
        .entity_mut(slider)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(fill(&app, slider), style.fill.normal);
    assert_eq!(thumb(&app, slider), style.thumb.normal);
}

// ---------------------------------------------------------------------------
// Theme resolution
// ---------------------------------------------------------------------------

#[test]
fn global_theme_change_updates_unscoped_sliders_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_slider(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let scoped = spawn_slider_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_fill_theme()));
    app.update();

    assert_eq!(fill(&app, unscoped), red());
    assert_eq!(fill(&app, scoped), Theme::default().slider.fill.normal);
}

#[test]
fn scoped_theme_change_updates_only_sliders_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_slider(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let inside = spawn_slider_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_fill_theme();
    app.update();

    assert_eq!(fill(&app, inside), red());
    assert_eq!(fill(&app, outside), Theme::default().slider.fill.normal);
}

#[test]
fn slider_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_fill_theme()).id();
    app.update();

    let late = spawn_slider_under(&mut app, panel);
    app.update();

    assert_eq!(fill(&app, late), red());
}

#[test]
fn editing_follows_a_custom_theme() {
    let mut app = test_app();
    let mut theme = Theme::default();
    theme.slider.editing_thumb = red();
    theme.slider.editing_ring = red();
    app.insert_resource(GlobalTheme(theme));
    let slider = app
        .world_mut()
        .spawn((slider().build(), SliderEditing))
        .id();
    app.update();
    focus(&mut app, slider, true);
    app.update();

    assert_eq!(thumb(&app, slider), red());
    assert_eq!(outline(&app, slider).color, red());
}

// ---------------------------------------------------------------------------
// Focus
// ---------------------------------------------------------------------------

#[test]
fn visible_focus_draws_the_ring_and_uses_the_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let slider = spawn_slider(&mut app);
    app.update();

    focus(&mut app, slider, true);
    app.update();

    let style = focus_theme().slider;
    assert_eq!(fill(&app, slider), style.fill.focused);
    let ring = outline(&app, slider);
    assert_eq!(ring.color, style.focus_ring.color);
    assert_eq!(ring.width, style.focus_ring.width);
}

// Pointer clicks hide the indicator while the entity keeps focus.
#[test]
fn focus_is_not_shown_while_the_indicator_is_hidden() {
    let mut app = test_app_with_focus_theme();
    let slider = spawn_slider(&mut app);
    app.update();

    focus(&mut app, slider, false);
    app.update();

    assert_eq!(fill(&app, slider), focus_theme().slider.fill.normal);
    assert_eq!(outline(&app, slider).color, Color::NONE);
}

#[test]
fn moving_focus_moves_the_ring() {
    let mut app = test_app_with_focus_theme();
    let first = spawn_slider(&mut app);
    let second = spawn_slider(&mut app);
    app.update();
    focus(&mut app, first, true);
    app.update();

    focus(&mut app, second, true);
    app.update();

    assert_eq!(outline(&app, first).color, Color::NONE);
    assert_eq!(
        outline(&app, second).color,
        focus_theme().slider.focus_ring.color
    );
}

#[test]
fn disabled_slider_shows_neither_ring_nor_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let slider = spawn_slider(&mut app);
    app.update();
    focus(&mut app, slider, true);
    app.world_mut()
        .entity_mut(slider)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(fill(&app, slider), focus_theme().slider.fill.disabled);
    assert_eq!(outline(&app, slider).color, Color::NONE);
}

// ---------------------------------------------------------------------------
// Editing mode
// ---------------------------------------------------------------------------

#[test]
fn editing_changes_the_thumb_and_the_ring_and_removal_restores_them() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = Theme::default().slider;
    app.update();
    focus(&mut app, slider, true);
    app.update();
    let idle_ring = outline(&app, slider);
    assert_eq!(idle_ring.color, style.focus_ring.color);

    app.world_mut().entity_mut(slider).insert(SliderEditing);
    app.update();
    assert_eq!(thumb(&app, slider), style.editing_thumb);
    let ring = outline(&app, slider);
    assert_eq!(ring.color, style.editing_ring);
    assert_eq!(ring.width, idle_ring.width);
    // The rail and the fill are not part of it.
    assert_eq!(rail(&app, slider), style.rail.focused);
    assert_eq!(fill(&app, slider), style.fill.focused);

    // Removal is not a `Changed`; it has to be picked up through `RemovedComponents`.
    app.world_mut().entity_mut(slider).remove::<SliderEditing>();
    app.update();
    assert_eq!(thumb(&app, slider), style.thumb.focused);
    assert_eq!(outline(&app, slider).color, style.focus_ring.color);
}

#[test]
fn editing_shows_no_ring_while_the_focus_indicator_is_hidden() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    app.update();
    focus(&mut app, slider, false);
    app.world_mut().entity_mut(slider).insert(SliderEditing);
    app.update();

    assert_eq!(outline(&app, slider).color, Color::NONE);
    assert_eq!(thumb(&app, slider), Theme::default().slider.editing_thumb);
}

#[test]
fn disabled_slider_ignores_editing() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    let style = Theme::default().slider;
    app.update();

    focus(&mut app, slider, true);
    app.world_mut()
        .entity_mut(slider)
        .insert((SliderEditing, InteractionDisabled));
    app.update();

    assert_eq!(thumb(&app, slider), style.thumb.disabled);
    assert_eq!(outline(&app, slider).color, Color::NONE);
}

// ---------------------------------------------------------------------------
// Tab order
// ---------------------------------------------------------------------------

#[test]
fn disabled_slider_leaves_the_tab_order_and_returns_when_enabled() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(slider)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(tab_index(&app, slider), -1);

    app.world_mut()
        .entity_mut(slider)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, slider), 0);
}

// ---------------------------------------------------------------------------
// Controlled value: the slider asks, the app decides
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, f32)>);

fn app_with_slider_behavior() -> App {
    let mut app = test_app();
    app.add_plugins(SliderPlugin)
        .init_resource::<Requests>()
        .add_observer(|change: On<ValueChange<f32>>, mut log: ResMut<Requests>| {
            log.0.push((change.source, change.value));
        });
    app
}

#[test]
fn a_request_reports_the_new_value_but_never_changes_the_value_by_itself() {
    let mut app = app_with_slider_behavior();
    let slider = spawn_slider(&mut app);
    app.update();

    app.world_mut().trigger(SetSliderValue {
        entity: slider,
        change: SliderValueChange::Absolute(0.75),
    });
    app.update();

    assert_eq!(app.world().resource::<Requests>().0, vec![(slider, 0.75)]);
    assert_eq!(
        app.world().get::<SliderValue>(slider),
        Some(&SliderValue(0.0))
    );
    assert_eq!(thumb_left(&app, slider), Val::Percent(0.0));
}

#[test]
fn the_app_applying_the_request_moves_the_parts() {
    let mut app = app_with_slider_behavior();
    app.add_observer(|change: On<ValueChange<f32>>, mut commands: Commands| {
        commands
            .entity(change.source)
            .insert(SliderValue(change.value));
    });
    let slider = spawn_slider(&mut app);
    app.update();

    app.world_mut().trigger(SetSliderValue {
        entity: slider,
        change: SliderValueChange::Absolute(0.75),
    });
    app.update();

    assert_eq!(thumb_left(&app, slider), Val::Percent(75.0));
    assert_eq!(fill_width(&app, slider), Val::Percent(75.0));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app
}

fn spawn_slider(app: &mut App) -> Entity {
    app.world_mut().spawn(slider().build()).id()
}

fn spawn_slider_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((slider().build(), ChildOf(parent)))
        .id()
}

fn red() -> Color {
    Color::srgb(0.9, 0.1, 0.1)
}

/// Default theme whose fill is red, so that it is distinguishable from the default.
fn red_fill_theme() -> Theme {
    let mut theme = Theme::default();
    theme.slider.fill = StateColors::uniform(red());
    theme
}

/// Default theme with a thick, square rail and no thumb.
fn squared_theme() -> Theme {
    let mut theme = Theme::default();
    theme.slider.shape = SliderShape {
        rail_height: 10.0,
        thumb_size: 0.0,
        rounded: false,
        ..default()
    };
    theme
}

/// Default theme with a ten-block bar.
fn blocks_theme() -> Theme {
    let mut theme = Theme::default();
    theme.slider.shape = SliderShape {
        rounded: false,
        segments: Some(SliderSegments {
            count: 10,
            gap: 0.2,
        }),
        ..default()
    };
    theme
}

/// Default theme whose `focused` fill color differs from every other state.
fn focus_theme() -> Theme {
    let mut theme = Theme::default();
    theme.slider.fill.focused = Color::srgb(0.1, 0.2, 0.3);
    theme
}

fn test_app_with_focus_theme() -> App {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(focus_theme()));
    app
}

fn focus(app: &mut App, slider: Entity, visible: bool) {
    app.insert_resource(InputFocus::from_entity(slider));
    app.insert_resource(InputFocusVisible(visible));
}

fn outline(app: &App, slider: Entity) -> Outline {
    *app.world().get::<Outline>(slider).unwrap()
}

fn tab_index(app: &App, slider: Entity) -> i32 {
    app.world().get::<TabIndex>(slider).unwrap().0
}

/// The parts, in the order the builder spawns them: the rail, then a travel area holding the fill
/// and the thumb.
fn parts(app: &App, slider: Entity) -> (Entity, Entity, Entity) {
    let world = app.world();
    let children = world.get::<Children>(slider).unwrap();
    let travel = world.get::<Children>(children[1]).unwrap();
    (children[0], travel[0], travel[1])
}

fn color_of(app: &App, entity: Entity) -> Color {
    app.world().get::<BackgroundColor>(entity).unwrap().0
}

fn rail(app: &App, slider: Entity) -> Color {
    color_of(app, parts(app, slider).0)
}

fn fill(app: &App, slider: Entity) -> Color {
    color_of(app, parts(app, slider).1)
}

fn thumb(app: &App, slider: Entity) -> Color {
    color_of(app, parts(app, slider).2)
}

fn fill_width(app: &App, slider: Entity) -> Val {
    app.world().get::<Node>(parts(app, slider).1).unwrap().width
}

fn thumb_left(app: &App, slider: Entity) -> Val {
    app.world().get::<Node>(parts(app, slider).2).unwrap().left
}

fn node(app: &App, entity: Entity) -> Node {
    app.world().get::<Node>(entity).unwrap().clone()
}

fn rail_part(app: &App, slider: Entity) -> Entity {
    parts(app, slider).0
}

fn fill_part(app: &App, slider: Entity) -> Entity {
    parts(app, slider).1
}

fn thumb_part(app: &App, slider: Entity) -> Entity {
    parts(app, slider).2
}

fn travel_part(app: &App, slider: Entity) -> Entity {
    app.world().get::<Children>(slider).unwrap()[1]
}

/// The stops of the part's only gradient; empty when it has none.
fn stops(app: &App, part: Entity) -> Vec<ColorStop> {
    let gradient = app.world().get::<BackgroundGradient>(part).unwrap();
    match gradient.0.first() {
        Some(Gradient::Linear(linear)) => linear.stops.clone(),
        Some(other) => panic!("not a linear gradient: {other:?}"),
        None => Vec::new(),
    }
}

/// How many blocks `stops` paints with `color` (each block is a pair of stops of that color).
fn painted_blocks(stops: &[ColorStop], color: Color) -> usize {
    stops.iter().filter(|stop| stop.color == color).count() / 2
}
