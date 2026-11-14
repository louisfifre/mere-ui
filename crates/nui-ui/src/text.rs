//! Text metrics as arithmetic: IBM Plex Mono is monospaced, so a width is a count times a constant and no
//! text-measuring trait is needed. Single line only: no wrapping or ellipsis (steps 12-13).

use nui_base::Weight;

/// Advance of every glyph, in em: 600/1000 in all weights.
pub const ADVANCE_EM: f32 = 0.6;
/// Ascent in em. UNVERIFIED against the font tables; `tests/metrics.rs` checks it against a picture.
pub const ASCENT_EM: f32 = 1.025;
/// Descent in em. UNVERIFIED against the font tables, and not checked by any test.
pub const DESCENT_EM: f32 = 0.275;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Tone {
    Text,
    Muted,
    Warn,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextLook {
    pub size: f32,
    /// CSS `line-height` in px.
    pub line: f32,
    pub weight: Weight,
    pub tone: Tone,
    /// CSS `letter-spacing` in px, added after every char including the last, as browsers do.
    pub tracking: f32,
}

impl TextLook {
    /// The body: `font: 12px/1.5`.
    pub fn body(tone: Tone) -> TextLook {
        TextLook { size: 12.0, line: 18.0, weight: Weight::Regular, tone, tracking: 0.0 }
    }
    /// `#status`, line 189: `font-size: 11px; line-height: 16px`.
    pub fn status(tone: Tone) -> TextLook {
        TextLook { size: 11.0, line: 16.0, weight: Weight::Regular, tone, tracking: 0.0 }
    }
    /// `h2, .heading`, line 104: `600 12px/1.25`, `letter-spacing: 0.06em`.
    pub fn heading() -> TextLook {
        TextLook { size: 12.0, line: 15.0, weight: Weight::SemiBold, tone: Tone::Text, tracking: 0.72 }
    }
    /// One char's step: the glyph advance plus the tracking.
    pub(crate) fn step(&self) -> f32 {
        ADVANCE_EM * self.size + self.tracking
    }
    pub fn width(&self, s: &str) -> f32 {
        s.chars().count() as f32 * self.step()
    }
}

pub fn text_width(s: &str, size: f32) -> f32 {
    s.chars().count() as f32 * ADVANCE_EM * size
}

/// The CSS line box puts half the leading above the font's content; DirectWrite's line box is the content
/// box, so text is drawn this much below the CSS line top.
pub(crate) fn half_leading(look: TextLook) -> f32 {
    (look.line - (ASCENT_EM + DESCENT_EM) * look.size) / 2.0
}

/// Distance from the CSS line top to the baseline (12/18 gives 13.5, 1.125 em, the gallery's number).
pub(crate) fn baseline(look: TextLook) -> f32 {
    half_leading(look) + ASCENT_EM * look.size
}
