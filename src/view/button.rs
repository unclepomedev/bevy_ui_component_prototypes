mod content;
mod size;
mod surface;
mod variant;

pub(crate) use self::content::{ButtonIcon, ButtonLabel};
pub use self::{size::ButtonSize, variant::ButtonVariant};

use self::{content::ButtonContent, surface::ButtonSurface};
use super::{
    focus::FocusRingTracker,
    icon_button::icon_button_node,
    style_targets::{drain_removed, pick_targets},
    theme_lookup::ThemeLookup,
};
use crate::style::{button::ButtonLook, state::VisualState};
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

        write_node(&look, &mut button);
        if let (Some(metrics), Some(node)) = (metrics, button.node.as_mut()) {
            write_layout(node, &icon_button_node(metrics, &style.shape));
        }
        parts
            .surface
            .apply(entity, style.surface.as_ref(), look.tint);
        parts.content.apply(entity, look.fg, metrics, theme);
    }
}

/// Writes the size and the shape, only where they changed.
fn write_layout(node: &mut Mut<Node>, want: &Node) {
    if node.height != want.height {
        node.height = want.height;
    }
    if node.padding != want.padding {
        node.padding = want.padding;
    }
    if node.column_gap != want.column_gap {
        node.column_gap = want.column_gap;
    }
    if node.border != want.border {
        node.border = want.border;
    }
    if node.border_radius != want.border_radius {
        node.border_radius = want.border_radius;
    }
}

/// Writes the node-level colors and the focus ring, only where they changed.
fn write_node(look: &ButtonLook, button: &mut ButtonRowItem) {
    button.bg.set_if_neq(BackgroundColor(look.background));
    button.border.set_if_neq(BorderColor::all(look.border));
    button.outline.set_if_neq(Outline::new(
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

/// What the style sync writes besides the button node itself.
#[derive(SystemParam)]
struct ButtonParts<'w, 's> {
    content: ButtonContent<'w, 's>,
    surface: ButtonSurface<'w, 's>,
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
