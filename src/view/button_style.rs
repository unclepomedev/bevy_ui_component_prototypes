use crate::{
    FocusRing, Theme,
    style::{
        theme::{ButtonStyle, VisualState},
        theme_lookup::ThemeLookup,
    },
};
use bevy::{
    ecs::system::SystemParam,
    input_focus::{InputFocus, InputFocusVisible},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{InteractionDisabled, UiSystems},
    ui_widgets::Button,
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

impl ButtonVariant {
    /// Returns the style this variant selects from `theme`.
    pub(crate) fn resolve<'a>(&'a self, theme: &'a Theme) -> &'a ButtonStyle {
        match self {
            Self::Neutral => &theme.buttons.neutral,
            Self::Primary => &theme.buttons.primary,
            Self::Affirmative => &theme.buttons.affirmative,
            Self::Danger => &theme.buttons.danger,
            Self::Custom(style) => style,
        }
    }
}

#[derive(Component, Clone, Default)]
pub(crate) struct ButtonLabel;
#[derive(Component, Clone, Default)]
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
        With<Button>,
        Or<(
            Changed<PickingInteraction>,
            Changed<ButtonVariant>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

fn sync_button_style(
    mut lookup: ThemeLookup,
    mut ring: FocusRingTracker,
    mut removed_disabled: RemovedComponents<InteractionDisabled>,
    changed: ChangedButtons,
    all: Query<Entity, (With<Button>, With<ButtonVariant>)>,
    mut buttons: ButtonQuery,
    mut content: ButtonContent,
) {
    let (focused, focus_moved) = ring.update();
    let removed: Vec<Entity> = removed_disabled.read().collect();
    let targets: Vec<Entity> = if lookup.needs_resync() {
        all.iter().collect()
    } else {
        changed.iter().chain(removed).chain(focus_moved).collect()
    };

    for entity in targets {
        let Ok((interaction, variant, disabled, mut bg, mut border, mut outline)) =
            buttons.get_mut(entity)
        else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let style = variant.resolve(theme);
        let is_focused = focused == Some(entity);
        let state = VisualState::resolve(interaction, is_focused, disabled);

        bg.set_if_neq(BackgroundColor(style.bg.pick(state)));
        border.set_if_neq(BorderColor::all(style.border.pick(state)));
        content.apply(entity, style.fg.pick(state), &theme.font);

        // The ring marks keyboard focus regardless of hover or press, but never on a disabled button.
        let focus_ring = if is_focused && !disabled {
            style.focus_ring
        } else {
            FocusRing::NONE
        };
        outline.set_if_neq(Outline::new(
            focus_ring.width,
            focus_ring.offset,
            focus_ring.color,
        ));
    }
}

/// Follows which button should show its focus ring.
#[derive(SystemParam)]
struct FocusRingTracker<'w, 's> {
    // Optional so that apps (and headless tests) without `InputFocusPlugin` simply have no focused button.
    focus: Option<Res<'w, InputFocus>>,
    visible: Option<Res<'w, InputFocusVisible>>,
    last: Local<'s, Option<Entity>>,
}

impl FocusRingTracker<'_, '_> {
    /// Returns the entity whose focus indicator is visible right now, and the entities
    /// whose ring state has changed since the previous call (the old and the new owner).
    fn update(&mut self) -> (Option<Entity>, Vec<Entity>) {
        let focused = self
            .focus
            .as_deref()
            .and_then(InputFocus::get)
            .filter(|_| self.visible.as_deref().is_some_and(|visible| visible.0));
        let previous = std::mem::replace(&mut *self.last, focused);
        let moved = if previous == focused {
            Vec::new()
        } else {
            previous.into_iter().chain(focused).collect()
        };
        (focused, moved)
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
            if let Ok(mut icon) = self.icons.get_mut(descendant)
                && icon.color != fg
            {
                icon.color = fg;
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
        &'static mut Outline,
    ),
    With<Button>,
>;
