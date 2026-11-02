use super::components::{
    InitialOption, LabelSpawner, Select, SelectArrow, SelectContent, SelectKey, SelectLabels,
    SelectList, SelectOptions, SelectPopup, SelectRequest, SelectRow, SelectScrollThumb,
    SelectScrollTrack, SelectTrigger, arrow_node, arrow_transform, content_node,
};
use crate::view::{button::ButtonLabel, pass_through};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::ValueChange,
};
use std::sync::Arc;

/// A select over the values of type `V`. Nothing is chosen at first.
pub fn select<V: SelectKey>() -> SelectBuilder<V> {
    SelectBuilder {
        values: Vec::new(),
        labels: Vec::new(),
        disabled: Vec::new(),
        placeholder: None,
        selected: None,
    }
}

pub struct SelectBuilder<V: SelectKey> {
    values: Vec<V>,
    labels: Vec<LabelSpawner>,
    disabled: Vec<bool>,
    placeholder: Option<LabelSpawner>,
    selected: Option<V>,
}

fn spawner<L: Bundle + Clone>(label: L) -> LabelSpawner {
    Arc::new(move |entity: &mut EntityWorldMut| {
        entity.insert(label.clone());
    })
}

fn choice_request<V: SelectKey>(values: Vec<V>) -> SelectRequest {
    SelectRequest {
        emit: Arc::new(
            move |commands: &mut Commands, select: Entity, index: usize| {
                if let Some(value) = values.get(index) {
                    commands.trigger(ValueChange {
                        source: select,
                        value: value.clone(),
                        is_final: true,
                    });
                }
            },
        ),
    }
}

impl<V: SelectKey> SelectBuilder<V> {
    /// Adds one option. The labels of different options may be of different types.
    /// The label is spawned again every time the option is shown, which is why it has to be `Clone`.
    /// A screen reader reads the `Text` of the label; a label without one is not read.
    pub fn option<L: Bundle + Clone>(self, value: V, label: L) -> Self {
        self.push(value, label, false)
    }

    /// Adds an option that is shown but cannot be chosen from the list. The arrow keys still pass over it, but
    /// it is not highlighted and Enter does nothing there. It can still be the chosen option (`selected`).
    pub fn option_disabled<L: Bundle + Clone>(self, value: V, label: L) -> Self {
        self.push(value, label, true)
    }

    fn push<L: Bundle + Clone>(mut self, value: V, label: L, disabled: bool) -> Self {
        self.values.push(value);
        self.labels.push(spawner(label));
        self.disabled.push(disabled);
        self
    }

    /// Adds the options of `items`, in order. Their labels are of one type; use `option` to mix.
    pub fn options<L: Bundle + Clone>(self, items: impl IntoIterator<Item = (V, L)>) -> Self {
        self.options_with_disabled(
            items
                .into_iter()
                .map(|(value, label)| (value, label, false)),
        )
    }

    /// Like `options`, with whether each option is disabled (see `option_disabled`).
    pub fn options_with_disabled<L: Bundle + Clone>(
        mut self,
        items: impl IntoIterator<Item = (V, L, bool)>,
    ) -> Self {
        for (value, label, disabled) in items {
            self = self.push(value, label, disabled);
        }
        self
    }

    /// Shown while nothing is chosen. A screen reader reads its `Text` too.
    pub fn placeholder<L: Bundle + Clone>(mut self, label: L) -> Self {
        self.placeholder = Some(spawner(label));
        self
    }

    /// The initial choice. A value that is not one of the options leaves nothing chosen.
    /// Later changes are the app's to apply.
    pub fn selected(mut self, value: V) -> Self {
        self.selected = Some(value);
        self
    }

    pub fn build(self) -> impl Bundle {
        let Self {
            values,
            labels,
            disabled,
            placeholder,
            selected,
        } = self;
        let option_labels = labels.clone();
        let request = choice_request(values.clone());
        (
            Select,
            InitialOption { value: selected },
            SelectOptions { values },
            request,
            SelectLabels {
                options: labels,
                placeholder,
            },
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                spawn_trigger(parent);
                spawn_popup(parent, &option_labels, &disabled);
            })),
        )
    }
}

fn spawn_trigger(parent: &mut RelatedSpawner<ChildOf>) {
    parent.spawn(SelectTrigger).with_children(|trigger| {
        trigger.spawn((SelectContent, content_node(), pass_through()));
        trigger.spawn((
            SelectArrow,
            arrow_node(),
            arrow_transform(),
            BorderColor::all(Color::NONE),
            pass_through(),
        ));
    });
}

fn spawn_popup(parent: &mut RelatedSpawner<ChildOf>, labels: &[LabelSpawner], disabled: &[bool]) {
    parent.spawn(SelectPopup).with_children(|popup| {
        popup.spawn(SelectList).with_children(|list| {
            for (index, (spawn_label, &disabled)) in labels.iter().zip(disabled).enumerate() {
                let mut row = list.spawn(SelectRow { index });
                if disabled {
                    row.insert(InteractionDisabled);
                }
                row.with_children(|row| spawn_option_label(row, spawn_label));
            }
        });
        popup.spawn(SelectScrollTrack).with_children(|track| {
            track.spawn(SelectScrollThumb);
        });
    });
}

fn spawn_option_label(row: &mut RelatedSpawner<ChildOf>, spawn_label: &LabelSpawner) {
    let mut label = row.spawn_empty();
    spawn_label(&mut label);
    // Unseen until the style sync colors it; a default `TextColor` is white.
    label.insert((ButtonLabel, TextColor(Color::NONE), pass_through()));
}
