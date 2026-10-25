//! Behavior tests for button styling and theme resolution.
//!
//! Real pointer picking, layout, and text shaping are not covered by `examples/button_check.rs`.

use crate::probe::probe_theme;
use bevy::{
    a11y::AccessibilityNode,
    asset::uuid_handle,
    ecs::system::RunSystemOnce,
    input_focus::{
        InputFocus, InputFocusVisible,
        tab_navigation::{NavAction, TabGroup, TabIndex, TabNavigation},
    },
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{
        InteractionDisabled, VisualBox, auto_directional_navigation::AutoDirectionalNavigation,
        widget::NodeImageMode,
    },
    ui_widgets::Button,
};
use bevy_ui_component_prototypes::{
    ButtonMetrics, ButtonShape, ButtonSize, ButtonSizeId, ButtonStyle, ButtonVariant,
    ButtonVariantId, FocusIndicator, GlobalTheme, SlicedImage, StateColors, Theme,
    UiComponentPrototypesPlugin, icon_button,
};

// ---------------------------------------------------------------------------
// Behavior wiring
// ---------------------------------------------------------------------------

#[test]
fn icon_button_carries_the_bevy_button_behavior_components() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    let world = app.world();
    assert!(world.get::<Button>(button).is_some());
    assert!(world.get::<AccessibilityNode>(button).is_some());
    assert!(world.get::<PickingInteraction>(button).is_some());
    assert_eq!(world.get::<TabIndex>(button).map(|t| t.0), Some(0));
    // `Outline::default()` would be a visible white line; the ring must start hidden.
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[derive(Component)]
struct Marker(u8);

#[derive(Component)]
struct Other;

#[test]
fn label_bundles_land_on_the_label_only_and_accumulate() {
    let mut app = test_app();
    let icon = Handle::<Image>::default();
    let button = app
        .world_mut()
        .spawn(
            icon_button("button")
                .icon(icon)
                .variant(ButtonVariantId::PRIMARY)
                .label_bundle(Marker(7))
                .label_bundle(Other)
                .build(),
        )
        .id();
    app.update();

    let world = app.world();
    let children: Vec<Entity> = world.get::<Children>(button).unwrap().iter().collect();
    let labels: Vec<_> = children
        .iter()
        .filter(|c| world.get::<Text>(**c).is_some())
        .collect();
    assert_eq!(labels.len(), 1);
    assert_eq!(world.get::<Marker>(*labels[0]).map(|m| m.0), Some(7));
    assert!(world.get::<Other>(*labels[0]).is_some());
    // Neither the root nor the icon gets them.
    assert!(world.get::<Marker>(button).is_none());
    assert!(
        children
            .iter()
            .filter(|c| world.get::<Text>(**c).is_none())
            .all(|c| world.get::<Marker>(*c).is_none())
    );
    // The theme still styles the label.
    assert_eq!(label_font(&app, button), probe_theme().font);
}

#[test]
fn a_label_never_wraps_because_the_button_is_one_line_high() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    assert_eq!(label_layout(&app, button).linebreak, LineBreak::NoWrap);
}

// ---------------------------------------------------------------------------
// Initial styling
// ---------------------------------------------------------------------------

#[test]
fn a_button_attached_to_an_existing_entity_is_styled_after_one_update() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            PickingInteraction::None,
            ButtonVariant::Named(ButtonVariantId::PRIMARY),
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Outline::default(),
        ))
        .id();
    app.update();

    app.world_mut().entity_mut(entity).insert(Button);
    app.update();

    assert_eq!(bg(&app, entity), primary(&probe_theme()).bg.normal);
}

// ---------------------------------------------------------------------------
// Interaction states
// ---------------------------------------------------------------------------

#[test]
fn hover_and_press_change_colors_and_release_restores_them() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    let style = primary(&probe_theme());
    app.update();

    for (state, expected) in [
        (PickingInteraction::Hovered, style.bg.hovered),
        (PickingInteraction::Pressed, style.bg.pressed),
        (PickingInteraction::None, style.bg.normal),
    ] {
        app.world_mut().entity_mut(button).insert(state);
        app.update();
        assert_eq!(bg(&app, button), expected);
    }
}

#[test]
fn the_border_color_follows_the_state() {
    let mut app = test_app();
    let mut theme = probe_theme();
    primary_mut(&mut theme).border = StateColors {
        normal: Color::srgb(0.1, 0.0, 0.0),
        hovered: Color::srgb(0.2, 0.0, 0.0),
        pressed: Color::srgb(0.3, 0.0, 0.0),
        disabled: Color::srgb(0.4, 0.0, 0.0),
        focused: Color::srgb(0.5, 0.0, 0.0),
    };
    let border = primary(&theme).border;
    app.insert_resource(GlobalTheme(theme));
    let button = spawn_primary(&mut app);
    app.update();

    for (state, expected) in [
        (PickingInteraction::Hovered, border.hovered),
        (PickingInteraction::Pressed, border.pressed),
        (PickingInteraction::None, border.normal),
    ] {
        app.world_mut().entity_mut(button).insert(state);
        app.update();
        assert_eq!(border_color(&app, button), BorderColor::all(expected));
    }
}

#[test]
fn interaction_disabled_grays_the_button_and_removal_restores_it() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    let style = primary(&probe_theme());
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(bg(&app, button), style.bg.disabled);
    assert_eq!(label_color(&app, button), style.fg.disabled);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(bg(&app, button), style.bg.normal);
    assert_eq!(label_color(&app, button), style.fg.normal);
}

// ---------------------------------------------------------------------------
// Theme resolution
// ---------------------------------------------------------------------------

// [T] swap the global theme
#[test]
fn global_theme_change_updates_unscoped_buttons_but_not_scoped_ones() {
    let mut app = test_app();
    let unscoped = spawn_primary(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let scoped = spawn_primary_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(swapped_theme()));
    app.update();

    let swapped = swapped_theme();
    assert_eq!(bg(&app, unscoped), primary(&swapped).bg.normal);
    assert_eq!(label_font(&app, unscoped), swapped.font);
    assert_eq!(bg(&app, scoped), primary(&probe_theme()).bg.normal);
}

#[test]
fn theme_font_smoothing_is_applied_to_the_label_and_follows_theme_changes() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();
    assert_eq!(label_smoothing(&app, button), FontSmoothing::AntiAliased);

    app.insert_resource(GlobalTheme(Theme {
        font_smoothing: FontSmoothing::None,
        ..probe_theme()
    }));
    app.update();
    assert_eq!(label_smoothing(&app, button), FontSmoothing::None);
}

#[test]
fn surface_theme_tints_the_image_and_keeps_the_background_transparent() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(surface_theme()));
    let button = spawn_primary(&mut app);
    app.update();

    let style = primary(&surface_theme());
    let node = image_node(&app, button).expect("surface adds an ImageNode");
    assert_eq!(node.image, panel_image());
    assert!(matches!(node.image_mode, NodeImageMode::Sliced(_)));
    assert_eq!(node.visual_box, VisualBox::BorderBox);
    assert_eq!(node.color, style.bg.normal);
    assert_eq!(bg(&app, button), Color::NONE);

    for (state, expected) in [
        (PickingInteraction::Hovered, style.bg.hovered),
        (PickingInteraction::Pressed, style.bg.pressed),
    ] {
        app.world_mut().entity_mut(button).insert(state);
        app.update();
        assert_eq!(image_node(&app, button).unwrap().color, expected);
    }

    app.world_mut()
        .entity_mut(button)
        .insert((PickingInteraction::None, InteractionDisabled));
    app.update();
    assert_eq!(image_node(&app, button).unwrap().color, style.bg.disabled);
}

#[test]
fn swapping_to_a_theme_without_surface_removes_the_image_and_restores_the_background() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(surface_theme()));
    let button = spawn_primary(&mut app);
    app.update();
    assert!(image_node(&app, button).is_some());

    app.insert_resource(GlobalTheme(probe_theme()));
    app.update();
    assert!(image_node(&app, button).is_none());
    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.normal);

    app.insert_resource(GlobalTheme(surface_theme()));
    app.update();
    assert!(image_node(&app, button).is_some());
    assert_eq!(bg(&app, button), Color::NONE);
}

// A default `ImageNode` paints a white box, so a style without a surface must never add one.
#[test]
fn a_style_without_a_surface_never_adds_an_image_node() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    app.update();

    assert!(image_node(&app, button).is_none());
}

#[test]
fn scoped_theme_change_updates_only_buttons_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_primary(&mut app);
    let panel = app.world_mut().spawn(probe_theme()).id();
    let inside = spawn_primary_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = swapped_theme();
    app.update();

    assert_eq!(bg(&app, inside), primary(&swapped_theme()).bg.normal);
    assert_eq!(bg(&app, outside), primary(&probe_theme()).bg.normal);
}

#[test]
fn button_spawned_later_inside_a_scope_uses_the_scope_theme_immediately() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    app.update();

    let late = spawn_primary_under(&mut app, panel);
    app.update();

    assert_eq!(bg(&app, late), primary(&swapped_theme()).bg.normal);
    assert_eq!(label_font(&app, late), swapped_theme().font);
}

#[test]
fn removing_the_theme_component_makes_buttons_follow_the_global_theme() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    let button = spawn_primary_under(&mut app, panel);
    app.update();
    assert_eq!(bg(&app, button), primary(&swapped_theme()).bg.normal);

    app.world_mut().entity_mut(panel).remove::<Theme>();
    app.update();

    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.normal);
}

#[test]
fn moving_a_button_to_another_scope_makes_it_follow_that_scope() {
    let mut app = test_app();
    let swapped = app.world_mut().spawn(swapped_theme()).id();
    let plain = app.world_mut().spawn(probe_theme()).id();
    let button = spawn_primary_under(&mut app, swapped);
    app.update();

    app.world_mut().entity_mut(button).insert(ChildOf(plain));
    app.update();

    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.normal);
}

#[test]
fn detaching_a_button_from_a_scope_makes_it_follow_the_global_theme() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    let button = spawn_primary_under(&mut app, panel);
    app.update();

    app.world_mut().entity_mut(button).remove::<ChildOf>();
    app.update();

    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.normal);
}

// ---------------------------------------------------------------------------
// Size and shape
// ---------------------------------------------------------------------------

const ROW: ButtonSizeId = ButtonSizeId::new("row");
const OTHER: ButtonSizeId = ButtonSizeId::new("other");

#[test]
fn a_name_the_theme_does_not_define_falls_back_to_the_base_size() {
    let mut app = test_app();
    // The theme has sizes, but not this one: falling back is told from picking any entry.
    let mut theme = probe_theme();
    theme.button_sizes.named.insert(
        OTHER,
        ButtonMetrics {
            height: Val::Px(99.0),
            ..roomy_row()
        },
    );
    app.insert_resource(GlobalTheme(theme));
    let button = spawn_sized(&mut app, ROW);

    app.update();

    assert_eq!(
        node(&app, button).height,
        probe_theme().button_sizes.base.height
    );
}

#[test]
fn a_theme_change_resizes_and_reshapes_the_buttons_with_their_content() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn(
            icon_button("button")
                .icon(Handle::default())
                .size(ROW)
                .variant(ButtonVariantId::PRIMARY)
                .build(),
        )
        .id();
    app.update();

    app.insert_resource(GlobalTheme(roomy_theme()));
    app.update();

    let row = roomy_row();
    let node = node(&app, button);
    assert_eq!(node.height, row.height);
    assert_eq!(node.padding, row.padding);
    assert_eq!(node.column_gap, row.column_gap);
    assert_eq!(node.border, UiRect::all(Val::Px(0.0)));
    assert_eq!(node.border_radius, BorderRadius::all(Val::Px(0.0)));
    assert_eq!(label_size(&app, button), row.font_size);
    let icon = icon_size(&app, button);
    assert_eq!((icon.width, icon.height), (Val::Px(33.0), Val::Px(33.0)));
    assert_eq!(icon_tint(&app, button), primary(&roomy_theme()).fg.normal);

    // And back: the name is no longer defined, and nothing of the other theme is left.
    app.insert_resource(GlobalTheme(probe_theme()));
    app.update();
    assert_eq!(
        self::node(&app, button).height,
        probe_theme().button_sizes.base.height
    );
    assert_eq!(
        self::node(&app, button).border,
        UiRect::all(Val::Px(primary(&probe_theme()).shape.border_width))
    );
}

#[test]
fn changing_the_size_of_a_button_resizes_it() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(roomy_theme()));
    let button = app
        .world_mut()
        .spawn(
            icon_button("button")
                .variant(ButtonVariantId::PRIMARY)
                .build(),
        )
        .id();
    app.update();
    assert_eq!(
        node(&app, button).height,
        roomy_theme().button_sizes.base.height
    );

    app.world_mut()
        .entity_mut(button)
        .insert(ButtonSize::Named(ROW));
    app.update();

    assert_eq!(node(&app, button).height, roomy_row().height);
    assert_eq!(label_size(&app, button), roomy_row().font_size);
}

// ---------------------------------------------------------------------------
// Variants
// ---------------------------------------------------------------------------

const GOLD: ButtonVariantId = ButtonVariantId::new("gold");

#[test]
fn the_default_variant_is_the_base_style_of_the_theme() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(gold_theme()));
    let button = app.world_mut().spawn(icon_button("button").build()).id();

    app.update();

    assert_eq!(bg(&app, button), gold_theme().buttons.base.bg.normal);
}

#[test]
fn changing_the_variant_of_a_button_restyles_it() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(gold_theme()));
    let button = spawn_variant(&mut app, ButtonVariantId::PRIMARY);
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(ButtonVariant::Named(GOLD));
    app.update();

    assert_eq!(bg(&app, button), gold_style().bg.normal);
}

#[test]
fn a_theme_change_that_drops_a_name_restyles_its_buttons_with_the_base_style() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(gold_theme()));
    let button = spawn_variant(&mut app, GOLD);
    app.update();

    app.insert_resource(GlobalTheme(probe_theme()));
    app.update();

    assert_eq!(bg(&app, button), probe_theme().buttons.base.bg.normal);
}

// ---------------------------------------------------------------------------
// Focus
// ---------------------------------------------------------------------------

#[test]
fn visible_focus_draws_the_ring_and_uses_the_focused_colors() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, true);
    app.update();

    let style = primary(&probe_theme());
    assert_eq!(bg(&app, button), style.bg.focused);
    assert_eq!(label_color(&app, button), style.fg.focused);
    let ring = outline(&app, button);
    assert_eq!(ring.color, style.focus_ring.color);
    assert_eq!(ring.width, style.focus_ring.width);
}

#[test]
fn moving_focus_moves_the_ring() {
    let mut app = test_app();
    let first = spawn_primary(&mut app);
    let second = spawn_primary(&mut app);
    app.update();
    focus(&mut app, first, true);
    app.update();

    focus(&mut app, second, true);
    app.update();

    let style = primary(&probe_theme());
    assert_eq!(outline(&app, first).color, Color::NONE);
    assert_eq!(bg(&app, first), style.bg.normal);
    assert_eq!(outline(&app, second).color, style.focus_ring.color);
    assert_eq!(bg(&app, second), style.bg.focused);
}

#[test]
fn an_always_visible_indicator_draws_the_ring_while_bevy_hides_it() {
    let mut app = test_app();
    app.insert_resource(FocusIndicator::Always);
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, false);
    app.update();

    let style = primary(&probe_theme());
    assert_eq!(outline(&app, button).color, style.focus_ring.color);
    assert_eq!(bg(&app, button), style.bg.focused);

    app.insert_resource(FocusIndicator::Auto);
    app.update();
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[test]
fn hover_wins_over_focus_for_colors_but_the_ring_stays() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.world_mut()
        .entity_mut(button)
        .insert(PickingInteraction::Hovered);
    app.update();

    let style = primary(&probe_theme());
    assert_eq!(bg(&app, button), style.bg.hovered);
    assert_eq!(outline(&app, button).color, style.focus_ring.color);
}

#[test]
fn disabled_button_shows_neither_ring_nor_focused_colors() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(bg(&app, button), primary(&probe_theme()).bg.disabled);
    assert_eq!(outline(&app, button).color, Color::NONE);
}

// ---------------------------------------------------------------------------
// Tab order
// ---------------------------------------------------------------------------

#[test]
fn a_custom_tab_index_is_restored_after_re_enabling() {
    let mut app = test_app();
    let button = app.world_mut().spawn((spawn_bundle(), TabIndex(3))).id();
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(tab_index(&app, button), -1);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, button), 3);
}

#[test]
fn a_button_spawned_disabled_starts_outside_the_tab_order() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((spawn_bundle(), InteractionDisabled))
        .id();
    app.update();
    assert_eq!(tab_index(&app, button), -1);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, button), 0);
}

// End to end: Tab skips the disabled middle button, in both directions.
#[test]
fn tab_navigation_skips_a_disabled_button() {
    let mut app = test_app();
    let group = app.world_mut().spawn(TabGroup::default()).id();
    let first = spawn_primary_under(&mut app, group);
    let middle = spawn_primary_under(&mut app, group);
    let last = spawn_primary_under(&mut app, group);
    // Without the visibility plugins nothing is ever propagated, and Tab navigation skips hidden entities.
    for button in [first, middle, last] {
        app.world_mut()
            .entity_mut(button)
            .insert(InheritedVisibility::VISIBLE);
    }

    // Control: with every button enabled, Tab visits the middle one.
    app.update();
    assert_eq!(navigate(&mut app, first, NavAction::Next), middle);

    app.world_mut()
        .entity_mut(middle)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(navigate(&mut app, first, NavAction::Next), last);
    assert_eq!(navigate(&mut app, last, NavAction::Previous), first);
}

// Bevy's automatic arrow-key navigation ignores `InteractionDisabled` too.
#[test]
fn disabled_button_leaves_directional_navigation_and_returns_when_enabled() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((spawn_bundle(), AutoDirectionalNavigation::default()))
        .id();
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_none()
    );

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_some()
    );
}

// Arrow keys must not reach a disabled button, also when the component is added to it later.
#[test]
fn directional_navigation_added_to_a_disabled_button_is_held_back_until_it_is_enabled() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((spawn_bundle(), InteractionDisabled))
        .id();
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(AutoDirectionalNavigation::default());
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_none()
    );

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_some()
    );
}

// Re-enabling must not give directional navigation to a button that never had it.
#[test]
fn re_enabling_does_not_add_directional_navigation() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert!(
        app.world()
            .get::<AutoDirectionalNavigation>(button)
            .is_none()
    );
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

/// The probe theme with its Primary/Affirmative palettes exchanged and another font.
fn swapped_theme() -> Theme {
    let mut theme = probe_theme();
    let named = &mut theme.buttons.named;
    let primary = named.remove(&ButtonVariantId::PRIMARY);
    let affirmative = named.remove(&ButtonVariantId::AFFIRMATIVE);
    named.extend(affirmative.map(|style| (ButtonVariantId::PRIMARY, style)));
    named.extend(primary.map(|style| (ButtonVariantId::AFFIRMATIVE, style)));
    theme.font = FontSource::sans_serif();
    theme
}

fn spawn_primary(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(
            icon_button("button")
                .variant(ButtonVariantId::PRIMARY)
                .build(),
        )
        .id()
}

fn spawn_primary_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((
            icon_button("button")
                .variant(ButtonVariantId::PRIMARY)
                .build(),
            ChildOf(parent),
        ))
        .id()
}

fn focus(app: &mut App, button: Entity, visible: bool) {
    app.insert_resource(InputFocus::from_entity(button));
    app.insert_resource(InputFocusVisible(visible));
}

fn outline(app: &App, button: Entity) -> Outline {
    *app.world().get::<Outline>(button).unwrap()
}

fn spawn_bundle() -> impl Bundle {
    icon_button("button")
        .variant(ButtonVariantId::PRIMARY)
        .build()
}

fn tab_index(app: &App, button: Entity) -> i32 {
    app.world().get::<TabIndex>(button).unwrap().0
}

fn navigate(app: &mut App, from: Entity, action: NavAction) -> Entity {
    app.world_mut()
        .run_system_once(move |navigation: TabNavigation| {
            navigation.navigate(&InputFocus::from_entity(from), action)
        })
        .unwrap()
        .expect("tab navigation finds a target")
}

fn bg(app: &App, button: Entity) -> Color {
    app.world().get::<BackgroundColor>(button).unwrap().0
}

fn border_color(app: &App, button: Entity) -> BorderColor {
    *app.world().get::<BorderColor>(button).unwrap()
}

fn label_color(app: &App, button: Entity) -> Color {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextColor>(child).map(|c| c.0))
        .expect("button has a label child")
}

fn panel_image() -> Handle<Image> {
    uuid_handle!("6a1f5c3e-0b52-4d8e-9a77-3c1d2e4f5a60")
}

/// The probe theme with a 9-sliced image on every button.
fn surface_theme() -> Theme {
    let mut theme = probe_theme();
    let surface = SlicedImage {
        image: panel_image(),
        slicer: TextureSlicer {
            border: BorderRect::all(8.0),
            ..default()
        },
    };
    for style in theme.buttons.named.values_mut() {
        style.surface = Some(surface.clone());
    }
    theme
}

fn image_node(app: &App, button: Entity) -> Option<&ImageNode> {
    app.world().get::<ImageNode>(button)
}

fn label_smoothing(app: &App, button: Entity) -> FontSmoothing {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextFont>(child).map(|f| f.font_smoothing))
        .expect("button has a label child")
}

fn label_layout(app: &App, button: Entity) -> TextLayout {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextLayout>(child).copied())
        .expect("button has a label child")
}

fn label_font(app: &App, button: Entity) -> FontSource {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextFont>(child).map(|f| f.font.clone()))
        .expect("button has a label child")
}

fn spawn_sized(app: &mut App, size: impl Into<ButtonSize>) -> Entity {
    app.world_mut()
        .spawn(
            icon_button("button")
                .size(size)
                .variant(ButtonVariantId::PRIMARY)
                .build(),
        )
        .id()
}

fn node(app: &App, button: Entity) -> Node {
    app.world().get::<Node>(button).unwrap().clone()
}

/// The `ROW` size of `roomy_theme`: every field differs from the probe theme's base size.
fn roomy_row() -> ButtonMetrics {
    ButtonMetrics {
        height: Val::Px(33.0),
        padding: UiRect::all(Val::Px(3.0)),
        column_gap: Val::Px(3.0),
        font_size: FontSize::Px(33.0),
        icon_size: 33.0,
    }
}

/// A theme that defines `ROW`, has its own base size, and gives the primary buttons another shape and text color.
fn roomy_theme() -> Theme {
    let mut theme = probe_theme();
    theme.button_sizes.named.insert(ROW, roomy_row());
    // Another name, so that picking by name is told from picking any entry.
    theme.button_sizes.named.insert(
        OTHER,
        ButtonMetrics {
            height: Val::Px(99.0),
            ..roomy_row()
        },
    );
    theme.button_sizes.base = ButtonMetrics {
        height: Val::Px(77.0),
        ..roomy_row()
    };
    // Not white, which is what an icon starts with.
    primary_mut(&mut theme).fg = StateColors::uniform(Color::srgb(0.9, 0.1, 0.1));
    primary_mut(&mut theme).shape = ButtonShape {
        border_width: 0.0,
        radius: 0.0,
    };
    theme
}

fn label_size(app: &App, button: Entity) -> FontSize {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<TextFont>(child).map(|f| f.font_size))
        .expect("button has a label child")
}

fn icon_tint(app: &App, button: Entity) -> Color {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find_map(|child| world.get::<ImageNode>(child).map(|icon| icon.color))
        .expect("button has an icon child")
}

fn icon_size(app: &App, button: Entity) -> Node {
    let world = app.world();
    world
        .get::<Children>(button)
        .unwrap()
        .iter()
        .find(|child| world.get::<ImageNode>(*child).is_some())
        .map(|icon| world.get::<Node>(icon).unwrap().clone())
        .expect("button has an icon child")
}

fn primary(theme: &Theme) -> ButtonStyle {
    theme.buttons.named[&ButtonVariantId::PRIMARY].clone()
}

fn primary_mut(theme: &mut Theme) -> &mut ButtonStyle {
    theme
        .buttons
        .named
        .get_mut(&ButtonVariantId::PRIMARY)
        .unwrap()
}

fn spawn_variant(app: &mut App, variant: impl Into<ButtonVariant>) -> Entity {
    app.world_mut()
        .spawn(icon_button("button").variant(variant).build())
        .id()
}

/// A style that differs from every style of the probe theme.
fn gold_style() -> ButtonStyle {
    let mut style = ButtonStyle::ghost(StateColors::uniform(Color::srgb(1.0, 0.85, 0.3)));
    style.bg = StateColors::uniform(Color::srgb(0.3, 0.2, 0.0));
    style
}

/// The probe theme plus `GOLD`.
fn gold_theme() -> Theme {
    let mut theme = probe_theme();
    theme.buttons.named.insert(GOLD, gold_style());
    theme
}
