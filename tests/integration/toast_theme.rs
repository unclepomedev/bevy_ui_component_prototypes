//! Behavior tests for the toast: structure and the pointer.
//!
//! Colors, sizes and the theme come with the style sync; the slide and the real screen are covered by
//! `examples/toast_check.rs`.

use crate::probe::probe_theme;
use bevy::{picking::Pickable, prelude::*};
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, Toast, ToastHost, ToastStack, ToastVariant, ToastVariantId,
    UiComponentPrototypesPlugin, toast, toast_host,
};

#[test]
fn a_toast_carries_its_structure_and_starts_unseen() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");

    let world = app.world();
    assert_eq!(
        world.get::<Toast>(toast),
        Some(&Toast {
            elapsed: 0.0,
            duration: None
        })
    );
    assert_eq!(
        world.get::<ToastVariant>(toast),
        Some(&ToastVariant::Default)
    );
    assert!(world.get::<UiTransform>(toast).is_some());
    assert_eq!(world.get::<Children>(toast).map(|c| c.len()), Some(1));
    assert_eq!(text(&app, toast), "Saved");
    // A default `TextColor` is white: the text must not show before the first style sync.
    assert_eq!(label_color(&app, toast), Color::NONE);
    assert_eq!(
        world.get::<BackgroundColor>(toast).map(|c| c.0),
        Some(Color::NONE)
    );
}

#[test]
fn the_builder_options_reach_the_components() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(
            toast("Done")
                .variant(ToastVariantId::SUCCESS)
                .duration(2.5)
                .build(),
        )
        .id();

    let world = app.world();
    assert_eq!(world.get::<Toast>(toast).unwrap().duration, Some(2.5));
    assert_eq!(
        world.get::<ToastVariant>(toast),
        Some(&ToastVariant::Named(ToastVariantId::SUCCESS))
    );
}

#[test]
fn the_text_keeps_its_line_breaks_and_wraps() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "first\nsecond");

    assert_eq!(text(&app, toast), "first\nsecond");
    // Unlike a button label, a toast wraps at the width the theme gives it.
    assert_eq!(label_layout(&app, toast).linebreak, LineBreak::WordBoundary);
}

#[test]
fn a_toast_and_its_text_let_the_pointer_through() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");

    let world = app.world();
    assert!(passes_through(world.get::<Pickable>(toast).unwrap()));
    assert!(passes_through(
        world.get::<Pickable>(label(&app, toast)).unwrap()
    ));
}

// ---------------------------------------------------------------------------
// The host
// ---------------------------------------------------------------------------

#[test]
fn a_host_has_the_defaults_and_lets_the_pointer_through() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();

    let world = app.world();
    assert_eq!(
        world.get::<ToastHost>(host),
        Some(&ToastHost {
            max: 5,
            stack: ToastStack::NewestAtBottom,
            slide: Val2::ZERO,
        })
    );
    assert!(world.get::<Node>(host).is_some());
    assert!(passes_through(world.get::<Pickable>(host).unwrap()));
}

#[test]
fn the_host_options_reach_the_component() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn(
            toast_host()
                .max(2)
                .stack(ToastStack::NewestAtTop)
                .slide(Val2::percent(100, 0))
                .build(),
        )
        .id();

    assert_eq!(
        app.world().get::<ToastHost>(host),
        Some(&ToastHost {
            max: 2,
            stack: ToastStack::NewestAtTop,
            slide: Val2::percent(100, 0),
        })
    );
}

// ---------------------------------------------------------------------------
// The stacking direction
// ---------------------------------------------------------------------------

#[test]
fn the_newest_toast_goes_to_the_end_the_stack_names() {
    // The newest toast is the last child.
    for (stack, direction) in [
        (ToastStack::NewestAtBottom, FlexDirection::Column),
        (ToastStack::NewestAtTop, FlexDirection::ColumnReverse),
    ] {
        let mut app = test_app();
        let host = app
            .world_mut()
            .spawn(toast_host().stack(stack).build())
            .id();

        app.update();

        assert_eq!(flex_direction(&app, host), direction, "{stack:?}");
    }
}

#[test]
fn changing_the_stack_changes_the_direction() {
    let mut app = test_app();
    let host = app.world_mut().spawn(toast_host().build()).id();
    app.update();

    app.world_mut().entity_mut(host).insert(ToastHost {
        stack: ToastStack::NewestAtTop,
        ..default()
    });
    app.update();

    assert_eq!(flex_direction(&app, host), FlexDirection::ColumnReverse);
}

// A `Node` spawned together with the host replaces the one the host requires.
#[test]
fn the_apps_node_keeps_its_own_fields_and_gets_the_direction() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn((
            toast_host().stack(ToastStack::NewestAtTop).build(),
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                top: px(24),
                align_items: AlignItems::FlexEnd,
                ..default()
            },
        ))
        .id();

    app.update();

    let node = app.world().get::<Node>(host).unwrap();
    assert_eq!(node.flex_direction, FlexDirection::ColumnReverse);
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.align_items, AlignItems::FlexEnd);
}

#[test]
fn a_node_inserted_later_gets_the_direction_back() {
    let mut app = test_app();
    let host = app
        .world_mut()
        .spawn(toast_host().stack(ToastStack::NewestAtTop).build())
        .id();
    app.update();

    app.world_mut().entity_mut(host).insert(Node::default());
    app.update();

    assert_eq!(flex_direction(&app, host), FlexDirection::ColumnReverse);
}

// ---------------------------------------------------------------------------
// Styling and theme resolution
// ---------------------------------------------------------------------------

#[test]
fn a_toast_is_styled_after_one_update() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");

    app.update();

    let style = probe_theme().toasts.base;
    assert_eq!(bg(&app, toast), style.bg);
    assert_eq!(border_color(&app, toast), BorderColor::all(style.border));
    assert_eq!(label_color(&app, toast), style.fg);
    let node = app.world().get::<Node>(toast).unwrap();
    assert_eq!(node.border, UiRect::all(Val::Px(style.shape.border_width)));
    assert_eq!(
        node.border_radius,
        BorderRadius::all(Val::Px(style.shape.radius))
    );
}

#[test]
fn a_named_variant_takes_its_style() {
    let mut app = test_app();
    let toast = spawn_success(&mut app);

    app.update();

    let style = probe_theme().toasts.named[&ToastVariantId::SUCCESS];
    assert_eq!(bg(&app, toast), style.bg);
    assert_eq!(label_color(&app, toast), style.fg);
}

#[test]
fn a_name_the_theme_lacks_falls_back_to_the_base_style() {
    let mut app = test_app();
    let toast = app
        .world_mut()
        .spawn(toast("Saved").variant(ToastVariantId::new("gold")).build())
        .id();

    app.update();

    assert_eq!(bg(&app, toast), probe_theme().toasts.base.bg);
}

#[test]
fn changing_the_variant_of_a_toast_restyles_it() {
    let mut app = test_app();
    let toast = spawn_toast(&mut app, "Saved");
    app.update();

    app.world_mut()
        .entity_mut(toast)
        .insert(ToastVariant::Named(ToastVariantId::DANGER));
    app.update();

    let style = probe_theme().toasts.named[&ToastVariantId::DANGER];
    assert_eq!(bg(&app, toast), style.bg);
    assert_eq!(label_color(&app, toast), style.fg);
}

#[test]
fn global_theme_change_updates_unscoped_toasts_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_success(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_success_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(red_success_theme()));
    app.update();

    assert_eq!(bg(&app, unscoped), red());
    assert_eq!(bg(&app, scoped), success_bg());
}

#[test]
fn scoped_theme_change_updates_only_toasts_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_success(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_success_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = red_success_theme();
    app.update();

    assert_eq!(bg(&app, inside), red());
    assert_eq!(bg(&app, outside), success_bg());
}

#[test]
fn toast_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(red_success_theme()).id();
    app.update();

    let late = spawn_success_under(&mut app, panel);
    app.update();

    assert_eq!(bg(&app, late), red());
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

fn spawn_toast(app: &mut App, text: &str) -> Entity {
    app.world_mut().spawn(toast(text).build()).id()
}

fn passes_through(pickable: &Pickable) -> bool {
    !pickable.should_block_lower && !pickable.is_hoverable
}

fn label(app: &App, toast: Entity) -> Entity {
    app.world().get::<Children>(toast).unwrap()[0]
}

fn text(app: &App, toast: Entity) -> String {
    app.world()
        .get::<Text>(label(app, toast))
        .unwrap()
        .0
        .clone()
}

fn label_color(app: &App, toast: Entity) -> Color {
    app.world().get::<TextColor>(label(app, toast)).unwrap().0
}

fn label_layout(app: &App, toast: Entity) -> TextLayout {
    *app.world().get::<TextLayout>(label(app, toast)).unwrap()
}

fn flex_direction(app: &App, host: Entity) -> FlexDirection {
    app.world().get::<Node>(host).unwrap().flex_direction
}

fn spawn_success(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(toast("Done").variant(ToastVariantId::SUCCESS).build())
        .id()
}

fn spawn_success_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((
            toast("Done").variant(ToastVariantId::SUCCESS).build(),
            ChildOf(parent),
        ))
        .id()
}

fn red() -> Color {
    Color::srgb(0.9, 0.1, 0.1)
}

fn success_bg() -> Color {
    probe_theme().toasts.named[&ToastVariantId::SUCCESS].bg
}

/// The probe theme with a red background for the success toasts.
fn red_success_theme() -> Theme {
    let mut theme = probe_theme();
    theme
        .toasts
        .named
        .get_mut(&ToastVariantId::SUCCESS)
        .unwrap()
        .bg = red();
    theme
}

fn bg(app: &App, toast: Entity) -> Color {
    app.world().get::<BackgroundColor>(toast).unwrap().0
}

fn border_color(app: &App, toast: Entity) -> BorderColor {
    *app.world().get::<BorderColor>(toast).unwrap()
}
