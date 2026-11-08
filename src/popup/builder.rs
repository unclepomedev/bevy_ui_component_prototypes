use super::components::{PopupMessage, PopupPanel, PopupRoot};
use crate::common::picking::pass_through;
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};

pub struct PopupBuilder {
    message: String,
}

/// A popup with an empty message.
pub fn popup() -> PopupBuilder {
    PopupBuilder {
        message: String::new(),
    }
}

impl PopupBuilder {
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn build(self) -> impl Bundle {
        let Self { message } = self;
        (
            PopupRoot,
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                parent.spawn(PopupPanel).with_children(|panel| {
                    panel.spawn((
                        PopupMessage,
                        Text::new(message),
                        // Unseen until the first style sync.
                        TextColor(Color::NONE),
                        pass_through(),
                    ));
                });
            })),
        )
    }
}
