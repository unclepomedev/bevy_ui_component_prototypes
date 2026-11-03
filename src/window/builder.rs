use super::{
    close::close_on_activate,
    components::{
        UiWindow, WindowContent, WindowGrip, WindowPart, WindowRect, WindowTitle, close_node,
        content_node, glyph_bar_node, glyph_bar_transform, glyph_box_node, grip_node,
        title_bar_node, title_node,
    },
    drag::drag_title_bar,
    requests::WindowEdge,
    resize::drag_grip,
    style::{WindowMetrics, WindowStyle},
};
use crate::common::picking::pass_through;
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    picking::hover::PickingInteraction,
    prelude::*,
};

type Extra = Box<dyn FnOnce(&mut EntityWorldMut) + Send + Sync>;

pub struct WindowBuilder {
    rect: WindowRect,
    title_extras: Vec<Extra>,
    content_extras: Vec<Extra>,
}

/// A window with the default [`WindowRect`]. Its parent has to be a node that places it.
pub fn window() -> WindowBuilder {
    WindowBuilder {
        rect: WindowRect::default(),
        title_extras: Vec::new(),
        content_extras: Vec::new(),
    }
}

impl WindowBuilder {
    pub fn rect(mut self, rect: WindowRect) -> Self {
        self.rect = rect;
        self
    }

    /// Adds `bundle` to the title entity, which is styled as text: give it a `Text`.
    /// Can be called more than once.
    pub fn title(mut self, bundle: impl Bundle) -> Self {
        self.title_extras.push(Box::new(move |title| {
            title.insert(bundle);
        }));
        self
    }

    /// Adds `bundle` to the [`WindowContent`] entity. Can be called more than once.
    pub fn content_bundle(mut self, bundle: impl Bundle) -> Self {
        self.content_extras.push(Box::new(move |content| {
            content.insert(bundle);
        }));
        self
    }

    pub fn build(self) -> impl Bundle {
        let Self {
            rect,
            title_extras,
            content_extras,
        } = self;
        (
            UiWindow,
            rect,
            Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                spawn_title_bar(parent, title_extras);
                let mut content = parent.spawn((WindowContent, content_node()));
                for extra in content_extras {
                    extra(&mut content);
                }
                spawn_grips(parent);
            })),
        )
    }
}

/// The parts start with the default style and metrics; the first style sync gives them the theme's.
fn spawn_title_bar(parent: &mut RelatedSpawner<ChildOf>, title_extras: Vec<Extra>) {
    let style = WindowStyle::default();
    let metrics = WindowMetrics::default();
    parent
        .spawn((
            WindowPart::TitleBar,
            title_bar_node(&metrics, &style.shape, style.close.side),
            BackgroundColor(Color::NONE),
        ))
        .observe(drag_title_bar)
        .with_children(|bar| {
            spawn_close(bar, &style, &metrics);
            let mut title = bar.spawn((
                WindowTitle,
                title_node(),
                TextLayout::new(Justify::Left, LineBreak::NoWrap),
                // Unseen until the first style sync.
                TextColor(Color::NONE),
                pass_through(),
            ));
            for extra in title_extras {
                extra(&mut title);
            }
        });
}

/// The grips come after the content, so they are above it.
fn spawn_grips(parent: &mut RelatedSpawner<ChildOf>) {
    let style = WindowStyle::default();
    let metrics = WindowMetrics::default();
    for edge in WindowEdge::ALL {
        parent
            .spawn((
                WindowPart::Grip,
                WindowGrip(edge),
                grip_node(edge, &metrics, &style.shape),
                BackgroundColor(Color::NONE),
            ))
            .observe(drag_grip);
    }
}

fn spawn_close(bar: &mut RelatedSpawner<ChildOf>, style: &WindowStyle, metrics: &WindowMetrics) {
    bar.spawn((
        WindowPart::Close,
        close_node(metrics, style),
        BackgroundColor(Color::NONE),
        PickingInteraction::default(),
        bevy::ui_widgets::Button,
    ))
    .observe(close_on_activate)
    .with_children(|close| {
        close
            .spawn((
                WindowPart::GlyphBox,
                glyph_box_node(metrics),
                BackgroundColor(Color::NONE),
                pass_through(),
            ))
            .with_children(|glyph| {
                for degrees in [45.0, -45.0] {
                    glyph.spawn((
                        WindowPart::GlyphBar,
                        glyph_bar_node(metrics),
                        glyph_bar_transform(degrees),
                        BackgroundColor(Color::NONE),
                        pass_through(),
                    ));
                }
            });
    });
}
