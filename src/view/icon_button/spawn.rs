use super::{ButtonSize, IconButton, icon_button_node};
use crate::{
    behavior::{ButtonColors, pass_through},
    style::color::{BORDER, TEXT_MAIN},
};
use bevy::prelude::*;

pub trait IconButtonExt {
    fn spawn_icon_button(
        &mut self,
        label: impl Into<String>,
        icon: Option<Handle<Image>>,
        font: FontSource,
        size: ButtonSize,
        colors: ButtonColors,
        marker: impl Bundle,
    ) -> EntityCommands<'_>;
}

impl IconButtonExt for ChildSpawnerCommands<'_> {
    fn spawn_icon_button(
        &mut self,
        label: impl Into<String>,
        icon: Option<Handle<Image>>,
        font: FontSource,
        size: ButtonSize,
        colors: ButtonColors,
        marker: impl Bundle,
    ) -> EntityCommands<'_> {
        let label = label.into();
        let mut ec = self.spawn((
            IconButton,
            icon_button_node(&size),
            colors,
            BackgroundColor(colors.normal),
            BorderColor::all(BORDER),
            marker,
        ));
        ec.with_children(|p| {
            if let Some(icon) = icon {
                p.spawn((
                    ImageNode::new(icon),
                    Node {
                        width: Val::Px(size.icon_size),
                        height: Val::Px(size.icon_size),
                        ..default()
                    },
                    pass_through(),
                ));
            }
            p.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: size.font_size,
                    ..default()
                },
                TextColor(TEXT_MAIN),
                pass_through(),
            ));
        });
        ec
    }
}
