use super::style::ButtonMetrics;
use crate::theme::Theme;
use bevy::{
    ecs::{lifecycle::HookContext, query::QueryData, system::SystemParam, world::DeferredWorld},
    prelude::*,
    ui_widgets::Button,
};

/// Marks a text inside a button as its label: the button gives it its foreground color,
/// the theme font and, from its size, the font size.
///
/// [`icon_button`](super::icon_button) spawns one. Put it on a text of your own to build content the
/// builder cannot, such as a card; the text may be nested at any depth under the button.
/// Add [`KeepFontSize`] to keep the size the text has.
#[derive(Component, Clone, Default)]
pub struct ButtonLabel;

/// Marks an image inside a button as its icon: the button tints it with the foreground color and,
/// from its size, makes it a square of `icon_size`. Like [`ButtonLabel`], it can be put on an
/// image of your own anywhere under the button; it needs an `ImageNode` and a `Node`.
#[derive(Component, Clone, Default)]
pub struct ButtonIcon;

/// On a [`ButtonLabel`]: the button colors the text but leaves its font size alone.
/// For a button that holds texts of several sizes.
#[derive(Component, Clone, Default)]
#[component(on_remove = restyle_owner)]
pub struct KeepFontSize;

/// On a [`ButtonLabel`] or a [`ButtonIcon`]: the button colors it with the style's
/// [`accent`](super::ButtonStyle::accent) instead of `fg`, so one part of the content can stand
/// out, such as the chevron of a card. A style without an accent colors it with `fg` as usual.
#[derive(Component, Clone, Default)]
#[component(on_add = restyle_owner, on_remove = restyle_owner)]
pub struct ButtonAccent;

/// Put on a button when a marker of its content changes, so that the next style sync visits it.
#[derive(Component)]
pub(crate) struct RestyleRequest;

/// Asks the button that owns the content `context.entity` to restyle it: the nearest button above it,
/// if there is one.
fn restyle_owner(mut world: DeferredWorld, context: HookContext) {
    let mut entity = context.entity;
    while let Some(parent) = world.get::<ChildOf>(entity).map(ChildOf::parent) {
        if world.get::<Button>(parent).is_some() {
            world.commands().entity(parent).try_insert(RestyleRequest);
            return;
        }
        entity = parent;
    }
}

#[derive(SystemParam)]
pub(crate) struct ButtonContent<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    labels: Query<'w, 's, LabelRow, With<ButtonLabel>>,
    icons: Query<'w, 's, IconRow, With<ButtonIcon>>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct LabelRow {
    color: &'static mut TextColor,
    font: &'static mut TextFont,
    keeps_size: Has<KeepFontSize>,
    accented: Has<ButtonAccent>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct IconRow {
    image: &'static mut ImageNode,
    node: &'static mut Node,
    accented: Has<ButtonAccent>,
}

impl ButtonContent<'_, '_> {
    /// Applies the colors, the theme font and the size to the button's descendants: `accent` to
    /// the content marked [`ButtonAccent`], `fg` to the rest.
    /// Without `metrics`, the sizes are left as they are.
    pub(crate) fn apply(
        &mut self,
        root: Entity,
        fg: Color,
        accent: Color,
        metrics: Option<&ButtonMetrics>,
        theme: &Theme,
    ) {
        for descendant in self.children.iter_descendants(root) {
            if let Ok(mut label) = self.labels.get_mut(descendant) {
                label
                    .color
                    .set_if_neq(TextColor(if label.accented { accent } else { fg }));
                if label.font.font != theme.font {
                    label.font.font = theme.font.clone();
                }
                if label.font.font_smoothing != theme.font_smoothing {
                    label.font.font_smoothing = theme.font_smoothing;
                }
                if let Some(metrics) = metrics
                    && !label.keeps_size
                    && label.font.font_size != metrics.font_size
                {
                    label.font.font_size = metrics.font_size;
                }
            }
            if let Ok(mut icon) = self.icons.get_mut(descendant) {
                let color = if icon.accented { accent } else { fg };
                if icon.image.color != color {
                    icon.image.color = color;
                }
                if let Some(metrics) = metrics {
                    let side = Val::Px(metrics.icon_size);
                    if icon.node.width != side {
                        icon.node.width = side;
                    }
                    if icon.node.height != side {
                        icon.node.height = side;
                    }
                }
            }
        }
    }
}
