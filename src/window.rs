mod builder;
mod close;
mod components;
mod defaults;
mod drag;
mod limits;
mod maximize;
mod requests;
mod resize;
mod stack;
mod style;
mod sync;

pub use self::{
    builder::{WindowBuilder, window},
    components::{
        ActiveWindow, Maximizable, Maximized, UiWindow, WindowContent, WindowGrip, WindowRect,
    },
    defaults::WindowDefaultsPlugin,
    limits::WindowLimits,
    requests::{
        CloseWindow, MoveWindow, RaiseWindow, ResizeWindow, ToggleMaximizeWindow, WindowEdge,
    },
    stack::WindowStack,
    style::{
        CloseShape, CloseStyle, MaximizeStyle, TitleAlign, TitleBarStyle, WindowMetrics,
        WindowShape, WindowSide, WindowStyle,
    },
};

pub(crate) use self::sync::WindowStylePlugin;
