use crate::{
    behavior::Pressable,
    style::{theme::ButtonStyle, theme_lookup::ThemeLookup},
};
use bevy::{
    ecs::system::SystemParam,
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{InteractionDisabled, UiSystems},
};

#[derive(Component, Clone, Debug, Default)]
pub enum ButtonVariant {
    #[default]
    Neutral,
    Primary,
    Affirmative,
    Danger,
    Custom(Box<ButtonStyle>),
}

#[derive(Component)]
pub(crate) struct ButtonLabel;
#[derive(Component)]
pub(crate) struct ButtonIcon;

pub(crate) struct ButtonStylePlugin;
impl Plugin for ButtonStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_button_style.before(UiSystems::Prepare));
    }
}

type ChangedButtons<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<Pressable>,
        Or<(
            Changed<PickingInteraction>,
            Changed<ButtonVariant>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

fn sync_button_style(
    lookup: ThemeLookup,
    mut removed_disabled: RemovedComponents<InteractionDisabled>,
    changed: ChangedButtons,
    all: Query<Entity, (With<Pressable>, With<ButtonVariant>)>,
    mut buttons: ButtonQuery,
    mut content: ButtonContent,
) {
    let removed: Vec<Entity> = removed_disabled.read().collect();
    let targets: Vec<Entity> = if lookup.needs_resync() {
        all.iter().collect()
    } else {
        changed.iter().chain(removed).collect()
    };

    for entity in targets {
        let Ok((interaction, variant, disabled, mut bg, mut border)) = buttons.get_mut(entity)
        else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let style = theme.resolve_button(variant);

        bg.set_if_neq(BackgroundColor(style.bg.pick(interaction, disabled)));
        border.set_if_neq(BorderColor::all(style.border.pick(interaction, disabled)));
        content.apply(entity, style.fg.pick(interaction, disabled), &theme.font);
    }
}

#[derive(SystemParam)]
struct ButtonContent<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    labels: Query<'w, 's, (&'static mut TextColor, &'static mut TextFont), With<ButtonLabel>>,
    icons: Query<'w, 's, &'static mut ImageNode, With<ButtonIcon>>,
}

impl ButtonContent<'_, '_> {
    /// Applies the foreground color and the theme font to the button's descendants.
    fn apply(&mut self, root: Entity, fg: Color, font: &FontSource) {
        for descendant in self.children.iter_descendants(root) {
            if let Ok((mut color, mut text_font)) = self.labels.get_mut(descendant) {
                color.set_if_neq(TextColor(fg));
                if text_font.font != *font {
                    text_font.font = font.clone();
                }
            }
            if let Ok(mut icon) = self.icons.get_mut(descendant) {
                if icon.color != fg {
                    icon.color = fg;
                }
            }
        }
    }
}

type ButtonQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static PickingInteraction,
        &'static ButtonVariant,
        Has<InteractionDisabled>,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    With<Pressable>,
>;
