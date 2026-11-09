mod builder;
mod close;
mod components;
mod defaults;
mod drag;
mod limits;
mod requests;
mod resize;
mod stack;
mod style;
mod sync;

pub use self::{
    builder::{WindowBuilder, window},
    components::{ActiveWindow, UiWindow, WindowContent, WindowGrip, WindowRect},
    defaults::WindowDefaultsPlugin,
    limits::WindowLimits,
    requests::{CloseWindow, MoveWindow, RaiseWindow, ResizeWindow, WindowEdge},
    stack::WindowStack,
    style::{
        CloseShape, CloseStyle, TitleAlign, TitleBarStyle, WindowMetrics, WindowShape, WindowSide,
        WindowStyle,
    },
};

pub(crate) use self::sync::WindowStylePlugin;
