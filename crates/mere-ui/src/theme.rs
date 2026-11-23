//! The two themes: each role of the design system points to a colour of the palette (`tokens::palette`).

use mere_base::Color;

use crate::tokens::palette::*;
use crate::tokens::{FADE_DISABLED, FADE_TINT};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Theme {
    /// `bg-ground`: the background everything sits on; faded colours are mixed over it.
    pub ground: Color,
    /// `border-line`: separators, the scrollbar's track, the running button's progress.
    pub line: Color,
    /// `text-primary`: words and values.
    pub text: Color,
    /// `bg-fill`: a pressed or primary control's fill. A step softer than `text`, so a large filled surface
    /// does not glare against the ground.
    pub fill: Color,
    /// `text-on-fill`: the word or icon on `fill`.
    pub on_fill: Color,
    /// `text-muted`: secondary words, resting tab words and icons.
    pub muted: Color,
    /// `text-warn`: a failure.
    pub warn: Color,
    /// `border-grid`: lines laid over a picture such as the map (translucent).
    pub grid: Color,
    /// A slider's track. No widget of the kit paints it; the design system has no role for it.
    pub track: Color,
    /// `border-edge`: the frame of a resting control.
    pub edge: Color,
    /// `bg-selection` and `text-on-selection`: selected text in a field, the hot row of an open menu.
    pub select: Color,
    pub select_text: Color,
    /// `body` and `small` text sizes.
    pub body: f32,
    pub small: f32,
}

impl Theme {
    pub fn dark() -> Theme {
        let text = Color::hex(GREY_100);
        Theme {
            ground: Color::hex(GREY_1000),
            line: Color::hex(GREY_800),
            text,
            fill: Color::hex(GREY_200),
            on_fill: Color::hex(GREY_1000),
            muted: Color::hex(GREY_400),
            warn: Color::hex(AMBER_300),
            grid: Color::hex(GREY_0).with_alpha(0.16),
            track: Color::hex(GREY_600),
            edge: Color::hex(GREY_700),
            select: Color::hex(GREY_700),
            select_text: text,
            body: 12.0,
            small: 11.0,
        }
    }

    pub fn light() -> Theme {
        let text = Color::hex(GREY_950);
        Theme {
            ground: Color::hex(GREY_50),
            line: Color::hex(GREY_200),
            text,
            fill: Color::hex(GREY_800),
            on_fill: Color::hex(GREY_50),
            muted: Color::hex(GREY_600),
            warn: Color::hex(AMBER_600),
            grid: Color::hex(GREY_1000).with_alpha(0.12),
            track: Color::hex(GREY_400),
            edge: Color::hex(GREY_300),
            select: Color::hex(GREY_300),
            select_text: text,
            body: 12.0,
            small: 11.0,
        }
    }

    /// What CSS `opacity` gives: `c` over the ground. Valid only when the ground is what lies behind, which
    /// is true for every faded thing of the page's columns.
    pub fn faded(&self, c: Color, opacity: f32) -> Color {
        mix(c, opacity, self.ground)
    }

    /// The disabled look, the one rule every piece follows: each of its colours, fill and frame included, faded to
    /// `FADE_DISABLED` over the ground; `c` itself when `disabled` is false. A piece passes every colour it paints
    /// through here instead of fading by hand, so that "disabled" means the same thing everywhere.
    pub fn disabled_if(&self, c: Color, disabled: bool) -> Color {
        if disabled { self.faded(c, FADE_DISABLED) } else { c }
    }

    /// `bg-tint`: the faint fill of what lies under the pointer without being pressed (a window button, a hover box, an
    /// immediate-selection row), and of the selected immediate-selection row: the text colour at `FADE_TINT`.
    pub fn tint(&self) -> Color {
        mix(self.text, FADE_TINT, self.ground)
    }
}

/// `top` at `opacity` over `under`, opaque: what CSS `opacity` gives over a known backdrop.
pub fn mix(top: Color, opacity: f32, under: Color) -> Color {
    let m = |a: f32, b: f32| a * opacity + b * (1.0 - opacity);
    Color { r: m(top.r, under.r), g: m(top.g, under.g), b: m(top.b, under.b), a: 1.0 }
}
