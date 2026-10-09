use super::{
    components::{ButtonColors, ButtonDisabled, Pressable, SelfStyled},
    events::Pressed,
};
use crate::Theme;
use bevy::{picking::hover::PickingInteraction, prelude::*};
// TODO remove button-related components

pub fn sync_pressable_visuals(
    theme: Res<Theme>,
    mut removed_disabled: RemovedComponents<ButtonDisabled>,
    changed: ChangedPressables,
    mut pressables: PressableVisuals,
    mut texts: Query<&mut TextColor>,
    mut icons: Query<&mut ImageNode>,
) {
    for entity in changed.iter().chain(removed_disabled.read()) {
        let Ok((interaction, colors, disabled, mut bg, children)) = pressables.get_mut(entity)
        else {
            continue;
        };

        let (bg_color, content_color) = if disabled {
            (theme.disabled_bg, theme.text_muted)
        } else {
            let bg = match interaction {
                PickingInteraction::Pressed => colors.pressed,
                PickingInteraction::Hovered => colors.hovered,
                PickingInteraction::None => colors.normal,
            };
            (bg, theme.text_main)
        };

        *bg = BackgroundColor(bg_color);
        for child in children.iter() {
            if let Ok(mut tc) = texts.get_mut(child) {
                tc.0 = content_color;
            }
            if let Ok(mut icon) = icons.get_mut(child) {
                icon.color = content_color;
            }
        }
    }
}

pub fn on_pressable_click(
    trigger: On<PointerClick>,
    pressables: Query<(), (With<Pressable>, Without<ButtonDisabled>)>,
    mut commands: Commands,
) {
    if pressables.get(trigger.entity).is_ok() {
        commands.trigger(Pressed {
            entity: trigger.entity,
        });
    }
}

type ChangedPressables<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        Or<(
            Changed<PickingInteraction>,
            Changed<ButtonColors>,
            Added<ButtonDisabled>,
        )>,
        With<Pressable>,
        Without<SelfStyled>,
    ),
>;

type PressableVisuals<'w, 's> = Query<
    'w,
    's,
    (
        &'static PickingInteraction,
        &'static ButtonColors,
        Has<ButtonDisabled>,
        &'static mut BackgroundColor,
        &'static Children,
    ),
    (With<Pressable>, Without<SelfStyled>),
>;
