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

type Extra = Box<dyn FnOnce(&mut EntityWorldMut) + Send + Sync>;

pub struct IconButtonBuilder {
    label: String,
    label_extras: Vec<Extra>,
    icon_extras: Vec<Extra>,
    icon: Option<Handle<Image>>,
    size: ButtonSize,
    variant: ButtonVariant,
}

pub fn icon_button(label: impl Into<String>) -> IconButtonBuilder {
    IconButtonBuilder {
        label: label.into(),
        label_extras: Vec::new(),
        icon_extras: Vec::new(),
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

    /// Adds `bundle` to the label entity. Can be called more than once.
    pub fn label_bundle(mut self, bundle: impl Bundle) -> Self {
        self.label_extras.push(Box::new(move |label| {
            label.insert(bundle);
        }));
        self
    }

    /// Adds `bundle` to the icon entity. Ignored without `icon`. Can be called more than once.
    pub fn icon_bundle(mut self, bundle: impl Bundle) -> Self {
        self.icon_extras.push(Box::new(move |icon| {
            icon.insert(bundle);
        }));
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
            label_extras,
            icon_extras,
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
                    let mut icon = parent.spawn((
                        ButtonIcon,
                        ImageNode::new(icon),
                        Node {
                            width: Val::Px(metrics.icon_size),
                            height: Val::Px(metrics.icon_size),
                            ..default()
                        },
                        pass_through(),
                    ));
                    for extra in icon_extras {
                        extra(&mut icon);
                    }
                }
                let mut label = parent.spawn((
                    ButtonLabel,
                    Text::new(label),
                    TextFont {
                        font_size: metrics.font_size,
                        ..default()
                    },
                    TextLayout::new(Justify::Left, LineBreak::NoWrap),
                    pass_through(),
                ));
                for extra in label_extras {
                    extra(&mut label);
                }
            })),
        )
    }
}
