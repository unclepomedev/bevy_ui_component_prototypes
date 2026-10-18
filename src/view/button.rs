mod content;
mod focus;
mod surface;
mod variant;

pub(crate) use self::content::{ButtonIcon, ButtonLabel};
pub use self::variant::ButtonVariant;

use self::{content::ButtonContent, focus::FocusRingTracker, surface::ButtonSurface};
use super::theme_lookup::ThemeLookup;
use crate::style::{button::ButtonLook, state::VisualState};
use bevy::{
    ecs::system::SystemParam,
    picking::hover::PickingInteraction,
    prelude::*,
    ui::{InteractionDisabled, UiSystems},
    ui_widgets::Button,
};

pub(crate) struct ButtonStylePlugin;
impl Plugin for ButtonStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_button_style.before(UiSystems::Prepare));
    }
}

/// Resolves each button's look from its theme and state and writes it to the button and its content.
fn sync_button_style(
    mut lookup: ThemeLookup,
    mut ring: FocusRingTracker,
    mut targets: StyleTargets,
    mut buttons: ButtonQuery,
    mut parts: ButtonParts,
) {
    let (focused, focus_moved) = ring.update();
    let targets = targets.collect(lookup.needs_resync(), focus_moved);

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
        let look = style.look(state, is_focused);

        write_node(&look, &mut bg, &mut border, &mut outline);
        parts
            .surface
            .apply(entity, style.surface.as_ref(), look.tint);
        parts.content.apply(entity, look.fg, theme);
    }
}

/// Writes the node-level colors and the focus ring, only where they changed.
fn write_node(
    look: &ButtonLook,
    bg: &mut Mut<BackgroundColor>,
    border: &mut Mut<BorderColor>,
    outline: &mut Mut<Outline>,
) {
    bg.set_if_neq(BackgroundColor(look.background));
    border.set_if_neq(BorderColor::all(look.border));
    outline.set_if_neq(Outline::new(
        look.ring.width,
        look.ring.offset,
        look.ring.color,
    ));
}

/// Finds the buttons whose look may have changed from the last run.
#[derive(SystemParam)]
struct StyleTargets<'w, 's> {
    changed: ChangedButtons<'w, 's>,
    all: Query<'w, 's, Entity, (With<Button>, With<ButtonVariant>)>,
    removed_disabled: RemovedComponents<'w, 's, InteractionDisabled>,
}

impl StyleTargets<'_, '_> {
    /// Every button after a theme change, otherwise only the ones that changed
    /// (including both ends of a focus move).
    fn collect(&mut self, resync: bool, focus_moved: Vec<Entity>) -> Vec<Entity> {
        let removed = self.removed_disabled.read();
        if resync {
            self.all.iter().collect()
        } else {
            self.changed
                .iter()
                .chain(removed)
                .chain(focus_moved)
                .collect()
        }
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

/// What the style sync writes besides the button node itself.
#[derive(SystemParam)]
struct ButtonParts<'w, 's> {
    content: ButtonContent<'w, 's>,
    surface: ButtonSurface<'w, 's>,
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
