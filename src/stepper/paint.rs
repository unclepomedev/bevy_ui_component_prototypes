use super::{
    components::{StepperButton, StepperCell, StepperLabel, StepperMarks, StepperPart},
    logic::{CellProgress, StepperFill},
    nodes::{arrow_glyph, button_node, cell_node, header_node, icon_node, mark_node, track_node},
    style::StepperStyle,
};
use crate::{common::state::VisualState, theme::Theme};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    picking::hover::PickingInteraction,
    prelude::*,
    ui::InteractionDisabled,
};

/// What every part of one stepper is painted from.
pub(super) struct Look<'a> {
    pub(super) theme: &'a Theme,
    /// The displayed position, already inside the range.
    pub(super) current: usize,
    pub(super) fill: Option<StepperFill>,
    pub(super) marks: &'a StepperMarks,
    /// The state of the stepper as a whole. A cell or a button has its own on top of it.
    pub(super) state: VisualState,
}

impl Look<'_> {
    fn style(&self) -> &StepperStyle {
        &self.theme.stepper
    }

    /// The state of a cell or a button that is `pressed` or `hovered` by itself.
    fn part_state(&self, interaction: &PickingInteraction, disabled: bool) -> VisualState {
        let disabled = disabled || self.state == VisualState::Disabled;
        VisualState::resolve(interaction, false, disabled)
    }
}

type CellFilter = (With<StepperCell>, Without<super::Stepper>);
type ButtonFilter = (
    With<StepperButton>,
    Without<super::Stepper>,
    Without<StepperCell>,
);
type PartFilter = (
    With<StepperPart>,
    Without<super::Stepper>,
    Without<StepperCell>,
    Without<StepperButton>,
);

/// Writes the colors and the layout of the parts under a stepper.
#[derive(SystemParam)]
pub(super) struct StepperPaint<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    cells: Query<'w, 's, CellRow, CellFilter>,
    buttons: Query<'w, 's, ButtonRow, ButtonFilter>,
    parts: Query<'w, 's, PartRow, PartFilter>,
    labels: Query<'w, 's, (&'static mut TextColor, &'static mut TextFont), With<StepperLabel>>,
}

impl StepperPaint<'_, '_> {
    pub(super) fn apply(&mut self, root: Entity, look: &Look) {
        let shape = &look.style().shape;
        for entity in self.children.iter_descendants(root) {
            if let Ok(mut cell) = self.cells.get_mut(entity) {
                write_cell(&mut cell, &mut self.parts, look);
            } else if let Ok(mut button) = self.buttons.get_mut(entity) {
                write_button(&mut button, &mut self.parts, look);
            } else if let Ok((mut color, mut font)) = self.labels.get_mut(entity) {
                color.set_if_neq(TextColor(look.style().label(look.state)));
                if font.font != look.theme.font {
                    font.font = look.theme.font.clone();
                }
                if font.font_smoothing != look.theme.font_smoothing {
                    font.font_smoothing = look.theme.font_smoothing;
                }
            } else if let Ok(mut part) = self.parts.get_mut(entity) {
                // The marks and the arrows are written by the cell and the button they belong to.
                match *part.kind {
                    StepperPart::Header => {
                        part.node.set_if_neq(header_node(shape));
                    }
                    StepperPart::Track => {
                        part.node.set_if_neq(track_node(shape));
                    }
                    StepperPart::Mark | StepperPart::Arrow => {}
                }
            }
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CellRow {
    cell: &'static StepperCell,
    interaction: &'static PickingInteraction,
    disabled: Has<InteractionDisabled>,
    bg: &'static mut BackgroundColor,
    node: &'static mut Node,
    children: Option<&'static Children>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ButtonRow {
    kind: &'static StepperButton,
    interaction: &'static PickingInteraction,
    disabled: Has<InteractionDisabled>,
    bg: &'static mut BackgroundColor,
    node: &'static mut Node,
    children: Option<&'static Children>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct PartRow {
    kind: &'static StepperPart,
    bg: &'static mut BackgroundColor,
    node: &'static mut Node,
    transform: &'static mut UiTransform,
    border: &'static mut BorderColor,
    // Only an icon in place of the drawn arrow has one.
    image: Option<&'static mut ImageNode>,
}

fn write_cell(cell: &mut CellRowItem, parts: &mut Query<PartRow, PartFilter>, look: &Look) {
    let style = look.style();
    let index = cell.cell.index();
    let state = look.part_state(cell.interaction, cell.disabled);
    let progress = CellProgress::of(index, look.current);
    cell.bg
        .set_if_neq(BackgroundColor(style.cell(progress, look.fill, state)));
    cell.node.set_if_neq(cell_node(&style.shape));

    let mark = if look.marks.contains(index) {
        style.mark(look.state)
    } else {
        Color::NONE
    };
    for child in cell.children.into_iter().flat_map(|c| c.iter()) {
        if let Ok(mut part) = parts.get_mut(child) {
            part.bg.set_if_neq(BackgroundColor(mark));
            part.node.set_if_neq(mark_node(&style.shape));
        }
    }
}

fn write_button(button: &mut ButtonRowItem, parts: &mut Query<PartRow, PartFilter>, look: &Look) {
    let shape = &look.style().shape;
    let state = look.part_state(button.interaction, button.disabled);
    let colors = look.style().arrow(state);
    button.bg.set_if_neq(BackgroundColor(colors.background));
    button.node.set_if_neq(button_node(shape));

    for child in button.children.into_iter().flat_map(|c| c.iter()) {
        let Ok(mut part) = parts.get_mut(child) else {
            continue;
        };
        if let Some(image) = part.image.as_mut() {
            if image.color != colors.arrow {
                image.color = colors.arrow;
            }
            part.node.set_if_neq(icon_node(shape));
        } else {
            let (node, transform) = arrow_glyph(shape, *button.kind);
            part.node.set_if_neq(node);
            part.transform.set_if_neq(transform);
            part.border.set_if_neq(BorderColor::all(colors.arrow));
        }
    }
}
