use super::{ButtonSize, IconButton, icon_button_node};
use crate::view::{
    button::{ButtonIcon, ButtonLabel, ButtonVariant},
    picking::pass_through,
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

    pub fn build(self) -> impl Bundle {
        let Self {
            label,
            icon,
            size,
            variant,
        } = self;

        (
            IconButton,
            icon_button_node(&size),
            variant,
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                if let Some(icon) = icon {
                    parent.spawn((
                        ButtonIcon,
                        ImageNode::new(icon),
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
                        font_size: size.font_size,
                        ..default()
                    },
                    pass_through(),
                ));
            })),
        )
    }
}
