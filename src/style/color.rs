use bevy::color::Color;

// TODO organize

// Background ===========
pub(crate) const BG_NORMAL: Color = Color::srgb(0.25, 0.25, 0.30);
pub(crate) const BG_HOVERED: Color = Color::srgb(0.35, 0.35, 0.45);
pub(crate) const BG_PRESSED: Color = Color::srgb(0.15, 0.15, 0.20);

// Primary ===========
pub(crate) const PRIMARY_NORMAL: Color = Color::srgb(0.70, 0.30, 0.25);
pub(crate) const PRIMARY_HOVERED: Color = Color::srgb(0.85, 0.40, 0.30);
pub(crate) const PRIMARY_PRESSED: Color = Color::srgb(0.55, 0.20, 0.18);

// Affirmative (teal-green) ===========
pub(crate) const AFFIRM_NORMAL: Color = Color::srgb(0.18, 0.52, 0.48);
pub(crate) const AFFIRM_HOVERED: Color = Color::srgb(0.26, 0.66, 0.60);
pub(crate) const AFFIRM_PRESSED: Color = Color::srgb(0.12, 0.40, 0.37);

// Danger (red) ===========
pub(crate) const DANGER_NORMAL: Color = Color::srgb(0.66, 0.20, 0.18);
pub(crate) const DANGER_HOVERED: Color = Color::srgb(0.80, 0.28, 0.25);
pub(crate) const DANGER_PRESSED: Color = Color::srgb(0.50, 0.15, 0.14);

// Disabled ===========
pub(crate) const DISABLED: Color = Color::srgb(0.18, 0.18, 0.20);

// Text ===========
pub(crate) const TEXT_MAIN: Color = Color::WHITE;
pub(crate) const TEXT_MUTED: Color = Color::srgb(0.5, 0.5, 0.5);

// Border ===========
/// Subtle divider / outline on panels and buttons.
pub(crate) const BORDER: Color = Color::srgb(0.35, 0.35, 0.42);

// Focus ring ===========
pub(crate) const FOCUS_RING: Color = Color::srgb(0.85, 0.88, 1.0);
pub(crate) const EDITING: Color = Color::srgb(1.0, 0.75, 0.25);
