use super::button::ButtonStyles;
use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default)]
pub struct Theme {
    pub buttons: ButtonStyles,
    pub font: FontSource,
    pub font_smoothing: FontSmoothing,
}

#[derive(Resource, Deref, DerefMut, Default, Clone)]
pub struct GlobalTheme(pub Theme);
