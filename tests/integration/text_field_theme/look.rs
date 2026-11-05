//! The look of the field in each state, and theme resolution.

use super::{input, spawn_field, text_color};
use crate::{
    probe::probe_theme,
    support::{bg, border_color, focus, node, outline, test_app},
};
use bevy::{
    input_focus::InputFocusVisible, picking::hover::Hovered, prelude::*, text::TextCursorStyle,
    ui::InteractionDisabled,
};
use bevy_ui_component_prototypes::{GlobalTheme, Theme};

#[test]
fn an_idle_field_has_the_normal_colors_of_the_theme() {
    let mut app = test_app();
    let style = probe_theme().text_field;
    let field = spawn_field(&mut app);

    app.update();

    assert_eq!(bg(&app, field), style.bg.normal);
    assert_eq!(
        border_color(&app, field),
        BorderColor::all(style.border.normal)
    );
    assert_eq!(text_color(&app, input(&app, field)), style.text.normal);
}

#[test]
fn the_pointer_over_the_field_gives_the_hovered_colors() {
    let mut app = test_app();
    let style = probe_theme().text_field;
    let field = spawn_field(&mut app);
    app.update();

    app.world_mut().entity_mut(field).insert(Hovered(true));
    app.update();

    assert_eq!(bg(&app, field), style.bg.hovered);
    assert_eq!(
        border_color(&app, field),
        BorderColor::all(style.border.hovered)
    );
}

#[test]
fn the_focus_gives_the_focused_colors_even_when_the_indicator_is_hidden() {
    let mut app = test_app();
    let style = probe_theme().text_field;
    let field = spawn_field(&mut app);
    app.update();
    let input = input(&app, field);

    focus(&mut app, input, false);
    app.update();

    assert_eq!(bg(&app, field), style.bg.focused);
    assert_eq!(
        border_color(&app, field),
        BorderColor::all(style.border.focused)
    );
    assert_eq!(outline(&app, field).width, Val::Px(0.0));
}

#[test]
fn the_ring_is_drawn_on_the_frame_while_the_indicator_is_visible() {
    let mut app = test_app();
    let ring = probe_theme().text_field.focus_ring;
    let field = spawn_field(&mut app);
    app.update();
    let input = input(&app, field);

    focus(&mut app, input, true);
    app.update();
    app.update();
    let shown = outline(&app, field);
    assert_eq!(
        (shown.color, shown.width, shown.offset),
        (ring.color, ring.width, ring.offset)
    );

    app.insert_resource(InputFocusVisible(false));
    app.update();
    app.update();
    assert_eq!(outline(&app, field).color, Color::NONE);
}

#[test]
fn losing_the_focus_gives_the_colors_back() {
    let mut app = test_app();
    let style = probe_theme().text_field;
    let other = app.world_mut().spawn_empty().id();
    let field = spawn_field(&mut app);
    app.update();
    let input = input(&app, field);
    focus(&mut app, input, true);
    app.update();

    focus(&mut app, other, true);
    app.update();

    assert_eq!(bg(&app, field), style.bg.normal);
    assert_eq!(
        border_color(&app, field),
        BorderColor::all(style.border.normal)
    );
}

#[test]
fn a_disabled_field_has_the_disabled_colors_and_no_ring() {
    let mut app = test_app();
    let style = probe_theme().text_field;
    let field = spawn_field(&mut app);
    app.update();
    let input = input(&app, field);
    focus(&mut app, input, true);
    app.update();

    app.world_mut()
        .entity_mut(field)
        .insert(InteractionDisabled);
    app.update();
    app.update();

    assert_eq!(bg(&app, field), style.bg.disabled);
    assert_eq!(text_color(&app, input), style.text.disabled);
    assert_eq!(outline(&app, field).color, Color::NONE);
}

#[test]
fn the_frame_and_the_text_follow_the_shape_and_the_metrics_of_the_theme() {
    let mut app = test_app();
    let theme = probe_theme();
    let field = spawn_field(&mut app);

    app.update();

    let node = node(&app, field);
    assert_eq!(
        node.border,
        UiRect::all(px(theme.text_field.shape.border_width))
    );
    assert_eq!(
        node.border_radius,
        BorderRadius::all(px(theme.text_field.shape.radius))
    );
    assert_eq!(node.padding, theme.text_field_metrics.padding);
    let font = app.world().get::<TextFont>(input(&app, field)).unwrap();
    assert_eq!(font.font_size, theme.text_field_metrics.font_size);
}

#[test]
fn the_cursor_and_the_selection_take_their_colors_from_the_theme() {
    let mut app = test_app();
    let style = probe_theme().text_field;
    let field = spawn_field(&mut app);

    app.update();

    let cursor = app
        .world()
        .get::<TextCursorStyle>(input(&app, field))
        .unwrap();
    assert_eq!(cursor.color, style.cursor);
    assert_eq!(cursor.selection_color, style.selection);
    assert_eq!(cursor.unfocused_selection_color, style.unfocused_selection);
}

#[test]
fn a_theme_on_an_ancestor_gives_the_field_its_look() {
    let mut app = test_app();
    let mut scoped = probe_theme();
    scoped.text_field.bg.normal = Color::srgb(0.1, 0.9, 0.2);
    let panel = app.world_mut().spawn(scoped).id();
    let field = spawn_field(&mut app);
    app.world_mut().entity_mut(field).insert(ChildOf(panel));

    app.update();

    assert_eq!(bg(&app, field), Color::srgb(0.1, 0.9, 0.2));
}

#[test]
fn changing_the_global_theme_changes_the_look() {
    let mut app = test_app();
    let field = spawn_field(&mut app);
    app.update();

    let mut theme = Theme::default();
    theme.text_field.bg.normal = Color::srgb(0.9, 0.2, 0.1);
    app.insert_resource(GlobalTheme(theme));
    app.update();

    assert_eq!(bg(&app, field), Color::srgb(0.9, 0.2, 0.1));
}
