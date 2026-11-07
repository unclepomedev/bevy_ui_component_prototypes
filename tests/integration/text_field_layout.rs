//! Layout tests for the text field, with the real layout of `bevy_ui` and no window or renderer.
//!
//! Each test builds its own app: the app of the other integration tests has no layout.

use bevy::{
    anti_alias::AntiAliasPlugin,
    audio::AudioPlugin,
    camera::RenderTarget,
    core_pipeline::CorePipelinePlugin,
    gilrs::GilrsPlugin,
    gizmos::GizmoPlugin,
    gizmos_render::GizmoRenderPlugin,
    gltf::GltfPlugin,
    light::LightPlugin,
    log::LogPlugin,
    pbr::PbrPlugin,
    post_process::PostProcessPlugin,
    prelude::*,
    render::RenderPlugin,
    sprite_render::SpriteRenderPlugin,
    text::TextLayoutInfo,
    ui::{ComputedNode, UiGlobalTransform},
    ui_render::UiRenderPlugin,
    window::ExitCondition,
    winit::WinitPlugin,
};
use bevy_ui_component_prototypes::{ScrollIndicator, UiComponentPrototypesPlugin, text_field};

/// An app that lays out UI for a target of 800 x 600, and a column of 400 x 300 to put fields in.
fn layout_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<WinitPlugin>()
            .disable::<RenderPlugin>()
            .disable::<PbrPlugin>()
            .disable::<UiRenderPlugin>()
            .disable::<SpriteRenderPlugin>()
            .disable::<CorePipelinePlugin>()
            .disable::<PostProcessPlugin>()
            .disable::<AntiAliasPlugin>()
            .disable::<GizmoPlugin>()
            .disable::<GizmoRenderPlugin>()
            .disable::<GltfPlugin>()
            .disable::<AudioPlugin>()
            .disable::<GilrsPlugin>()
            .disable::<LogPlugin>()
            .disable::<LightPlugin>()
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            }),
    );
    app.add_plugins(UiComponentPrototypesPlugin);
    let camera = app
        .world_mut()
        .spawn((
            Camera2d,
            RenderTarget::None {
                size: UVec2::new(800, 600),
            },
        ))
        .id();
    let column = app
        .world_mut()
        .spawn((
            Node {
                width: px(400),
                height: px(300),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            UiTargetCamera(camera),
        ))
        .id();
    (app, column)
}

fn size(app: &App, entity: Entity) -> Vec2 {
    app.world().get::<ComputedNode>(entity).unwrap().size()
}

// The default theme: a font of 16 with a line of 1.2 times that, a padding of 6 above and below, and a
// border of 1.
const FRAME_HEIGHT: f32 = 19.2 + 2.0 * 6.0 + 2.0 * 1.0;

#[test]
fn a_field_is_as_wide_as_its_parent_and_one_line_high() {
    let (mut app, column) = layout_app();
    let field = app
        .world_mut()
        .spawn((
            text_field().placeholder(Text::new("Name")).build(),
            ChildOf(column),
        ))
        .id();

    for _ in 0..6 {
        app.update();
    }

    let size = size(&app, field);
    assert_eq!(size.x, 400.0);
    assert!((size.y - FRAME_HEIGHT).abs() <= 1.0, "{size:?}");
}

#[test]
fn a_field_with_a_visible_width_is_as_wide_as_its_glyphs() {
    let (mut app, column) = layout_app();
    let field = app
        .world_mut()
        .spawn((text_field().visible_width(10.0).build(), ChildOf(column)))
        .id();

    for _ in 0..6 {
        app.update();
    }

    // Ten glyphs of the default font, which is monospaced, with the padding and the border.
    let size = size(&app, field);
    assert!(size.x > 100.0 && size.x < 130.0, "{size:?}");
    assert!((size.y - FRAME_HEIGHT).abs() <= 1.0, "{size:?}");
}

#[test]
fn a_visible_width_field_can_align_with_text_in_a_row() {
    let (mut app, column) = layout_app();
    let row = app
        .world_mut()
        .spawn((
            Node {
                width: px(400),
                height: px(100),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(column),
        ))
        .id();
    let label = app
        .world_mut()
        .spawn((Text::new("Label"), ChildOf(row)))
        .id();
    let field = app
        .world_mut()
        .spawn((
            text_field()
                .visible_width(10.0)
                .align_self(AlignSelf::Center)
                .build(),
            ChildOf(row),
        ))
        .id();

    for _ in 0..6 {
        app.update();
    }

    assert_eq!(
        app.world().get::<Node>(field).unwrap().align_self,
        AlignSelf::Center
    );
    let label_center = app
        .world()
        .get::<UiGlobalTransform>(label)
        .unwrap()
        .translation
        .y;
    let field_center = app
        .world()
        .get::<UiGlobalTransform>(field)
        .unwrap()
        .translation
        .y;
    assert!(
        (label_center - field_center).abs() <= 1.0,
        "label {label_center}, field {field_center}"
    );
}

#[test]
fn a_text_area_is_as_high_as_its_lines() {
    let (mut app, column) = layout_app();
    let field = app
        .world_mut()
        .spawn((text_field().lines(4.0).build(), ChildOf(column)))
        .id();

    for _ in 0..6 {
        app.update();
    }

    let size = size(&app, field);
    assert_eq!(size.x, 400.0);
    assert!(
        (size.y - (FRAME_HEIGHT + 3.0 * 19.2)).abs() <= 1.0,
        "{size:?}"
    );
}

#[test]
fn a_text_area_wraps_long_text_instead_of_growing() {
    let (mut app, column) = layout_app();
    let long = "word ".repeat(200);
    let field = app
        .world_mut()
        .spawn((
            text_field().lines(3.0).initial(long).build(),
            ChildOf(column),
        ))
        .id();

    for _ in 0..6 {
        app.update();
    }

    let size = size(&app, field);
    assert_eq!(size.x, 400.0);
    assert!(
        (size.y - (FRAME_HEIGHT + 2.0 * 19.2)).abs() <= 1.0,
        "{size:?}"
    );
    // The height is that of the lines shown, so the text itself must be on more lines than that.
    let input = app.world().get::<Children>(field).unwrap()[0];
    let text = app.world().get::<TextLayoutInfo>(input).unwrap();
    assert!(text.size.y > 3.0 * 19.2, "{:?}", text.size);
}

fn indicator(app: &App, field: Entity) -> Entity {
    let children = app.world().get::<Children>(field).unwrap();
    children
        .iter()
        .find(|child| app.world().get::<ScrollIndicator>(*child).is_some())
        .expect("the text area has a scroll indicator")
}

fn thumb_height(app: &App, track: Entity) -> Val {
    let thumb = app.world().get::<Children>(track).unwrap()[0];
    app.world().get::<Node>(thumb).unwrap().height
}

#[test]
fn the_indicator_of_a_text_area_shows_only_when_the_text_does_not_fit() {
    let (mut app, column) = layout_app();
    let fits = app
        .world_mut()
        .spawn((
            text_field().lines(4.0).initial("one\ntwo").build(),
            ChildOf(column),
        ))
        .id();
    let overflows = app
        .world_mut()
        .spawn((
            text_field()
                .lines(4.0)
                .initial("1\n2\n3\n4\n5\n6\n7\n8\n")
                .build(),
            ChildOf(column),
        ))
        .id();

    for _ in 0..8 {
        app.update();
    }

    let display = |field| {
        let track = indicator(&app, field);
        app.world().get::<Node>(track).unwrap().display
    };
    assert_eq!(display(fits), Display::None);
    assert_eq!(display(overflows), Display::Flex);
    // 4 of 9 lines are shown: the thumb is shorter than the track, which is shorter than the field.
    let Val::Px(thumb) = thumb_height(&app, indicator(&app, overflows)) else {
        panic!("the thumb has a height in pixels");
    };
    let field_height = size(&app, overflows).y;
    assert!(
        thumb > 0.0 && thumb < field_height * 0.6,
        "{thumb} of {field_height}"
    );
}

#[test]
fn a_single_line_field_has_no_indicator() {
    let (mut app, column) = layout_app();
    let field = app
        .world_mut()
        .spawn((text_field().build(), ChildOf(column)))
        .id();

    app.update();

    let children = app.world().get::<Children>(field).unwrap();
    assert!(
        children
            .iter()
            .all(|child| app.world().get::<ScrollIndicator>(child).is_none())
    );
}
