mod area;
mod builder;
mod components;
mod parts;
mod style;
mod sync;
mod targets;
mod value;

pub use self::{
    builder::{TextFieldBuilder, text_field},
    components::{TextField, TextFieldInput},
    style::{TextFieldMetrics, TextFieldShape, TextFieldStyle},
    value::SetFieldText,
};

pub(crate) use self::{
    area::TextAreaPlugin, sync::TextFieldStylePlugin, value::TextFieldValuePlugin,
};
