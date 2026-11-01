//! Behavior tests for the progress bar: structure, the position of the fill, variants, theme
//! resolution and the blocks.
//!
//! The real screen is covered by `examples/progress_check.rs`.

use crate::{
    gradient::{painted_blocks, stops},
    probe::probe_theme,
};
use accesskit::Role;
use bevy::{a11y::AccessibilityNode, picking::Pickable, prelude::*};
use bevy_ui_component_prototypes::{
    BarSegments, GlobalTheme, ProgressRange, ProgressShape, ProgressValue, ProgressVariant,
    ProgressVariantId, Theme, UiComponentPrototypesPlugin, progress_bar,
};

// ---------------------------------------------------------------------------
// Structure
// ---------------------------------------------------------------------------

#[test]
fn a_bar_carries_its_structure_and_lets_the_pointer_through() {
    let mut app = test_app();
    let bar = spawn_bar(&mut app);

    let world = app.world();
    assert_eq!(
        world.get::<AccessibilityNode>(bar).map(|n| n.role()),
        Some(Role::ProgressIndicator)
    );
    assert_eq!(
        world.get::<ProgressRange>(bar),
        Some(&ProgressRange::default())
    );
    assert_eq!(world.get::<ProgressValue>(bar), Some(&ProgressValue(0.0)));
    assert_eq!(
        world.get::<ProgressVariant>(bar),
        Some(&ProgressVariant::Default)
    );
    assert_eq!(world.get::<Children>(bar).map(|c| c.len()), Some(1));
    assert!(passes_through(world.get::<Pickable>(bar).unwrap()));
    assert!(passes_through(
        world.get::<Pickable>(fill_part(&app, bar)).unwrap()
    ));
}

#[test]
fn the_builder_options_reach_the_components() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(
            progress_bar()
                .range(10.0, 30.0)
                .value(15.0)
                .variant(ProgressVariantId::PRIMARY)
                .build(),
        )
        .id();

    let world = app.world();
    assert_eq!(
        world.get::<ProgressRange>(bar),
        Some(&ProgressRange {
            start: 10.0,
            end: 30.0
        })
    );
    assert_eq!(world.get::<ProgressValue>(bar), Some(&ProgressValue(15.0)));
    assert_eq!(
        world.get::<ProgressVariant>(bar),
        Some(&ProgressVariant::Named(ProgressVariantId::PRIMARY))
    );
}

#[test]
fn without_a_value_the_bar_starts_at_the_range_start() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(progress_bar().range(10.0, 30.0).build())
        .id();

    assert_eq!(
        app.world().get::<ProgressValue>(bar),
        Some(&ProgressValue(10.0))
    );
}

// ---------------------------------------------------------------------------
// Initial styling and the position of the fill
// ---------------------------------------------------------------------------

#[test]
fn a_bar_is_styled_after_one_update() {
    let mut app = test_app();
    let bar = spawn_bar(&mut app);

    app.update();

    let style = probe_theme().progress.base;
    let round = BorderRadius::all(Val::Px(style.shape.height / 2.0));
    assert_eq!(rail(&app, bar), style.rail);
    assert_eq!(fill(&app, bar), style.fill);
    assert_eq!(node(&app, bar).height, Val::Px(style.shape.height));
    assert_eq!(node(&app, bar).border_radius, round);
    assert_eq!(node(&app, fill_part(&app, bar)).border_radius, round);
    assert_eq!(fill_width(&app, bar), Val::Percent(0.0));
}

#[test]
fn the_initial_value_places_the_fill_after_one_update() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(progress_bar().value(0.5).build())
        .id();

    app.update();

    assert_eq!(fill_width(&app, bar), Val::Percent(50.0));
}

#[test]
fn the_fill_follows_the_value_and_the_range() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(progress_bar().range(100.0, 300.0).value(150.0).build())
        .id();
    app.update();
    assert_eq!(fill_width(&app, bar), Val::Percent(25.0));

    app.world_mut().entity_mut(bar).insert(ProgressValue(300.0));
    app.update();
    assert_eq!(fill_width(&app, bar), Val::Percent(100.0));

    // The range changing under a fixed value moves the fill as well.
    app.world_mut().entity_mut(bar).insert(ProgressRange {
        start: 100.0,
        end: 500.0,
    });
    app.update();
    assert_eq!(fill_width(&app, bar), Val::Percent(50.0));
}

#[test]
fn the_value_can_be_written_through_deref() {
    let mut app = test_app();
    let bar = spawn_bar(&mut app);
    app.update();

    **app.world_mut().get_mut::<ProgressValue>(bar).unwrap() = 0.25;
    app.update();

    assert_eq!(fill_width(&app, bar), Val::Percent(25.0));
}

#[test]
fn the_fill_never_leaves_the_rail() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(progress_bar().value(7.0).build())
        .id();
    app.update();
    assert_eq!(fill_width(&app, bar), Val::Percent(100.0));

    app.world_mut().entity_mut(bar).insert(ProgressValue(-3.0));
    app.update();
    assert_eq!(fill_width(&app, bar), Val::Percent(0.0));
}

#[test]
fn an_empty_range_and_a_nan_value_show_an_empty_bar() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(progress_bar().range(1.0, 1.0).value(1.0).build())
        .id();
    app.update();
    assert_eq!(fill_width(&app, bar), Val::Percent(0.0));

    app.world_mut()
        .entity_mut(bar)
        .insert((ProgressRange::default(), ProgressValue(f32::NAN)));
    app.update();
    assert_eq!(fill_width(&app, bar), Val::Percent(0.0));
}

#[test]
fn fraction_is_where_the_value_lies_between_the_ends() {
    let range = ProgressRange {
        start: 10.0,
        end: 30.0,
    };
    assert_eq!(range.fraction(10.0), 0.0);
    assert_eq!(range.fraction(20.0), 0.5);
    assert_eq!(range.fraction(30.0), 1.0);
    assert_eq!(range.fraction(99.0), 1.0);
    assert_eq!(range.fraction(-99.0), 0.0);
}

// ---------------------------------------------------------------------------
// Variants
// ---------------------------------------------------------------------------

#[test]
fn a_named_variant_takes_its_style() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(progress_bar().variant(ProgressVariantId::PRIMARY).build())
        .id();

    app.update();

    let style = probe_theme().progress.named[&ProgressVariantId::PRIMARY];
    assert_eq!(rail(&app, bar), style.rail);
    assert_eq!(fill(&app, bar), style.fill);
}

#[test]
fn a_name_the_theme_lacks_falls_back_to_the_base_style() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(
            progress_bar()
                .variant(ProgressVariantId::new("gold"))
                .build(),
        )
        .id();

    app.update();

    assert_eq!(fill(&app, bar), probe_theme().progress.base.fill);
}

#[test]
fn changing_the_variant_of_a_bar_restyles_it() {
    let mut app = test_app();
    let bar = spawn_bar(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(bar)
        .insert(ProgressVariant::Named(ProgressVariantId::DANGER));
    app.update();

    let style = probe_theme().progress.named[&ProgressVariantId::DANGER];
    assert_eq!(rail(&app, bar), style.rail);
    assert_eq!(fill(&app, bar), style.fill);
}

// ---------------------------------------------------------------------------
// Shape
// ---------------------------------------------------------------------------

#[test]
fn a_custom_shape_reaches_the_bar_and_its_fill() {
    let mut app = test_app();
    let mut theme = probe_theme();
    theme.progress.base.shape = ProgressShape {
        height: 20.0,
        rounded: false,
        segments: None,
    };
    app.insert_resource(GlobalTheme(theme));
    let bar = spawn_bar(&mut app);

    app.update();

    let square = BorderRadius::all(Val::Px(0.0));
    assert_eq!(node(&app, bar).height, Val::Px(20.0));
    assert_eq!(node(&app, bar).border_radius, square);
    assert_eq!(node(&app, fill_part(&app, bar)).border_radius, square);
}

// ---------------------------------------------------------------------------
// Segments
// ---------------------------------------------------------------------------

#[test]
fn a_segmented_rail_and_fill_are_blocks_drawn_by_gradients() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let bar = app
        .world_mut()
        .spawn(progress_bar().value(0.8).build())
        .id();

    app.update();

    let style = blocks_theme().progress.base;
    // The plain backgrounds are out of the way, so that the gaps show through.
    assert_eq!(rail(&app, bar), Color::NONE);
    assert_eq!(fill(&app, bar), Color::NONE);
    assert_eq!(painted_blocks(&stops(&app, bar), style.rail), 10);
    assert_eq!(
        painted_blocks(&stops(&app, fill_part(&app, bar)), style.fill),
        8
    );
}

#[test]
fn the_lit_blocks_follow_the_value_and_the_fill_spans_the_bar() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(blocks_theme()));
    let bar = app
        .world_mut()
        .spawn(progress_bar().value(0.8).build())
        .id();
    app.update();

    app.world_mut().entity_mut(bar).insert(ProgressValue(0.3));
    app.update();

    let fill_color = blocks_theme().progress.base.fill;
    assert_eq!(
        painted_blocks(&stops(&app, fill_part(&app, bar)), fill_color),
        3
    );
    assert_eq!(fill_width(&app, bar), Val::Percent(100.0));
}

#[test]
fn a_plain_bar_has_no_gradients() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(progress_bar().value(0.5).build())
        .id();

    app.update();

    assert!(stops(&app, bar).is_empty());
    assert!(stops(&app, fill_part(&app, bar)).is_empty());
}

#[test]
fn swapping_between_segmented_and_plain_themes_switches_the_look_back_and_forth() {
    let mut app = test_app();
    let bar = app
        .world_mut()
        .spawn(progress_bar().value(0.5).build())
        .id();
    app.update();

    app.insert_resource(GlobalTheme(blocks_theme()));
    app.update();
    assert!(!stops(&app, bar).is_empty());
    assert_eq!(fill(&app, bar), Color::NONE);

    app.insert_resource(GlobalTheme(probe_theme()));
    app.update();
    assert!(stops(&app, bar).is_empty());
    assert!(stops(&app, fill_part(&app, bar)).is_empty());
    assert_eq!(fill(&app, bar), probe_theme().progress.base.fill);
    assert_eq!(fill_width(&app, bar), Val::Percent(50.0));
}

// ---------------------------------------------------------------------------
// Theme resolution
// ---------------------------------------------------------------------------

#[test]
fn global_theme_change_updates_unscoped_bars_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_bar(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_bar_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_fill_theme()));
    app.update();

    assert_eq!(fill(&app, unscoped), red());
    assert_eq!(fill(&app, scoped), probe_theme().progress.base.fill);
}

#[test]
fn scoped_theme_change_updates_only_bars_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_bar(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_bar_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_fill_theme();
    app.update();

    assert_eq!(fill(&app, inside), red());
    assert_eq!(fill(&app, outside), probe_theme().progress.base.fill);
}

#[test]
fn a_bar_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_fill_theme()).id();
    app.update();

    let late = spawn_bar_under(&mut app, panel);
    app.update();

    assert_eq!(fill(&app, late), red());
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, UiComponentPrototypesPlugin));
    app.insert_resource(GlobalTheme(probe_theme()));
    app
}

fn spawn_bar(app: &mut App) -> Entity {
    app.world_mut().spawn(progress_bar().build()).id()
}

fn spawn_bar_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((progress_bar().build(), ChildOf(parent)))
        .id()
}

fn passes_through(pickable: &Pickable) -> bool {
    !pickable.should_block_lower && !pickable.is_hoverable
}

fn red() -> Color {
    Color::srgb(0.9, 0.1, 0.1)
}

/// The probe theme with a red fill for the base style.
fn red_fill_theme() -> Theme {
    let mut theme = probe_theme();
    theme.progress.base.fill = red();
    theme
}

/// The probe theme with a ten-block bar.
fn blocks_theme() -> Theme {
    let mut theme = probe_theme();
    theme.progress.base.shape.segments = Some(BarSegments {
        count: 10,
        gap: 0.2,
    });
    theme
}

fn fill_part(app: &App, bar: Entity) -> Entity {
    app.world().get::<Children>(bar).unwrap()[0]
}

fn color_of(app: &App, entity: Entity) -> Color {
    app.world().get::<BackgroundColor>(entity).unwrap().0
}

fn rail(app: &App, bar: Entity) -> Color {
    color_of(app, bar)
}

fn fill(app: &App, bar: Entity) -> Color {
    color_of(app, fill_part(app, bar))
}

fn fill_width(app: &App, bar: Entity) -> Val {
    node(app, fill_part(app, bar)).width
}

fn node(app: &App, entity: Entity) -> Node {
    app.world().get::<Node>(entity).unwrap().clone()
}
