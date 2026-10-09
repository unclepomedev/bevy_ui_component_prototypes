use super::{ButtonSize, IconButton, icon_button_node};
use crate::{
    Theme,
    behavior::{ButtonColors, pass_through},
};
use bevy::prelude::*;

pub trait IconButtonExt {
    fn spawn_icon_button(
        &mut self,
        theme: &Theme,
        label: impl Into<String>,
        icon: Option<Handle<Image>>,
        size: ButtonSize,
        colors: ButtonColors,
        marker: impl Bundle,
    ) -> EntityCommands<'_>;
}

impl IconButtonExt for ChildSpawnerCommands<'_> {
    fn spawn_icon_button(
        &mut self,
        theme: &Theme,
        label: impl Into<String>,
        icon: Option<Handle<Image>>,
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
            BorderColor::all(theme.border),
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
                    font: theme.font.clone(),
                    font_size: size.font_size,
                    ..default()
                },
                TextColor(theme.text_main),
                pass_through(),
            ));
        });
        ec
    }
}
