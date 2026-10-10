# bevy_ui_component_prototypes

## Setup

```rust
app.add_plugins((DefaultPlugins, TabNavigationPlugin, UiComponentPrototypesPlugin));
```

`UiComponentPrototypesPlugin` provides theming only. It does not add `UiWidgetsPlugins`.
Add the plugins for the behavior you use yourself:

- `ButtonPlugin` (included in `DefaultPlugins`): button activation
- `InputDispatchPlugin` (included in `DefaultPlugins`): keyboard input to the focused entity (Tab, Enter, Space)
- `TabNavigationPlugin`: Tab-key focus navigation

Buttons emit `bevy::ui_widgets::Activate`. For Tab navigation, place them under a `TabGroup` ([example](examples/button_check.rs)).
