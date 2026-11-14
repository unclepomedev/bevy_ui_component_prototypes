use super::style::ButtonMetrics;
use crate::theme::Theme;
use bevy::{ecs::system::SystemParam, prelude::*};

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
pub struct KeepFontSize;

#[derive(SystemParam)]
pub(crate) struct ButtonContent<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    labels: Query<
        'w,
        's,
        (
            &'static mut TextColor,
            &'static mut TextFont,
            Has<KeepFontSize>,
        ),
        With<ButtonLabel>,
    >,
    icons: Query<'w, 's, (&'static mut ImageNode, &'static mut Node), With<ButtonIcon>>,
}

impl ButtonContent<'_, '_> {
    /// Applies the foreground color, the theme font and the size to the button's descendants.
    /// Without `metrics`, the sizes are left as they are.
    pub(crate) fn apply(
        &mut self,
        root: Entity,
        fg: Color,
        metrics: Option<&ButtonMetrics>,
        theme: &Theme,
    ) {
        for descendant in self.children.iter_descendants(root) {
            if let Ok((mut color, mut text_font, keeps_size)) = self.labels.get_mut(descendant) {
                color.set_if_neq(TextColor(fg));
                if text_font.font != theme.font {
                    text_font.font = theme.font.clone();
                }
                if text_font.font_smoothing != theme.font_smoothing {
                    text_font.font_smoothing = theme.font_smoothing;
                }
                if let Some(metrics) = metrics
                    && !keeps_size
                    && text_font.font_size != metrics.font_size
                {
                    text_font.font_size = metrics.font_size;
                }
            }
            if let Ok((mut icon, mut node)) = self.icons.get_mut(descendant) {
                if icon.color != fg {
                    icon.color = fg;
                }
                if let Some(metrics) = metrics {
                    let side = Val::Px(metrics.icon_size);
                    if node.width != side {
                        node.width = side;
                    }
                    if node.height != side {
                        node.height = side;
                    }
                }
            }
        }
    }
}
