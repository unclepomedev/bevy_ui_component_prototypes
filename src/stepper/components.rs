use super::{
    logic::{Move, StepperFill, clamp_index, target},
    nodes::stepper_node,
    style::StepperShape,
};
use crate::common::{focus::hidden_outline, tab_order::KeyboardNavigable};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    input_focus::tab_navigation::TabIndex,
    picking::hover::PickingInteraction,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::Button,
};
use std::iter::once as iter_once;

/// The number of positions a stepper is meant for. More is allowed, but the cells get narrow.
pub const RECOMMENDED_MAX_STEPS: usize = 20;

/// A themed stepper: one of a fixed number of ordered positions, shown as a row of cells
/// between a previous and a next button.
///
/// The stepper is controlled: it never changes its own [`StepperValue`]. It emits
/// [`bevy::ui_widgets::ValueChange<usize>`] carrying the requested index, and the app applies it by
/// inserting a new value (or adds [`super::stepper_self_update`] to let the stepper do exactly that).
/// A request equal to the current value is not emitted.
///
/// A press on a cell requests its position, and the buttons request one position away. On the
/// focused stepper, Left and Right do the same, and Home and End request the ends. The focused
/// stepper consumes Left and Right, so directional navigation does not move the focus; Up and
/// Down pass. While it has `InteractionDisabled`, nothing is requested.
///
/// The number of positions is fixed when it is built; to change it, build a new stepper. It is meant
/// for up to [`RECOMMENDED_MAX_STEPS`] positions.
///
/// The width follows the parent layout (`100%`).
///
/// An inline stepper has the buttons on either side of the cells in one row and no label,
/// where the others have the buttons above them.
///
/// It has no parts of its own: build it with [`stepper`](super::stepper) rather than adding this
/// component alone.
///
/// To reach it with the Tab key, put it under a `TabGroup`. It is the only stop: its buttons and
/// cells have no `TabIndex`. The focus ring is drawn with the entity's own `Outline`, so do not set
/// an `Outline` on it yourself.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[require(
    StepperValue,
    StepperSteps,
    StepperMarks,
    TabIndex(0),
    KeyboardNavigable,
    Outline = hidden_outline(),
    Node = stepper_node(&StepperShape::default(), false)
)]
pub struct Stepper {
    wrap: bool,
    fill: Option<StepperFill>,
    inline: bool,
}

impl Stepper {
    pub(crate) fn new(wrap: bool, fill: Option<StepperFill>, inline: bool) -> Self {
        Self { wrap, fill, inline }
    }

    /// Whether the buttons are on either side of the cells, in one row.
    pub fn is_inline(&self) -> bool {
        self.inline
    }

    /// Whether stepping past an end continues from the other end.
    pub fn wraps(&self) -> bool {
        self.wrap
    }

    /// Changes [`Self::wraps`]. The buttons follow at the next update.
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    /// The fill this stepper chose; `None` follows the theme.
    pub fn fill(&self) -> Option<StepperFill> {
        self.fill
    }

    /// Changes [`Self::fill`]. The cells follow at the next update.
    pub fn set_fill(&mut self, fill: Option<StepperFill>) {
        self.fill = fill;
    }
}

/// The current position of a stepper, as a zero-based index. An index past the last position is
/// shown as the last.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepperValue {
    index: usize,
}

impl StepperValue {
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    /// The index as stored, even when it is past the last position.
    pub fn get(self) -> usize {
        self.index
    }
}

/// The number of positions of a stepper. At least 1, and fixed once the stepper is built.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperSteps {
    count: usize,
}

impl StepperSteps {
    pub(crate) fn new(count: usize) -> Self {
        Self { count }
    }

    pub fn get(self) -> usize {
        self.count
    }
}

impl Default for StepperSteps {
    fn default() -> Self {
        Self::new(1)
    }
}

/// The cells of a stepper that carry a mark.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct StepperMarks {
    indices: Vec<usize>,
}

impl StepperMarks {
    /// The marks under the cells at `indices`. An index past the last position shows nothing.
    pub fn new(indices: impl IntoIterator<Item = usize>) -> Self {
        Self {
            indices: indices.into_iter().collect(),
        }
    }

    /// Whether the cell at `index` carries a mark.
    pub fn contains(&self, index: usize) -> bool {
        self.indices.contains(&index)
    }
}

/// Where a stepper stands, and what a move would do from there.
#[derive(QueryData)]
pub(crate) struct StepperView {
    stepper: &'static Stepper,
    value: &'static StepperValue,
    steps: &'static StepperSteps,
    disabled: Has<InteractionDisabled>,
}

impl StepperViewItem<'_, '_> {
    /// The displayed position: the stored value, brought into range.
    pub(crate) fn current(&self) -> usize {
        clamp_index(self.value.get(), self.steps.get())
    }

    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub(crate) fn is_inline(&self) -> bool {
        self.stepper.is_inline()
    }

    pub(crate) fn fill(&self) -> Option<StepperFill> {
        self.stepper.fill()
    }

    /// The value `request` would change the stepper to, or `None` when it would not change it.
    pub(crate) fn change_for(&self, request: Move) -> Option<usize> {
        target(
            self.current(),
            self.steps.get(),
            self.stepper.wraps(),
            request,
        )
    }
}

/// Finds the stepper a part belongs to.
#[derive(SystemParam)]
pub(crate) struct StepperOwners<'w, 's> {
    steppers: Query<'w, 's, (), With<Stepper>>,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl StepperOwners<'_, '_> {
    /// The stepper `entity` is, or is a part of.
    pub(crate) fn of(&self, entity: Entity) -> Option<Entity> {
        iter_once(entity)
            .chain(self.parents.iter_ancestors(entity))
            .find(|candidate| self.steppers.contains(*candidate))
    }
}

/// A cell of a stepper: pressing it requests its position.
///
/// An app finds the cell of a position by this component, for example to decorate it with a child.
/// The stepper paints the cell itself and the mark under it, and leaves any other child alone.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(Button, PickingInteraction)]
pub struct StepperCell {
    index: usize,
}

impl StepperCell {
    pub(crate) fn new(index: usize) -> Self {
        Self { index }
    }

    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(Button, PickingInteraction)]
pub(crate) enum StepperButton {
    Prev,
    Next,
}

impl StepperButton {
    pub(crate) fn delta(self) -> isize {
        match self {
            Self::Prev => -1,
            Self::Next => 1,
        }
    }
}

/// The text between the buttons. Its content is the app's.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StepperLabel;

/// The part of a stepper a style system writes to.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepperPart {
    /// The row with the buttons and the label. An inline stepper has none.
    Header,
    Track,
    Mark,
    Arrow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn a_stepper_has_one_position_until_built_with_more() {
        assert_eq!(StepperSteps::default().get(), 1);
        assert_eq!(StepperSteps::new(10).get(), 10);
    }

    #[test]
    fn a_value_reads_back_what_it_was_made_with() {
        assert_eq!(StepperValue::new(7).get(), 7);
        assert_eq!(StepperValue::default().get(), 0);
    }

    #[test]
    fn marks_tell_which_cells_carry_one() {
        let marks = StepperMarks::new([3, 7]);
        assert!(marks.contains(3));
        assert!(marks.contains(7));
        assert!(!marks.contains(4));
        assert!(!StepperMarks::default().contains(0));
    }

    #[test]
    fn the_view_shows_the_position_in_range_and_what_a_move_would_do() {
        let mut world = World::new();
        let stepper = |wrap| {
            (
                Stepper::new(wrap, None, false),
                StepperValue::new(99),
                StepperSteps::new(10),
            )
        };
        let ends = world.spawn(stepper(false)).id();
        let wrapped = world.spawn(stepper(true)).id();

        let mut query = world.query::<StepperView>();
        let view = query.get(&world, ends).unwrap();
        assert_eq!(view.current(), 9);
        assert_eq!(view.change_for(Move::By(1)), None);
        assert_eq!(view.change_for(Move::By(-1)), Some(8));
        assert_eq!(
            query.get(&world, wrapped).unwrap().change_for(Move::By(1)),
            Some(0)
        );
    }

    #[test]
    fn the_owner_of_a_part_is_the_stepper_above_it() {
        let mut world = World::new();
        let stepper = world.spawn(Stepper::default()).id();
        let middle = world.spawn_empty().id();
        let leaf = world.spawn_empty().id();
        let stranger = world.spawn_empty().id();
        world.entity_mut(stepper).add_child(middle);
        world.entity_mut(middle).add_child(leaf);

        let mut state = SystemState::<StepperOwners>::new(&mut world);
        let owners = state.get(&world).unwrap();
        assert_eq!(owners.of(leaf), Some(stepper));
        assert_eq!(owners.of(stepper), Some(stepper));
        assert_eq!(owners.of(stranger), None);
    }

    #[test]
    fn the_buttons_step_in_opposite_directions() {
        assert_eq!(StepperButton::Prev.delta(), -1);
        assert_eq!(StepperButton::Next.delta(), 1);
    }
}
