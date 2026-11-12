use super::{
    components::{
        Stepper, StepperButton, StepperCell, StepperLabel, StepperMarks, StepperPart, StepperSteps,
        StepperValue,
    },
    logic::StepperFill,
    nodes::{
        arrow_glyph, button_node, cell_node, header_node, icon_node, label_node, mark_node,
        stepper_node, track_node,
    },
    style::StepperShape,
};
use crate::common::{font_size::FONT_SIZE_MD, picking::pass_through};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};

type Extra = Box<dyn FnOnce(&mut EntityWorldMut) + Send + Sync>;

/// Starts a [`Stepper`] of one position, at 0.
pub fn stepper() -> StepperBuilder {
    StepperBuilder {
        steps: 1,
        value: 0,
        wrap: false,
        fill: None,
        marks: Vec::new(),
        label: None,
        compact: false,
        prev_icon: None,
        next_icon: None,
    }
}

/// Builder for a [`Stepper`]; start it with [`stepper`] and spawn what [`build`](Self::build) returns.
pub struct StepperBuilder {
    steps: usize,
    value: usize,
    wrap: bool,
    fill: Option<StepperFill>,
    marks: Vec<usize>,
    label: Option<Vec<Extra>>,
    compact: bool,
    prev_icon: Option<Handle<Image>>,
    next_icon: Option<Handle<Image>>,
}

impl StepperBuilder {
    /// The number of positions; fixed once built. Zero is a mistake and becomes 1.
    pub fn steps(mut self, steps: usize) -> Self {
        debug_assert!(steps > 0, "a stepper needs at least one position");
        self.steps = steps.max(1);
        self
    }

    /// The initial value. Later changes are the app's to apply.
    pub fn value(mut self, value: usize) -> Self {
        self.value = value;
        self
    }

    /// Whether stepping past an end continues from the other end. Off when not given.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Which cells are lit; the theme's choice when not given.
    pub fn fill(mut self, fill: StepperFill) -> Self {
        self.fill = Some(fill);
        self
    }

    /// The cells that carry a mark.
    pub fn marks(mut self, marks: impl IntoIterator<Item = usize>) -> Self {
        self.marks = marks.into_iter().collect();
        self
    }

    /// Adds `bundle` to the label entity, which is shown between the buttons only once this is called. Can be called more than once.
    /// The theme sets the label's `TextColor` and the font and the smoothing of its `TextFont`. The text and its size are the app's.
    pub fn label_bundle(mut self, bundle: impl Bundle) -> Self {
        self.label
            .get_or_insert_with(Vec::new)
            .push(Box::new(move |label| {
                label.insert(bundle);
            }));
        self
    }

    /// Drops the cells and keeps only the buttons and the label.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Replaces the arrow of the previous button.
    pub fn prev_icon(mut self, icon: Handle<Image>) -> Self {
        self.prev_icon = Some(icon);
        self
    }

    /// Replaces the arrow of the next button.
    pub fn next_icon(mut self, icon: Handle<Image>) -> Self {
        self.next_icon = Some(icon);
        self
    }

    /// The whole stepper: the root and its parts. Spawn it as it is.
    pub fn build(self) -> impl Bundle {
        let Self {
            steps,
            value,
            wrap,
            fill,
            marks,
            label,
            compact,
            prev_icon,
            next_icon,
        } = self;
        let shape = StepperShape::default();
        (
            Stepper::new(wrap, fill),
            StepperValue::new(value),
            StepperSteps::new(steps),
            StepperMarks::new(marks),
            stepper_node(&shape),
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                parent
                    .spawn((StepperPart::Header, header_node(&shape)))
                    .with_children(|header| {
                        spawn_button(header, &shape, StepperButton::Prev, prev_icon);
                        if let Some(extras) = label {
                            spawn_label(header, extras);
                        }
                        spawn_button(header, &shape, StepperButton::Next, next_icon);
                    });
                if !compact {
                    spawn_track(parent, &shape, steps);
                }
            })),
        )
    }
}

fn spawn_button(
    parent: &mut RelatedSpawner<ChildOf>,
    shape: &StepperShape,
    button: StepperButton,
    icon: Option<Handle<Image>>,
) {
    parent
        .spawn((button, button_node(shape)))
        .with_children(|content| match icon {
            Some(icon) => {
                content.spawn((
                    StepperPart::Arrow,
                    ImageNode::new(icon),
                    icon_node(shape),
                    pass_through(),
                ));
            }
            None => {
                content.spawn((
                    StepperPart::Arrow,
                    arrow_glyph(shape, button),
                    pass_through(),
                ));
            }
        });
}

fn spawn_label(parent: &mut RelatedSpawner<ChildOf>, extras: Vec<Extra>) {
    let mut label = parent.spawn((
        StepperLabel,
        Text::new(""),
        TextFont {
            font_size: FONT_SIZE_MD,
            ..default()
        },
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        label_node(),
        pass_through(),
    ));
    for extra in extras {
        extra(&mut label);
    }
}

fn spawn_track(parent: &mut RelatedSpawner<ChildOf>, shape: &StepperShape, steps: usize) {
    parent
        .spawn((StepperPart::Track, track_node(shape)))
        .with_children(|track| {
            for index in 0..steps {
                track
                    .spawn((StepperCell::new(index), cell_node(shape)))
                    .with_children(|cell| {
                        cell.spawn((StepperPart::Mark, mark_node(shape), pass_through()));
                    });
            }
        });
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input_focus::tab_navigation::TabIndex;

    #[derive(Component)]
    struct Caption;

    fn count<C: Component>(world: &mut World) -> usize {
        world.query::<&C>().iter(world).count()
    }

    fn spawn(builder: StepperBuilder) -> World {
        let mut world = World::new();
        world.spawn(builder.build());
        world
    }

    #[test]
    fn every_position_gets_a_cell_with_a_mark_place() {
        let mut world = spawn(stepper().steps(10));
        assert_eq!(count::<StepperCell>(&mut world), 10);
        assert_eq!(count::<StepperPart>(&mut world), 1 + 1 + 10 + 2);
        assert_eq!(count::<StepperButton>(&mut world), 2);
    }

    #[test]
    fn a_compact_stepper_has_no_cells() {
        let mut world = spawn(stepper().steps(5).compact());
        assert_eq!(count::<StepperCell>(&mut world), 0);
        assert_eq!(count::<StepperButton>(&mut world), 2);
    }

    #[test]
    fn the_label_exists_only_when_asked_for_and_carries_the_bundle() {
        let mut world = spawn(stepper());
        assert_eq!(count::<StepperLabel>(&mut world), 0);

        let mut world = spawn(stepper().label_bundle(Caption));
        let mut labels = world.query_filtered::<(), (With<StepperLabel>, With<Caption>)>();
        assert_eq!(labels.iter(&world).count(), 1);
    }

    #[test]
    fn the_stepper_is_the_only_tab_stop() {
        let mut world = spawn(stepper().steps(4).label_bundle(Caption));
        assert_eq!(count::<TabIndex>(&mut world), 1);
    }

    #[test]
    fn the_initial_settings_are_kept() {
        let mut world = spawn(
            stepper()
                .steps(10)
                .value(4)
                .wrap(true)
                .fill(StepperFill::Current)
                .marks([3, 7]),
        );
        let (body, value, steps, marks) = world
            .query::<(&Stepper, &StepperValue, &StepperSteps, &StepperMarks)>()
            .single(&world)
            .unwrap();
        assert!(body.wraps());
        assert_eq!(body.fill(), Some(StepperFill::Current));
        assert_eq!(value.get(), 4);
        assert_eq!(steps.get(), 10);
        assert!(marks.contains(3) && marks.contains(7) && !marks.contains(4));
    }

    #[test]
    fn a_stepper_follows_the_theme_unless_told_otherwise() {
        let mut world = spawn(stepper());
        let body = world.query::<&Stepper>().single(&world).unwrap();
        assert!(!body.wraps());
        assert_eq!(body.fill(), None);
    }

    #[test]
    fn an_icon_replaces_the_arrow_it_is_given_for() {
        let mut world = spawn(stepper().prev_icon(Handle::default()));
        assert_eq!(count::<ImageNode>(&mut world), 1);
        // Both buttons still have their picture, one of them an icon: the header, the track,
        // the mark of the one cell, and the two pictures.
        assert_eq!(count::<StepperPart>(&mut world), 1 + 1 + 1 + 2);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "at least one position")]
    fn no_positions_is_a_mistake() {
        let _ = stepper().steps(0);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn no_positions_becomes_one() {
        let mut world = spawn(stepper().steps(0));
        assert_eq!(count::<StepperCell>(&mut world), 1);
    }
}
