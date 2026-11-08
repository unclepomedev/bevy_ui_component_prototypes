mod area;
mod builder;
mod components;
mod entry;
mod parts;
mod style;
mod sync;
mod targets;
mod value;

pub use self::{
    builder::{TextFieldBuilder, text_field},
    components::{TextField, TextFieldInput},
    entry::Submit,
    style::{TextFieldMetrics, TextFieldShape, TextFieldStyle},
    value::SetFieldText,
};

use self::{
    area::TextAreaPlugin, entry::TextFieldEntryPlugin, sync::TextFieldStylePlugin,
    value::TextFieldValuePlugin,
};

use bevy::prelude::*;

pub(crate) struct TextFieldPlugin;
impl Plugin for TextFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TextFieldStylePlugin,
            TextFieldValuePlugin,
            TextFieldEntryPlugin,
            TextAreaPlugin,
        ));
    }
}
