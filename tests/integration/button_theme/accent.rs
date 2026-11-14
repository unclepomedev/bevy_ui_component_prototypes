//! Tests of the accent: content that takes its own color from the style.

use super::{primary, primary_mut};
use crate::{probe::probe_theme, support::test_app};
use bevy::{prelude::*, ui::InteractionDisabled};
use bevy_ui_component_prototypes::{
    GlobalTheme,
    button::{ButtonAccent, ButtonIcon, ButtonLabel, ButtonVariant, ButtonVariantId, IconButton},
};

struct Card {
    button: Entity,
    plain: Entity,
    accented: Entity,
    icon: Entity,
}

fn spawn_card(app: &mut App) -> Card {
    let world = app.world_mut();
    let button = world
        .spawn((IconButton, ButtonVariant::Named(ButtonVariantId::PRIMARY)))
        .id();
    let plain = world
        .spawn((ButtonLabel, Text::new("plain"), ChildOf(button)))
        .id();
    let accented = world
        .spawn((
            ButtonLabel,
            ButtonAccent,
            Text::new("accented"),
            ChildOf(button),
        ))
        .id();
    let icon = world
        .spawn((
            ButtonIcon,
            ButtonAccent,
            ImageNode::default(),
            Node::default(),
            ChildOf(button),
        ))
        .id();
    Card {
        button,
        plain,
        accented,
        icon,
    }
}

fn text_color(app: &App, entity: Entity) -> Color {
    app.world().get::<TextColor>(entity).unwrap().0
}

fn icon_color(app: &App, entity: Entity) -> Color {
    app.world().get::<ImageNode>(entity).unwrap().color
}

#[test]
fn accented_content_takes_the_accent_and_the_rest_takes_fg() {
    let mut app = test_app();
    let theme = probe_theme();
    app.insert_resource(GlobalTheme(theme.clone()));
    let card = spawn_card(&mut app);

    app.update();

    let style = primary(&theme);
    let accent = style.accent.expect("the probe style has an accent");
    assert_ne!(accent.normal, style.fg.normal);
    assert_eq!(text_color(&app, card.plain), style.fg.normal);
    assert_eq!(text_color(&app, card.accented), accent.normal);
    assert_eq!(icon_color(&app, card.icon), accent.normal);
}

#[test]
fn a_style_without_an_accent_colors_accented_content_with_fg() {
    let mut app = test_app();
    let mut theme = probe_theme();
    primary_mut(&mut theme).accent = None;
    app.insert_resource(GlobalTheme(theme.clone()));
    let card = spawn_card(&mut app);

    app.update();

    let fg = primary(&theme).fg.normal;
    assert_eq!(text_color(&app, card.accented), fg);
    assert_eq!(icon_color(&app, card.icon), fg);
}

#[test]
fn the_accent_follows_the_state_of_the_button() {
    let mut app = test_app();
    let theme = probe_theme();
    app.insert_resource(GlobalTheme(theme.clone()));
    let card = spawn_card(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(card.button)
        .insert(InteractionDisabled);
    app.update();

    let accent = primary(&theme).accent.unwrap();
    assert_eq!(text_color(&app, card.accented), accent.disabled);
    assert_eq!(icon_color(&app, card.icon), accent.disabled);
}

#[test]
fn marking_or_unmarking_content_recolors_it_with_no_other_change() {
    let mut app = test_app();
    let theme = probe_theme();
    app.insert_resource(GlobalTheme(theme.clone()));
    let card = spawn_card(&mut app);
    app.update();
    let style = primary(&theme);
    let accent = style.accent.unwrap();

    app.world_mut().entity_mut(card.plain).insert(ButtonAccent);
    app.update();
    assert_eq!(text_color(&app, card.plain), accent.normal);

    app.world_mut()
        .entity_mut(card.accented)
        .remove::<ButtonAccent>();
    app.update();
    assert_eq!(text_color(&app, card.accented), style.fg.normal);
}
