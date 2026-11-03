mod components;
mod style;
mod sync;

pub use self::{
    components::{ScrollIndicator, scroll_indicator},
    style::{ScrollIndicatorMetrics, ScrollIndicatorStyle},
};

pub(crate) use self::sync::ScrollIndicatorStylePlugin;
