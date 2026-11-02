//! Tests of Styling.

use super::{label_color, shown, spawn_fruit, trigger};
use crate::{
    probe::probe_theme,
    support::{bg, outline, test_app},
};
use bevy::{
    input_focus::{InputFocus, InputFocusVisible},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::InteractionDisabled,
};
use bevy_ui_component_prototypes::SelectedIndex;

#[test]
fn the_trigger_and_its_label_are_styled_after_one_update() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    let style = probe_theme().select.trigger;
    let metrics = probe_theme().select_metrics.trigger;
    let trigger = trigger(&app, root);
    assert_eq!(bg(&app, trigger), style.bg.normal);
    assert_eq!(
        *app.world().get::<BorderColor>(trigger).unwrap(),
        BorderColor::all(style.border.normal)
    );
    let node = app.world().get::<Node>(trigger).unwrap();
    assert_eq!(node.height, metrics.height);
    assert_eq!(node.padding, metrics.padding);
    assert_eq!(node.border, UiRect::all(Val::Px(style.shape.border_width)));

    let label = shown(&app, root).unwrap();
    assert_eq!(label_color(&app, label), style.fg.normal);
    let font = app.world().get::<TextFont>(label).unwrap();
    assert_eq!(font.font, probe_theme().font);
    assert_eq!(font.font_size, metrics.font_size);
}

#[test]
fn the_chevron_takes_the_color_of_the_text() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);

    app.update();

    assert_eq!(
        *app.world().get::<BorderColor>(arrow(&app, root)).unwrap(),
        BorderColor::all(probe_theme().select.trigger.fg.normal)
    );
}

#[test]
fn a_label_that_replaces_another_is_styled_in_the_same_update() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(root)
        .insert(SelectedIndex { index: Some(0) });
    app.update();

    let label = shown(&app, root).unwrap();
    assert_eq!(
        label_color(&app, label),
        probe_theme().select.trigger.fg.normal
    );
}

#[test]
fn hover_and_press_change_the_colors_of_the_trigger_and_its_text() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let style = probe_theme().select.trigger;
    app.update();
    let trigger = trigger(&app, root);

    for (state, bg_color, fg_color) in [
        (
            PickingInteraction::Hovered,
            style.bg.hovered,
            style.fg.hovered,
        ),
        (
            PickingInteraction::Pressed,
            style.bg.pressed,
            style.fg.pressed,
        ),
        (PickingInteraction::None, style.bg.normal, style.fg.normal),
    ] {
        app.world_mut().entity_mut(trigger).insert(state);
        app.update();
        assert_eq!(bg(&app, trigger), bg_color);
        let label = shown(&app, root).unwrap();
        assert_eq!(label_color(&app, label), fg_color);
    }
}

#[test]
fn interaction_disabled_grays_the_trigger_and_removal_restores_it() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    let style = probe_theme().select.trigger;
    app.update();
    let trigger = trigger(&app, root);

    app.world_mut()
        .entity_mut(trigger)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(bg(&app, trigger), style.bg.disabled);
    assert_eq!(
        label_color(&app, shown(&app, root).unwrap()),
        style.fg.disabled
    );

    app.world_mut()
        .entity_mut(trigger)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(bg(&app, trigger), style.bg.normal);
}

#[test]
fn visible_focus_draws_the_ring_on_the_trigger() {
    let mut app = test_app();
    let root = spawn_fruit(&mut app);
    app.update();
    let trigger = trigger(&app, root);

    app.insert_resource(InputFocus::from_entity(trigger));
    app.insert_resource(InputFocusVisible(true));
    app.update();

    let style = probe_theme().select.trigger;
    assert_eq!(bg(&app, trigger), style.bg.focused);
    assert_eq!(outline(&app, trigger).color, style.focus_ring.color);
}

fn arrow(app: &App, root: Entity) -> Entity {
    app.world().get::<Children>(trigger(app, root)).unwrap()[1]
}
