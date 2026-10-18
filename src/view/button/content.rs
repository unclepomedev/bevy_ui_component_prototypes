use crate::style::theme::Theme;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Component, Clone, Default)]
pub(crate) struct ButtonLabel;
#[derive(Component, Clone, Default)]
pub(crate) struct ButtonIcon;

#[derive(SystemParam)]
pub(super) struct ButtonContent<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    labels: Query<'w, 's, (&'static mut TextColor, &'static mut TextFont), With<ButtonLabel>>,
    icons: Query<'w, 's, &'static mut ImageNode, With<ButtonIcon>>,
}

impl ButtonContent<'_, '_> {
    /// Applies the foreground color and the theme font to the button's descendants.
    pub(super) fn apply(&mut self, root: Entity, fg: Color, theme: &Theme) {
        for descendant in self.children.iter_descendants(root) {
            if let Ok((mut color, mut text_font)) = self.labels.get_mut(descendant) {
                color.set_if_neq(TextColor(fg));
                if text_font.font != theme.font {
                    text_font.font = theme.font.clone();
                }
                if text_font.font_smoothing != theme.font_smoothing {
                    text_font.font_smoothing = theme.font_smoothing;
                }
            }
            if let Ok(mut icon) = self.icons.get_mut(descendant)
                && icon.color != fg
            {
                icon.color = fg;
            }
        }
    }
}
