# bevy_ui_component_prototypes

Boilerplate for UI components in Bevy games: themed widgets built on top of `bevy_ui` and `bevy_ui_widgets`.

The API is changing a lot at this stage. Expect breaking changes in every release.

Won't support WASM.

## Usage

The crate root only exposes `UiComponentPrototypesPlugin`, `GlobalTheme` and `Theme`. Each widget has its own module that holds its builder function, components and style types.

```rust
use bevy_ui_component_prototypes::{
    GlobalTheme, Theme, UiComponentPrototypesPlugin,
    slider::{SliderShape, slider}, common::BarSegments
};
```

Modules: `button`, `popup`, `slider`, `select`, `text_field`, `toggle_switch`, `progress`, `scroll_indicator`, `toast`, `tooltip`, `window`, and `common` (types shared by several widgets, such as `ShowsFocus` and `StateColors`).

## Behavior

State that every app would rebuild the same way (the lifetime of a toast, the tab order of disabled widgets) is managed by `UiComponentPrototypesPlugin`. State that depends on the app's policy (raising or closing a window) is asked for by an event, and applied by an opt-in plugin such as `WindowDefaultsPlugin`, or by your own observer.

## License

MIT or Apache-2.0
