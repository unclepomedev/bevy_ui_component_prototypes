//! Tests of the buttons of a popup and of what they report.

use super::{panel, spawn_popup};
use crate::{
    probe::probe_theme,
    support::{red, test_app},
};
use bevy::{prelude::*, ui_widgets::Activate};
use bevy_ui_component_prototypes::{
    ButtonVariant, ButtonVariantId, GlobalTheme, IconButton, PopupChoice, PopupChosen, popup,
};

#[derive(Resource, Default)]
struct Chosen(Vec<(Entity, PopupChoice)>);

fn app_with_log() -> App {
    let mut app = test_app();
    app.init_resource::<Chosen>();
    app.world_mut()
        .add_observer(|chosen: On<PopupChosen>, mut log: ResMut<Chosen>| {
            log.0.push((chosen.popup, chosen.choice));
        });
    app
}

pub(super) fn spawn_confirm_popup(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(
            popup()
                .message("Delete?")
                .button("Delete", PopupChoice::Confirm, ButtonVariantId::DANGER)
                .button("Keep", PopupChoice::Cancel, ButtonVariant::Default)
                .build(),
        )
        .id()
}

/// The message comes first in the panel, and the row of buttons after it.
pub(super) fn buttons(app: &App, root: Entity) -> Vec<Entity> {
    let panel = app.world().get::<Children>(panel(app, root)).unwrap();
    app.world().get::<Children>(panel[1]).unwrap().to_vec()
}

fn label(app: &App, button: Entity) -> String {
    let children = app.world().get::<Children>(button).unwrap();
    let text = children
        .iter()
        .find_map(|child| app.world().get::<Text>(child))
        .unwrap();
    text.0.clone()
}

#[test]
fn the_buttons_follow_the_message_in_the_order_they_were_added() {
    let mut app = test_app();
    let root = spawn_confirm_popup(&mut app);

    let buttons = buttons(&app, root);
    assert_eq!(buttons.len(), 2);
    assert_eq!(label(&app, buttons[0]), "Delete");
    assert_eq!(label(&app, buttons[1]), "Keep");
    assert!(
        buttons
            .iter()
            .all(|b| app.world().get::<IconButton>(*b).is_some())
    );
}

#[test]
fn a_button_takes_the_variant_it_was_given() {
    let mut app = test_app();
    let root = spawn_confirm_popup(&mut app);

    let buttons = buttons(&app, root);
    let variant = |b: Entity| *app.world().get::<ButtonVariant>(b).unwrap();
    assert_eq!(
        variant(buttons[0]),
        ButtonVariant::Named(ButtonVariantId::DANGER)
    );
    assert_eq!(variant(buttons[1]), ButtonVariant::Default);
}

#[test]
fn a_popup_without_buttons_holds_only_the_message() {
    let mut app = test_app();
    let root = spawn_popup(&mut app, "Saved");

    let panel = app.world().get::<Children>(panel(&app, root)).unwrap();
    assert_eq!(panel.len(), 1);
}

#[test]
fn activating_a_button_reports_its_choice_on_its_popup() {
    let mut app = app_with_log();
    let root = spawn_confirm_popup(&mut app);
    let buttons = buttons(&app, root);

    app.world_mut().trigger(Activate { entity: buttons[0] });
    app.world_mut().trigger(Activate { entity: buttons[1] });
    app.update();

    assert_eq!(
        app.world().resource::<Chosen>().0,
        [(root, PopupChoice::Confirm), (root, PopupChoice::Cancel)]
    );
}

#[test]
fn the_choice_goes_to_the_popup_of_the_button() {
    let mut app = app_with_log();
    let first = spawn_confirm_popup(&mut app);
    let second = spawn_confirm_popup(&mut app);

    let buttons = buttons(&app, second);
    app.world_mut().trigger(Activate { entity: buttons[1] });
    app.update();

    assert_eq!(
        app.world().resource::<Chosen>().0,
        [(second, PopupChoice::Cancel)]
    );
    assert!(app.world().get_entity(first).is_ok());
}

#[test]
fn the_popup_stays_after_a_choice() {
    let mut app = app_with_log();
    let root = spawn_confirm_popup(&mut app);
    let buttons = buttons(&app, root);

    app.world_mut().trigger(Activate { entity: buttons[0] });
    app.update();

    assert!(app.world().get_entity(root).is_ok());
}

#[test]
fn the_gap_and_the_width_come_from_the_theme_and_follow_it() {
    let mut app = test_app();
    let root = spawn_confirm_popup(&mut app);
    app.update();

    let metrics = probe_theme().popup_metrics;
    let buttons = buttons(&app, root);
    let row = app.world().get::<ChildOf>(buttons[0]).unwrap().parent();
    let node = |e: Entity| app.world().get::<Node>(e).unwrap().clone();
    assert_eq!(node(row).column_gap, metrics.button_gap);
    assert!(
        buttons
            .iter()
            .all(|b| node(*b).width == metrics.button_width)
    );

    let mut theme = probe_theme();
    theme.popup_metrics.button_gap = Val::Px(1.0);
    theme.popup_metrics.button_width = Val::Px(2.0);
    theme.popup.bg = red();
    app.insert_resource(GlobalTheme(theme));
    app.update();

    let node = |e: Entity| app.world().get::<Node>(e).unwrap().clone();
    assert_eq!(node(row).column_gap, Val::Px(1.0));
    assert!(buttons.iter().all(|b| node(*b).width == Val::Px(2.0)));
}
