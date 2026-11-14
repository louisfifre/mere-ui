//! The page's colours and sizes, dark and light, as plain values (preview.html's `:root` blocks).

use nui_base::Color;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Theme {
    /// `--ground`: the page's background.
    pub ground: Color,
    /// `--line`: separators.
    pub line: Color,
    /// `--text`: main text, and the filled button.
    pub text: Color,
    /// `--muted`: secondary text.
    pub muted: Color,
    /// `--warn`: the amber of failures.
    pub warn: Color,
    /// `--grid`: grid lines over the map (translucent).
    pub grid: Color,
    /// `--track`: the slider's track.
    pub track: Color,
    /// `--edge`: the hairline of fields and disabled buttons.
    pub edge: Color,
    /// `font: 12px/1.5` of the body.
    pub body: f32,
    /// The 11 px of the status lines.
    pub small: f32,
}

impl Theme {
    /// `:root`, lines 38-45.
    pub fn dark() -> Theme {
        Theme {
            ground: Color::hex(0x000000),
            line: Color::hex(0x2a2a2a),
            text: Color::hex(0xececec),
            muted: Color::hex(0x8a8a8a),
            warn: Color::hex(0xe5a44e),
            grid: Color::hex(0xffffff).with_alpha(0.16),
            track: Color::hex(0x6a6a6a),
            edge: Color::hex(0x4a4a4a),
            body: 12.0,
            small: 11.0,
        }
    }

    /// The light variables, lines 50-57. Chosen by the application: no OS detection yet (step 18).
    pub fn light() -> Theme {
        Theme {
            ground: Color::hex(0xfafaf8),
            line: Color::hex(0xdeded9),
            text: Color::hex(0x171717),
            muted: Color::hex(0x707070),
            warn: Color::hex(0xa85d00),
            grid: Color::hex(0x000000).with_alpha(0.12),
            track: Color::hex(0x9a9a96),
            edge: Color::hex(0xb4b4ae),
            body: 12.0,
            small: 11.0,
        }
    }

    /// What CSS `opacity` gives: `c` over the ground. Valid only when the ground is what lies behind, which
    /// is true for every faded thing of the page's columns.
    pub fn faded(&self, c: Color, opacity: f32) -> Color {
        mix(c, opacity, self.ground)
    }
}

/// `top` at `opacity` over `under`, opaque: what CSS `opacity` gives over a known backdrop.
pub fn mix(top: Color, opacity: f32, under: Color) -> Color {
    let m = |a: f32, b: f32| a * opacity + b * (1.0 - opacity);
    Color { r: m(top.r, under.r), g: m(top.g, under.g), b: m(top.b, under.b), a: 1.0 }
}
