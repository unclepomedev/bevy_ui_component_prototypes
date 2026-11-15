use super::{
    choose::choose_on_activate,
    components::{
        PopupButton, PopupButtons, PopupChoice, PopupMessage, PopupPanel, PopupRoot,
        popup_buttons_node,
    },
    style::PopupMetrics,
};
use crate::{
    button::{ButtonVariant, icon_button},
    common::picking::pass_through,
};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};

struct ButtonSpec {
    label: String,
    choice: PopupChoice,
    variant: Option<ButtonVariant>,
}

pub struct PopupBuilder {
    message: String,
    buttons: Vec<ButtonSpec>,
}

/// A popup with an empty message and no buttons. Without a button, the user cannot dismiss it.
pub fn popup() -> PopupBuilder {
    PopupBuilder {
        message: String::new(),
        buttons: Vec::new(),
    }
}

impl PopupBuilder {
    /// Designed as a short message that fits within the screen; does not scroll.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Adds a button after the ones added before it. Activating it makes the popup emit
    /// [`PopupChosen`](super::PopupChosen) with `choice`.
    ///
    /// Its look is the theme's: [`PopupStyle::button`](super::PopupStyle::button).
    pub fn button(mut self, label: impl Into<String>, choice: PopupChoice) -> Self {
        self.buttons.push(ButtonSpec {
            label: label.into(),
            choice,
            variant: None,
        });
        self
    }

    /// Like [`button`](Self::button), but with a variant of the app's choosing, such as a danger
    /// button for a destructive choice, instead of the theme's popup button.
    pub fn button_with(
        mut self,
        label: impl Into<String>,
        choice: PopupChoice,
        variant: impl Into<ButtonVariant>,
    ) -> Self {
        self.buttons.push(ButtonSpec {
            label: label.into(),
            choice,
            variant: Some(variant.into()),
        });
        self
    }

    pub fn build(self) -> impl Bundle {
        let Self { message, buttons } = self;
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
                    if !buttons.is_empty() {
                        spawn_buttons(panel, buttons);
                    }
                });
            })),
        )
    }
}

fn spawn_buttons(panel: &mut RelatedSpawner<ChildOf>, buttons: Vec<ButtonSpec>) {
    panel
        .spawn((
            PopupButtons,
            popup_buttons_node(&PopupMetrics::default()),
            pass_through(),
        ))
        .with_children(|row| {
            for ButtonSpec {
                label,
                choice,
                variant,
            } in buttons
            {
                // Until the first style sync, a button without a variant of its own has the base one.
                let button = icon_button()
                    .label(label)
                    .variant(variant.unwrap_or_default())
                    .build();
                row.spawn((button, PopupButton { choice, variant }))
                    .observe(choose_on_activate);
            }
        });
}
