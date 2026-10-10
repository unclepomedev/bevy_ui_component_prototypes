use super::{ButtonSize, IconButton, icon_button_node};
use crate::{
    Theme,
    behavior::pass_through,
    view::button_style::{ButtonIcon, ButtonLabel, ButtonVariant},
};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};

pub struct IconButtonBuilder {
    label: String,
    icon: Option<Handle<Image>>,
    size: ButtonSize,
    variant: ButtonVariant,
}

pub fn icon_button(label: impl Into<String>) -> IconButtonBuilder {
    IconButtonBuilder {
        label: label.into(),
        icon: None,
        size: ButtonSize::default(),
        variant: ButtonVariant::default(),
    }
}

impl IconButtonBuilder {
    pub fn icon(mut self, icon: Handle<Image>) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn build(self, theme: &Theme) -> impl Bundle + use<> {
        let Self {
            label,
            icon,
            size,
            variant,
        } = self;
        let style = *theme.resolve_button(&variant);
        let font = theme.font.clone();

        (
            IconButton,
            icon_button_node(&size),
            variant,
            BackgroundColor(style.bg.normal),
            BorderColor::all(style.border.normal),
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                if let Some(icon) = icon {
                    parent.spawn((
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
                parent.spawn((
                    ButtonLabel,
                    Text::new(label),
                    TextFont {
                        font,
                        font_size: size.font_size,
                        ..default()
                    },
                    TextColor(style.fg.normal),
                    pass_through(),
                ));
            })),
        )
    }
}
