use crate::style::{button::ButtonMetrics, theme::Theme};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Component, Clone, Default)]
pub(crate) struct ButtonLabel;
#[derive(Component, Clone, Default)]
pub(crate) struct ButtonIcon;

#[derive(SystemParam)]
pub(super) struct ButtonContent<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    labels: Query<'w, 's, (&'static mut TextColor, &'static mut TextFont), With<ButtonLabel>>,
    icons: Query<'w, 's, (&'static mut ImageNode, &'static mut Node), With<ButtonIcon>>,
}

impl ButtonContent<'_, '_> {
    /// Applies the foreground color, the theme font and the size to the button's descendants.
    /// Without `metrics`, the sizes are left as they are.
    pub(super) fn apply(
        &mut self,
        root: Entity,
        fg: Color,
        metrics: Option<&ButtonMetrics>,
        theme: &Theme,
    ) {
        for descendant in self.children.iter_descendants(root) {
            if let Ok((mut color, mut text_font)) = self.labels.get_mut(descendant) {
                color.set_if_neq(TextColor(fg));
                if text_font.font != theme.font {
                    text_font.font = theme.font.clone();
                }
                if text_font.font_smoothing != theme.font_smoothing {
                    text_font.font_smoothing = theme.font_smoothing;
                }
                if let Some(metrics) = metrics
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
