//! Tests of size and shape.

use super::{primary, primary_mut};
use crate::{
    probe::probe_theme,
    support::{node, test_app},
};
use bevy::prelude::*;
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme,
    button::{
        ButtonLabel, ButtonMetrics, ButtonShape, ButtonSize, ButtonSizeId, ButtonVariant,
        ButtonVariantId, IconButton, KeepFontSize, icon_button,
    },
    common::StateColors,
};

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
            icon_button()
                .label("button")
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
            icon_button()
                .label("button")
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

#[test]
fn a_label_that_keeps_its_font_size_is_colored_but_not_resized() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(roomy_theme()));
    let small = FontSize::Px(11.0);
    let text = |label: &'static str| {
        (
            Text::new(label),
            TextFont {
                font_size: small,
                ..default()
            },
        )
    };
    // A card: the labels sit at different depths, as in content the builder cannot make.
    let card = app
        .world_mut()
        .spawn((
            IconButton,
            ButtonSize::Named(ROW),
            ButtonVariant::Named(ButtonVariantId::PRIMARY),
            children![
                (ButtonLabel, text("sized")),
                (
                    Node::default(),
                    children![(ButtonLabel, KeepFontSize, text("kept"))]
                ),
            ],
        ))
        .id();

    app.update();

    let world = app.world();
    let label = |text: &str| {
        world
            .iter_entities()
            .find(|e| e.get::<Text>().is_some_and(|t| t.0 == text))
            .map(|e| {
                (
                    e.get::<TextFont>().unwrap().font_size,
                    e.get::<TextColor>().unwrap().0,
                )
            })
            .unwrap()
    };
    let fg = primary(&roomy_theme()).fg.normal;
    assert_eq!(label("sized"), (roomy_row().font_size, fg));
    assert_eq!(label("kept"), (small, fg));
    assert_eq!(node(&app, card).height, roomy_row().height);
}

#[test]
fn a_label_that_stops_keeping_its_font_size_is_resized_with_no_other_change() {
    let mut app = test_app();
    app.insert_resource(GlobalTheme(roomy_theme()));
    let small = FontSize::Px(11.0);
    let card = app
        .world_mut()
        .spawn((
            IconButton,
            ButtonSize::Named(ROW),
            ButtonVariant::Named(ButtonVariantId::PRIMARY),
            children![(
                Node::default(),
                children![(
                    ButtonLabel,
                    KeepFontSize,
                    Text::new("nested"),
                    TextFont {
                        font_size: small,
                        ..default()
                    },
                )]
            )],
        ))
        .id();
    app.update();
    let label = app
        .world()
        .iter_entities()
        .find(|e| e.get::<Text>().is_some_and(|t| t.0 == "nested"))
        .map(|e| e.id())
        .unwrap();
    let size = |app: &App| app.world().get::<TextFont>(label).unwrap().font_size;
    assert_eq!(size(&app), small);

    app.world_mut().entity_mut(label).remove::<KeepFontSize>();
    app.update();

    assert_eq!(size(&app), roomy_row().font_size);
    assert_eq!(node(&app, card).height, roomy_row().height);
}

#[test]
fn a_keeping_label_despawned_or_outside_any_button_is_harmless() {
    let mut app = test_app();
    let button = app
        .world_mut()
        .spawn((
            IconButton,
            children![(ButtonLabel, KeepFontSize, Text::new("a"))],
        ))
        .id();
    let stray = app
        .world_mut()
        .spawn((ButtonLabel, KeepFontSize, Text::new("stray")))
        .id();
    app.update();
    let label = app.world().get::<Children>(button).unwrap()[0];

    app.world_mut().entity_mut(label).despawn();
    app.world_mut().entity_mut(stray).remove::<KeepFontSize>();
    app.update();

    assert!(app.world().get_entity(button).is_ok());
}

fn spawn_sized(app: &mut App, size: impl Into<ButtonSize>) -> Entity {
    app.world_mut()
        .spawn(
            icon_button()
                .label("button")
                .size(size)
                .variant(ButtonVariantId::PRIMARY)
                .build(),
        )
        .id()
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
