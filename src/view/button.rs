mod content;
mod layout;
mod size;
mod surface;
mod variant;

pub use self::{size::ButtonSize, variant::ButtonVariant};

pub(crate) use self::{
    content::{ButtonContent, ButtonIcon, ButtonLabel},
    layout::{button_node, write_button_layout},
    surface::ButtonSurface,
};

use super::{
    focus::FocusRingTracker,
    style_targets::{drain_removed, pick_targets},
    theme_lookup::ThemeLookup,
};
use crate::{
    Theme,
    style::{
        button::{ButtonLook, ButtonMetrics, ButtonStyle},
        state::VisualState,
    },
};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
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
        let Ok(mut button) = buttons.get_mut(entity) else {
            continue;
        };

        let theme = lookup.for_entity(entity);
        let style = button.variant.resolve(theme);
        let metrics = button.size.map(|size| size.resolve(theme));
        let is_focused = focused == Some(entity);
        let state = VisualState::resolve(button.interaction, is_focused, button.disabled);
        let look = style.look(state, is_focused);

        write_button_colors(
            &look,
            &mut button.bg,
            &mut button.border,
            &mut button.outline,
        );
        if let (Some(metrics), Some(node)) = (metrics, button.node.as_mut()) {
            write_button_layout(node, &button_node(metrics, &style.shape));
        }
        parts.paint(entity, style, &look, metrics, theme);
    }
}

/// Writes the node-level colors and the focus ring, only where they changed.
pub(crate) fn write_button_colors(
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
        let removed = drain_removed(&mut self.removed_disabled);
        pick_targets(
            resync,
            self.all.iter(),
            self.changed.iter(),
            removed,
            focus_moved,
        )
    }
}

type ChangedButtons<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<Button>,
        Or<(
            Added<Button>,
            Changed<PickingInteraction>,
            Changed<ButtonVariant>,
            Changed<ButtonSize>,
            Added<InteractionDisabled>,
        )>,
    ),
>;

/// What the style sync writes besides the node itself: the image surface and the content (text and icon).
#[derive(SystemParam)]
pub(crate) struct ButtonParts<'w, 's> {
    content: ButtonContent<'w, 's>,
    surface: ButtonSurface<'w, 's>,
}

impl ButtonParts<'_, '_> {
    /// Draws the image surface of `style` and styles the content, for the `look` the button has now.
    pub(crate) fn paint(
        &mut self,
        button: Entity,
        style: &ButtonStyle,
        look: &ButtonLook,
        metrics: Option<&ButtonMetrics>,
        theme: &Theme,
    ) {
        self.surface
            .apply(button, style.surface.as_ref(), look.tint);
        self.content.apply(button, look.fg, metrics, theme);
    }
}

// Without the icon, which the content writes a `Node` of its own to.
type ButtonQuery<'w, 's> = Query<'w, 's, ButtonRow, (With<Button>, Without<ButtonIcon>)>;

/// What the sync reads from a button and writes to it.
#[derive(QueryData)]
#[query_data(mutable)]
struct ButtonRow {
    interaction: &'static PickingInteraction,
    variant: &'static ButtonVariant,
    size: Option<&'static ButtonSize>,
    disabled: Has<InteractionDisabled>,
    bg: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
    outline: &'static mut Outline,
    // A button without a `Node` of its own is styled all the same.
    node: Option<&'static mut Node>,
}
