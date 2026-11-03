//! A select in a window: its popup is above the window.

use super::*;

fn spawn_window_with_select(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(
            window()
                .content_bundle(Children::spawn(Spawn(
                    select::<u32>()
                        .options([(1, Text::new("One")), (2, Text::new("Two"))])
                        .build(),
                )))
                .build(),
        )
        .id()
}

/// The popup of the select inside `window`.
fn popup_in(app: &mut App, window: Entity) -> Entity {
    let popups: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Popover>>()
        .iter(app.world())
        .collect();
    popups
        .into_iter()
        .find(|popup| {
            app.world().get::<ChildOf>(*popup).is_some()
                && std::iter::successors(Some(*popup), |e| {
                    app.world().get::<ChildOf>(*e).map(|c| c.parent())
                })
                .any(|e| e == window)
        })
        .expect("the window has a popup")
}

#[test]
fn the_popup_of_a_select_in_a_window_is_one_layer_above_the_window_and_escapes_its_clip() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window_with_select(&mut app));
    app.update();
    app.update();

    for window in [a, b] {
        let popup = popup_in(&mut app, window);
        assert_eq!(layer(&app, popup), layer(&app, window).map(|z| z + 1));
        assert!(app.world().get::<OverrideClip>(popup).is_some());
    }
}

#[test]
fn the_popup_follows_its_window_when_the_window_is_raised() {
    let mut app = app_with_defaults(WindowDefaultsPlugin::default());
    let [a, b] = [(); 2].map(|_| spawn_window_with_select(&mut app));
    app.update();

    app.world_mut().trigger(RaiseWindow { window: a });
    app.update();
    app.update();

    // `a` is on the top now, and `b` is under it: so is the popup of `b`.
    let popup_a = popup_in(&mut app, a);
    let popup_b = popup_in(&mut app, b);
    assert_eq!(layer(&app, popup_a), Some(21));
    assert_eq!(layer(&app, popup_b), Some(11));
}
