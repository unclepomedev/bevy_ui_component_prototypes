use bevy::color::Color;

// TODO organize

// Background ===========
pub const BG_NORMAL: Color = Color::srgb(0.25, 0.25, 0.30);
pub const BG_HOVERED: Color = Color::srgb(0.35, 0.35, 0.45);
pub const BG_PRESSED: Color = Color::srgb(0.15, 0.15, 0.20);

// Primary ===========
pub const PRIMARY_NORMAL: Color = Color::srgb(0.70, 0.30, 0.25);
pub const PRIMARY_HOVERED: Color = Color::srgb(0.85, 0.40, 0.30);
pub const PRIMARY_PRESSED: Color = Color::srgb(0.55, 0.20, 0.18);

// Affirmative (teal-green) ===========
pub const AFFIRM_NORMAL: Color = Color::srgb(0.18, 0.52, 0.48);
pub const AFFIRM_HOVERED: Color = Color::srgb(0.26, 0.66, 0.60);
pub const AFFIRM_PRESSED: Color = Color::srgb(0.12, 0.40, 0.37);

// Danger (red) ===========
pub(crate) const DANGER_NORMAL: Color = Color::srgb(0.66, 0.20, 0.18);
pub(crate) const DANGER_HOVERED: Color = Color::srgb(0.80, 0.28, 0.25);
pub(crate) const DANGER_PRESSED: Color = Color::srgb(0.50, 0.15, 0.14);

// Disabled ===========
pub const DISABLED: Color = Color::srgb(0.18, 0.18, 0.20);

// Text ===========
pub const TEXT_MAIN: Color = Color::WHITE;
pub const TEXT_MUTED: Color = Color::srgb(0.5, 0.5, 0.5);

// Border ===========
/// Subtle divider / outline on panels and buttons.
pub const BORDER: Color = Color::srgb(0.35, 0.35, 0.42);
