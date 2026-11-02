//! Tests of the tab order.

use super::spawn_slider;
use crate::support::{tab_index, test_app};
use bevy::ui::InteractionDisabled;

#[test]
fn disabled_slider_leaves_the_tab_order_and_returns_when_enabled() {
    let mut app = test_app();
    let slider = spawn_slider(&mut app);
    app.update();

    app.world_mut()
        .entity_mut(slider)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(tab_index(&app, slider), -1);

    app.world_mut()
        .entity_mut(slider)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(tab_index(&app, slider), 0);
}
