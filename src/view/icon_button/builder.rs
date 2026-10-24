use super::{ButtonSize, IconButton};
use crate::{
    style::button::ButtonMetrics,
    view::{
        button::{ButtonIcon, ButtonLabel, ButtonVariant},
        picking::pass_through,
    },
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

    pub fn size(mut self, size: impl Into<ButtonSize>) -> Self {
        self.size = size.into();
        self
    }

    pub fn variant(mut self, variant: impl Into<ButtonVariant>) -> Self {
        self.variant = variant.into();
        self
    }

    pub fn build(self) -> impl Bundle {
        let Self {
            label,
            icon,
            size,
            variant,
        } = self;
        let metrics = ButtonMetrics::default();

        (
            IconButton,
            size,
            variant,
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                if let Some(icon) = icon {
                    parent.spawn((
                        ButtonIcon,
                        ImageNode::new(icon),
                        Node {
                            width: Val::Px(metrics.icon_size),
                            height: Val::Px(metrics.icon_size),
                            ..default()
                        },
                        pass_through(),
                    ));
                }
                parent.spawn((
                    ButtonLabel,
                    Text::new(label),
                    TextFont {
                        font_size: metrics.font_size,
                        ..default()
                    },
                    TextLayout::new(Justify::Left, LineBreak::NoWrap),
                    pass_through(),
                ));
            })),
        )
    }
}
