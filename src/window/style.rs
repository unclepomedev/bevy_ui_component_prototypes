use crate::common::{
    color::{
        BG_HOVERED, BG_NORMAL, BG_PRESSED, BORDER, DANGER_HOVERED, DANGER_NORMAL, DANGER_PRESSED,
        DISABLED, TEXT_MAIN, TEXT_MUTED,
    },
    state::{StateColors, VisualState},
};
use bevy::prelude::*;

/// Which end of the title bar the close button is at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowSide {
    Left,
    #[default]
    Right,
}

/// How the title is aligned in the space the close button leaves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TitleAlign {
    #[default]
    Start,
    Center,
}

/// The corners of the close button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CloseShape {
    /// Fully rounded: a circle for a square button.
    Circle,
    #[default]
    Square,
}

/// Border width and corner radius of a window, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowShape {
    pub border_width: f32,
    pub radius: f32,
}

impl Default for WindowShape {
    fn default() -> Self {
        Self {
            border_width: 1.0,
            radius: 0.0,
        }
    }
}

/// Colors of the title bar. `fg` is the color of the title text.
/// The `inactive_` colors are for a window that does not have `ActiveWindow`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TitleBarStyle {
    pub bg: Color,
    pub fg: Color,
    pub inactive_bg: Color,
    pub inactive_fg: Color,
    pub align: TitleAlign,
}

impl TitleBarStyle {
    /// The background and the text color of the title bar.
    pub(crate) fn colors(&self, active: bool) -> (Color, Color) {
        if active {
            (self.bg, self.fg)
        } else {
            (self.inactive_bg, self.inactive_fg)
        }
    }
}

/// Colors and place of the close button. The glyph is the cross drawn on it.
#[derive(Clone, Copy, Debug)]
pub struct CloseStyle {
    pub side: WindowSide,
    pub shape: CloseShape,
    pub bg: StateColors,
    pub glyph: StateColors,
}

/// Colors and shape of a window: the frame, the title bar, and the close button.
#[derive(Clone, Copy, Debug)]
pub struct WindowStyle {
    /// The color of the frame and of the content area.
    pub bg: Color,
    pub border: Color,
    pub shape: WindowShape,
    pub title_bar: TitleBarStyle,
    pub close: CloseStyle,
}

/// The footprint of a window: the title bar, the close button, and its glyph. In logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowMetrics {
    pub title_height: f32,
    pub title_padding: UiRect,
    /// The space between the close button and the title.
    pub title_gap: f32,
    pub title_font_size: FontSize,
    pub close_width: f32,
    pub close_height: f32,
    /// The side of the square that the cross is drawn in.
    pub glyph_size: f32,
    pub glyph_stroke: f32,
}

/// `normal` when the pointer is away, `hovered` over it, `pressed` while held; there is no focus or disabled look.
fn states(normal: Color, hovered: Color, pressed: Color) -> StateColors {
    StateColors {
        normal,
        hovered,
        pressed,
        disabled: normal,
        focused: normal,
    }
}

impl WindowStyle {
    /// Circular close button at the left of a centered title; the cross shows only under the pointer.
    pub fn mac() -> Self {
        Self {
            bg: BG_NORMAL,
            border: BORDER,
            shape: WindowShape {
                border_width: 1.0,
                radius: 10.0,
            },
            title_bar: TitleBarStyle {
                bg: BG_HOVERED,
                fg: TEXT_MAIN,
                inactive_bg: BG_NORMAL,
                inactive_fg: TEXT_MUTED,
                align: TitleAlign::Center,
            },
            close: CloseStyle {
                side: WindowSide::Left,
                shape: CloseShape::Circle,
                bg: states(DANGER_NORMAL, DANGER_HOVERED, DANGER_PRESSED),
                glyph: states(Color::NONE, BG_PRESSED, BG_PRESSED),
            },
        }
    }

    /// Square close button at the right of a left-aligned title; it turns red under the pointer.
    pub fn windows() -> Self {
        Self {
            bg: BG_NORMAL,
            border: BORDER,
            shape: WindowShape::default(),
            title_bar: TitleBarStyle {
                bg: BG_PRESSED,
                fg: TEXT_MAIN,
                inactive_bg: DISABLED,
                inactive_fg: TEXT_MUTED,
                align: TitleAlign::Start,
            },
            close: CloseStyle {
                side: WindowSide::Right,
                shape: CloseShape::Square,
                bg: states(Color::NONE, DANGER_NORMAL, DANGER_PRESSED),
                glyph: StateColors::uniform(TEXT_MAIN),
            },
        }
    }
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self::windows()
    }
}

impl WindowMetrics {
    pub fn mac() -> Self {
        Self {
            title_height: 28.0,
            title_padding: UiRect::horizontal(Val::Px(10.0)),
            title_gap: 8.0,
            title_font_size: FontSize::Px(16.0),
            close_width: 14.0,
            close_height: 14.0,
            glyph_size: 8.0,
            glyph_stroke: 2.0,
        }
    }

    pub fn windows() -> Self {
        Self {
            title_height: 32.0,
            title_padding: UiRect::left(Val::Px(10.0)),
            title_gap: 8.0,
            title_font_size: FontSize::Px(16.0),
            close_width: 44.0,
            close_height: 32.0,
            glyph_size: 12.0,
            glyph_stroke: 2.0,
        }
    }
}

impl Default for WindowMetrics {
    fn default() -> Self {
        Self::windows()
    }
}

/// What the close button looks like in one state, resolved from a [`CloseStyle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CloseLook {
    pub(crate) bg: Color,
    pub(crate) glyph: Color,
}

impl CloseStyle {
    pub(crate) fn look(&self, state: VisualState) -> CloseLook {
        CloseLook {
            bg: self.bg.pick(state),
            glyph: self.glyph.pick(state),
        }
    }

    /// The radius of the button: half of its shorter side for a circle.
    pub(crate) fn radius(&self, metrics: &WindowMetrics) -> f32 {
        match self.shape {
            CloseShape::Circle => metrics.close_width.min(metrics.close_height) / 2.0,
            CloseShape::Square => 0.0,
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_presets_differ_in_what_a_user_sees() {
        let (mac, windows) = (WindowStyle::mac(), WindowStyle::windows());
        assert_ne!(mac.close.side, windows.close.side);
        assert_ne!(mac.close.shape, windows.close.shape);
        assert_ne!(mac.title_bar.align, windows.title_bar.align);
    }

    #[test]
    fn the_default_is_one_of_the_presets() {
        assert_eq!(
            WindowStyle::default().close.side,
            WindowStyle::windows().close.side
        );
        assert_eq!(WindowMetrics::default(), WindowMetrics::windows());
    }

    #[test]
    fn an_inactive_title_bar_uses_the_inactive_colors() {
        for style in [WindowStyle::mac(), WindowStyle::windows()] {
            let bar = style.title_bar;
            assert_eq!(bar.colors(true), (bar.bg, bar.fg));
            assert_eq!(bar.colors(false), (bar.inactive_bg, bar.inactive_fg));
            assert_ne!(bar.colors(true), bar.colors(false));
        }
    }

    #[test]
    fn the_close_look_follows_the_visual_state() {
        let close = WindowStyle::mac().close;
        let look = close.look(VisualState::Pressed);
        assert_eq!(look.bg, close.bg.pressed);
        assert_eq!(look.glyph, close.glyph.pressed);
    }

    #[test]
    fn the_mac_cross_shows_only_under_the_pointer() {
        let close = WindowStyle::mac().close;
        assert_eq!(close.look(VisualState::Normal).glyph, Color::NONE);
        assert_ne!(close.look(VisualState::Hovered).glyph, Color::NONE);
    }

    #[test]
    fn a_circle_is_half_of_the_shorter_side_and_a_square_has_no_radius() {
        let metrics = WindowMetrics {
            close_width: 30.0,
            close_height: 20.0,
            ..WindowMetrics::default()
        };
        let mut close = WindowStyle::mac().close;
        assert_eq!(close.radius(&metrics), 10.0);
        close.shape = CloseShape::Square;
        assert_eq!(close.radius(&metrics), 0.0);
    }
}
