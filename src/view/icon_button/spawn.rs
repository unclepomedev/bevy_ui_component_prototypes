use super::{ButtonSize, IconButton, icon_button_node};
use crate::view::button_style::{ButtonIcon, ButtonLabel, ButtonVariant};
use crate::{Theme, behavior::pass_through};
use bevy::prelude::*;

pub trait IconButtonExt {
    fn spawn_icon_button(
        &mut self,
        theme: &Theme,
        label: impl Into<String>,
        icon: Option<Handle<Image>>,
        size: ButtonSize,
        variant: ButtonVariant,
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
        variant: ButtonVariant,
        marker: impl Bundle,
    ) -> EntityCommands<'_> {
        let label = label.into();
        let style = *theme.resolve_button(&variant);
        let mut ec = self.spawn((
            IconButton,
            icon_button_node(&size),
            variant,
            BackgroundColor(style.bg.normal),
            BorderColor::all(style.border.normal),
            marker,
        ));
        ec.with_children(|p| {
            if let Some(icon) = icon {
                p.spawn((
                    ButtonIcon,
                    ImageNode::new(icon).with_color(style.fg.normal),
                    Node {
                        width: Val::Px(size.icon_size),
                        height: Val::Px(size.icon_size),
                        ..default()
                    },
                    pass_through(),
                ));
            }
            p.spawn((
                ButtonLabel,
                Text::new(label),
                TextFont {
                    font: theme.font.clone(),
                    font_size: size.font_size,
                    ..default()
                },
                TextColor(style.fg.normal),
                pass_through(),
            ));
        });
        ec
    }
}
