//! Behavior tests for button styling and theme resolution.
//!
//! Real pointer picking, layout, and text shaping are not covered by `examples/button_check.rs`.

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

// ---------------------------------------------------------------------------
// Initial styling
// ---------------------------------------------------------------------------

#[test]
fn newly_spawned_button_is_styled_after_one_update() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);

    app.update();

    let expected = Theme::default();
    assert_eq!(bg(&app, button), primary(&expected).bg.normal);
    assert_eq!(label_color(&app, button), primary(&expected).fg.normal);
    assert_eq!(label_font(&app, button), expected.font);
}

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

    assert_eq!(bg(&app, entity), primary(&Theme::default()).bg.normal);
}

#[test]
fn a_ghost_style_leaves_the_button_without_a_background_and_colors_only_the_label() {
    let mut app = test_app();
    let fg = StateColors::uniform(Color::srgb(0.2, 0.9, 0.3));
    let mut theme = Theme::default();
    *primary_mut(&mut theme) = ButtonStyle::ghost(fg);
    app.insert_resource(GlobalTheme(theme));
    let button = spawn_primary(&mut app);

    app.update();

    assert_eq!(bg(&app, button), Color::NONE);
    assert_eq!(label_color(&app, button), fg.normal);
}

#[test]
fn a_ghost_button_takes_no_room_for_a_border() {
    let mut app = test_app();
    let mut theme = Theme::default();
    *primary_mut(&mut theme) = ButtonStyle::ghost(StateColors::uniform(Color::WHITE));
    app.insert_resource(GlobalTheme(theme));
    let button = spawn_primary(&mut app);

    app.update();

    assert_eq!(node(&app, button).border, UiRect::all(Val::Px(0.0)));
}

// ---------------------------------------------------------------------------
// Interaction states
// ---------------------------------------------------------------------------

#[test]
fn hover_and_press_change_colors_and_release_restores_them() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    let style = primary(&Theme::default());
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
fn interaction_disabled_grays_the_button_and_removal_restores_it() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    let style = primary(&Theme::default());
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
    let panel = app.world_mut().spawn(Theme::default()).id();
    let scoped = spawn_primary_under(&mut app, panel);
    app.update();

    app.insert_resource(GlobalTheme(swapped_theme()));
    app.update();

    let swapped = swapped_theme();
    assert_eq!(bg(&app, unscoped), primary(&swapped).bg.normal);
    assert_eq!(label_font(&app, unscoped), swapped.font);
    assert_eq!(bg(&app, scoped), primary(&Theme::default()).bg.normal);
}

#[test]
fn theme_font_smoothing_is_applied_to_the_label_and_follows_theme_changes() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();
    assert_eq!(label_smoothing(&app, button), FontSmoothing::AntiAliased);

    app.insert_resource(GlobalTheme(Theme {
        font_smoothing: FontSmoothing::None,
        ..default()
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

    app.insert_resource(GlobalTheme(Theme::default()));
    app.update();
    assert!(image_node(&app, button).is_none());
    assert_eq!(bg(&app, button), primary(&Theme::default()).bg.normal);

    app.insert_resource(GlobalTheme(surface_theme()));
    app.update();
    assert!(image_node(&app, button).is_some());
    assert_eq!(bg(&app, button), Color::NONE);
}

#[test]
fn default_theme_never_adds_an_image_node() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
    app.update();
    assert!(image_node(&app, button).is_none());
}

#[test]
fn scoped_theme_change_updates_only_buttons_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_primary(&mut app);
    let panel = app.world_mut().spawn(Theme::default()).id();
    let inside = spawn_primary_under(&mut app, panel);
    app.update();

    *app.world_mut().get_mut::<Theme>(panel).unwrap() = swapped_theme();
    app.update();

    assert_eq!(bg(&app, inside), primary(&swapped_theme()).bg.normal);
    assert_eq!(bg(&app, outside), primary(&Theme::default()).bg.normal);
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

    assert_eq!(bg(&app, button), primary(&Theme::default()).bg.normal);
}

#[test]
fn detaching_a_button_from_a_scope_makes_it_follow_the_global_theme() {
    let mut app = test_app();
    let panel = app.world_mut().spawn(swapped_theme()).id();
    let button = spawn_primary_under(&mut app, panel);
    app.update();

    app.world_mut().entity_mut(button).remove::<ChildOf>();
    app.update();

    assert_eq!(bg(&app, button), primary(&Theme::default()).bg.normal);
}

// ---------------------------------------------------------------------------
// Size and shape
// ---------------------------------------------------------------------------

const ROW: ButtonSizeId = ButtonSizeId::new("row");

#[test]
fn a_button_takes_its_size_and_shape_from_the_theme_after_one_update() {
    let mut app = test_app();
    let button = spawn_sized(&mut app, ButtonSizeId::MEDIUM);

    app.update();

    let theme = Theme::default();
    let metrics = theme.button_sizes.named[&ButtonSizeId::MEDIUM];
    let shape = primary(&theme).shape;
    let node = node(&app, button);
    assert_eq!(node.height, metrics.height);
    assert_eq!(node.padding, metrics.padding);
    assert_eq!(node.column_gap, metrics.column_gap);
    assert_eq!(node.border, UiRect::all(Val::Px(shape.border_width)));
    assert_eq!(node.border_radius, BorderRadius::all(Val::Px(shape.radius)));
    assert_eq!(label_size(&app, button), metrics.font_size);
}

#[test]
fn the_default_theme_defines_three_sizes_that_grow_in_height_and_text() {
    let mut app = test_app();
    let buttons = [
        ButtonSizeId::SMALL,
        ButtonSizeId::MEDIUM,
        ButtonSizeId::LARGE,
    ]
    .map(|id| spawn_sized(&mut app, id));

    app.update();

    let heights = buttons.map(|button| px(node(&app, button).height));
    let texts = buttons.map(|button| px_font(label_size(&app, button)));
    assert!(heights[0] < heights[1] && heights[1] < heights[2]);
    assert!(texts[0] < texts[1] && texts[1] < texts[2]);
}

#[test]
fn a_name_of_the_apps_own_is_defined_by_the_theme() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(roomy_theme()));
    let button = spawn_sized(&mut app, ROW);

    app.update();

    assert_eq!(node(&app, button).height, roomy_row().height);
    assert_eq!(node(&app, button).padding, roomy_row().padding);
}

#[test]
fn the_default_size_is_the_base_size_of_the_theme() {
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
}

#[test]
fn a_name_the_theme_does_not_define_falls_back_to_the_base_size() {
    let mut app = test_app();
    let button = spawn_sized(&mut app, ROW);

    app.update();

    assert_eq!(
        node(&app, button).height,
        Theme::default().button_sizes.base.height
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
    assert_eq!(node.border, UiRect::all(Val::Px(0.0)));
    assert_eq!(node.border_radius, BorderRadius::all(Val::Px(0.0)));
    assert_eq!(label_size(&app, button), row.font_size);
    let icon = icon_size(&app, button);
    assert_eq!((icon.width, icon.height), (Val::Px(33.0), Val::Px(33.0)));

    // And back: the name is no longer defined, and nothing of the other theme is left.
    app.insert_resource(GlobalTheme(Theme::default()));
    app.update();
    assert_eq!(
        self::node(&app, button).height,
        Theme::default().button_sizes.base.height
    );
    assert_eq!(
        self::node(&app, button).border,
        UiRect::all(Val::Px(primary(&Theme::default()).shape.border_width))
    );
}

#[test]
fn a_scoped_theme_resizes_only_the_buttons_inside_the_scope() {
    let mut app = test_app();
    let outside = spawn_sized(&mut app, ROW);
    let panel = app.world_mut().spawn(roomy_theme()).id();
    let inside = app
        .world_mut()
        .spawn((
            icon_button("button")
                .size(ROW)
                .variant(ButtonVariantId::PRIMARY)
                .build(),
            ChildOf(panel),
        ))
        .id();

    app.update();

    assert_eq!(node(&app, inside).height, roomy_row().height);
    assert_eq!(
        node(&app, outside).height,
        Theme::default().button_sizes.base.height
    );
}

#[test]
fn changing_the_size_of_a_button_resizes_it() {
    let mut app = test_app();
    let button = spawn_sized(&mut app, ButtonSizeId::SMALL);
    app.update();

    app.world_mut()
        .entity_mut(button)
        .insert(ButtonSize::Named(ButtonSizeId::LARGE));
    app.update();

    let large = Theme::default().button_sizes.named[&ButtonSizeId::LARGE];
    assert_eq!(node(&app, button).height, large.height);
    assert_eq!(label_size(&app, button), large.font_size);
}

// ---------------------------------------------------------------------------
// Variants
// ---------------------------------------------------------------------------

const GOLD: ButtonVariantId = ButtonVariantId::new("gold");

#[test]
fn a_name_of_the_apps_own_takes_its_style_from_the_theme() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(gold_theme()));
    let button = spawn_variant(&mut app, GOLD);

    app.update();

    assert_eq!(bg(&app, button), gold_style().bg.normal);
    assert_eq!(label_color(&app, button), gold_style().fg.normal);
}

#[test]
fn the_default_variant_is_the_base_style_of_the_theme() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(gold_theme()));
    let button = app.world_mut().spawn(icon_button("button").build()).id();

    app.update();

    assert_eq!(bg(&app, button), gold_theme().buttons.base.bg.normal);
}

#[test]
fn a_name_the_theme_does_not_define_falls_back_to_the_base_style() {
    let mut app = test_app();
    let button = spawn_variant(&mut app, GOLD);

    app.update();

    assert_eq!(bg(&app, button), Theme::default().buttons.base.bg.normal);
}

#[test]
fn the_default_theme_defines_four_distinct_variants() {
    let mut app = test_app();
    let ids = [
        ButtonVariantId::NEUTRAL,
        ButtonVariantId::PRIMARY,
        ButtonVariantId::AFFIRMATIVE,
        ButtonVariantId::DANGER,
    ];
    let buttons = ids.map(|id| spawn_variant(&mut app, id));

    app.update();

    let colors = buttons.map(|button| bg(&app, button));
    for (index, color) in colors.iter().enumerate() {
        assert_eq!(
            *color,
            Theme::default().buttons.named[&ids[index]].bg.normal
        );
        assert!(!colors[index + 1..].contains(color));
    }
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

    app.insert_resource(GlobalTheme(Theme::default()));
    app.update();

    assert_eq!(bg(&app, button), Theme::default().buttons.base.bg.normal);
}

// ---------------------------------------------------------------------------
// Focus
// ---------------------------------------------------------------------------

#[test]
fn visible_focus_draws_the_ring_and_uses_the_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, true);
    app.update();

    let style = primary(&focus_theme());
    assert_eq!(bg(&app, button), style.bg.focused);
    assert_eq!(label_color(&app, button), style.fg.focused);
    let ring = outline(&app, button);
    assert_eq!(ring.color, style.focus_ring.color);
    assert_eq!(ring.width, style.focus_ring.width);
}

#[test]
fn focus_is_not_shown_while_the_indicator_is_hidden() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, false);
    app.update();

    assert_eq!(bg(&app, button), primary(&focus_theme()).bg.normal);
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[test]
fn hiding_the_indicator_removes_the_ring_and_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.update();

    app.world_mut().resource_mut::<InputFocusVisible>().0 = false;
    app.update();

    assert_eq!(bg(&app, button), primary(&focus_theme()).bg.normal);
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[test]
fn moving_focus_moves_the_ring() {
    let mut app = test_app_with_focus_theme();
    let first = spawn_primary(&mut app);
    let second = spawn_primary(&mut app);
    app.update();
    focus(&mut app, first, true);
    app.update();

    focus(&mut app, second, true);
    app.update();

    let style = primary(&focus_theme());
    assert_eq!(outline(&app, first).color, Color::NONE);
    assert_eq!(bg(&app, first), style.bg.normal);
    assert_eq!(outline(&app, second).color, style.focus_ring.color);
    assert_eq!(bg(&app, second), style.bg.focused);
}

#[test]
fn an_always_visible_indicator_draws_the_ring_while_bevy_hides_it() {
    let mut app = test_app_with_focus_theme();
    app.insert_resource(FocusIndicator::Always);
    let button = spawn_primary(&mut app);
    app.update();

    focus(&mut app, button, false);
    app.update();

    let style = primary(&focus_theme());
    assert_eq!(outline(&app, button).color, style.focus_ring.color);
    assert_eq!(bg(&app, button), style.bg.focused);

    app.insert_resource(FocusIndicator::Auto);
    app.update();
    assert_eq!(outline(&app, button).color, Color::NONE);
}

#[test]
fn hover_wins_over_focus_for_colors_but_the_ring_stays() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.world_mut()
        .entity_mut(button)
        .insert(PickingInteraction::Hovered);
    app.update();

    let style = primary(&focus_theme());
    assert_eq!(bg(&app, button), style.bg.hovered);
    assert_eq!(outline(&app, button).color, style.focus_ring.color);
}

#[test]
fn disabled_button_shows_neither_ring_nor_focused_colors() {
    let mut app = test_app_with_focus_theme();
    let button = spawn_primary(&mut app);
    app.update();
    focus(&mut app, button, true);
    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();

    assert_eq!(bg(&app, button), primary(&focus_theme()).bg.disabled);
    assert_eq!(outline(&app, button).color, Color::NONE);
}

// ---------------------------------------------------------------------------
// Tab order
// ---------------------------------------------------------------------------

#[test]
fn disabled_button_leaves_the_tab_order_and_returns_when_enabled() {
    let mut app = test_app();
    let button = spawn_primary(&mut app);
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
    assert_eq!(tab_index(&app, button), 0);
}

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

#[test]
fn a_button_spawned_disabled_does_not_join_directional_navigation() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((
            spawn_bundle(),
            AutoDirectionalNavigation::default(),
            InteractionDisabled,
        ))
        .id();
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
    app
}

/// A theme whose Primary/Affirmative palettes are exchanged and whose font differs
/// so that it is distinguishable from `Theme::default()`.
fn swapped_theme() -> Theme {
    let mut theme = Theme::default();
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
                .size(ButtonSizeId::SMALL)
                .variant(ButtonVariantId::PRIMARY)
                .build(),
        )
        .id()
}

fn spawn_primary_under(app: &mut App, parent: Entity) -> Entity {
    app.world_mut()
        .spawn((
            icon_button("button")
                .size(ButtonSizeId::SMALL)
                .variant(ButtonVariantId::PRIMARY)
                .build(),
            ChildOf(parent),
        ))
        .id()
}

/// Default theme whose `focused` colors differ from every other state, so a test can tell them apart.
fn focus_theme() -> Theme {
    let mut theme = Theme::default();
    let style = primary_mut(&mut theme);
    style.bg.focused = Color::srgb(0.1, 0.2, 0.3);
    style.fg.focused = Color::srgb(0.9, 0.8, 0.7);
    theme
}

fn test_app_with_focus_theme() -> App {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(focus_theme()));
    app
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
        .size(ButtonSizeId::SMALL)
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

/// Default theme with a 9-sliced image on every button.
fn surface_theme() -> Theme {
    let mut theme = Theme::default();
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

/// The `ROW` size of `roomy_theme`: every field differs from the default.
fn roomy_row() -> ButtonMetrics {
    ButtonMetrics {
        height: Val::Px(33.0),
        padding: UiRect::all(Val::Px(3.0)),
        column_gap: Val::Px(3.0),
        font_size: FontSize::Px(33.0),
        icon_size: 33.0,
    }
}

/// A theme that defines `ROW`, has its own base size, and gives the primary buttons another shape.
fn roomy_theme() -> Theme {
    let mut theme = Theme::default();
    theme.button_sizes.named.insert(ROW, roomy_row());
    theme.button_sizes.base = ButtonMetrics {
        height: Val::Px(77.0),
        ..roomy_row()
    };
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

fn px(value: Val) -> f32 {
    match value {
        Val::Px(px) => px,
        other => panic!("not pixels: {other:?}"),
    }
}

fn px_font(size: FontSize) -> f32 {
    match size {
        FontSize::Px(px) => px,
        other => panic!("not pixels: {other:?}"),
    }
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

/// A style that differs from every style of the default theme.
fn gold_style() -> ButtonStyle {
    let mut style = ButtonStyle::ghost(StateColors::uniform(Color::srgb(1.0, 0.85, 0.3)));
    style.bg = StateColors::uniform(Color::srgb(0.3, 0.2, 0.0));
    style
}

/// The default theme plus `GOLD`, and a base style of its own.
fn gold_theme() -> Theme {
    let mut theme = Theme::default();
    theme.buttons.named.insert(GOLD, gold_style());
    theme.buttons.base.bg = StateColors::uniform(Color::srgb(0.0, 0.2, 0.3));
    theme
}
