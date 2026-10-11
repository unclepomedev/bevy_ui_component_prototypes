use super::{ButtonSize, icon_button_node};
use crate::view::button_style::ButtonLabel;
use crate::{ButtonVariant, IconButton};
use bevy::prelude::*;

pub fn icon_button_scene(
    label: impl Into<String>,
    size: ButtonSize,
    variant: ButtonVariant,
) -> impl Scene {
    let label: String = label.into();
    let node = icon_button_node(&size);
    let font_size = size.font_size;
    bsn! {
        IconButton
        node
        variant
        Children [
            ButtonLabel
            Text({label.clone()})
            TextFont { font_size: {font_size} }
            Pickable { should_block_lower: false, is_hoverable: false }
        ]
    }
}
