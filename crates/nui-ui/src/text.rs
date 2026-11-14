//! Text metrics as arithmetic: IBM Plex Mono is monospaced, so a width is a count times a constant and no
//! text-measuring trait is needed. The ellipsis (`ellipsize`) and the wrapping of a paragraph (`wrap_lines`) are here.

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
    /// `small`, line 109: 11 px with the body's unitless 1.5 line height inherited.
    pub fn small(tone: Tone) -> TextLook {
        TextLook { size: 11.0, line: 16.5, weight: Weight::Regular, tone, tracking: 0.0 }
    }
    /// `.label`, line 105: 11 px muted with `letter-spacing: 0.08em` (0.88 px). The caller uppercases.
    pub fn label() -> TextLook {
        TextLook { size: 11.0, line: 16.5, weight: Weight::Regular, tone: Tone::Muted, tracking: 0.88 }
    }
    /// One char's step: the glyph advance plus the tracking.
    pub(crate) fn step(&self) -> f32 {
        ADVANCE_EM * self.size + self.tracking
    }
    pub fn width(&self, s: &str) -> f32 {
        s.chars().count() as f32 * self.step()
    }
}

/// `text-overflow: ellipsis` with `white-space: nowrap`: `s` if it fits in `w`, else the longest whole-glyph
/// prefix that leaves room for the ellipsis, which is one monospaced cell (Chrome cuts at glyph boundaries).
pub fn ellipsize(s: &str, look: TextLook, w: f32) -> String {
    if look.width(s) <= w + 0.01 {
        return s.to_string();
    }
    let step = look.step();
    // The 1e-3 keeps a column that is an exact multiple of the step from losing a glyph to rounding.
    let n = ((w - step) / step + 1e-3).floor().max(0.0) as usize;
    let mut out: String = s.chars().take(n).collect();
    out.push('\u{2026}');
    out
}

/// CSS `white-space: normal` with `overflow-wrap: break-word`: words (split on any whitespace) are joined
/// by single spaces, a line takes as many as fit in `w`, and a word wider than `w` by itself is cut at whole
/// glyphs. Its last piece goes on as the current line, so the next word may follow it.
pub fn wrap_lines(s: &str, look: TextLook, w: f32) -> Vec<String> {
    wrap_lines_after(s, look, w, 0.0)
}

/// `wrap_lines` for text that follows something inline on its first line (the page's `.saved .path`: a label,
/// then the path with `overflow-wrap: anywhere`): `lead` px are taken off the first line only, the next lines
/// start at the left edge. A word too long for what is left of the first line is cut there, not moved down.
pub fn wrap_lines_after(s: &str, look: TextLook, w: f32, lead: f32) -> Vec<String> {
    let step = look.step();
    // A column narrower than one glyph still takes one glyph per line, as the browser does.
    let per_of = |room: f32| ((room + 0.01) / step).floor().max(1.0) as usize;
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cur_n = 0usize;
    for word in s.split_whitespace() {
        let room = if lines.is_empty() { w - lead } else { w } + 0.01;
        let n = word.chars().count();
        let joined = if cur_n == 0 { n } else { cur_n + 1 + n };
        if joined as f32 * step <= room {
            if cur_n > 0 {
                cur.push(' ');
            }
            cur.push_str(word);
            cur_n = joined;
            continue;
        }
        let first_open = lines.is_empty() && cur_n == 0 && lead > 0.0;
        if cur_n > 0 {
            lines.push(std::mem::take(&mut cur));
            cur_n = 0;
        }
        if !first_open && n as f32 * step <= w + 0.01 {
            cur.push_str(word);
            cur_n = n;
            continue;
        }
        let chars: Vec<char> = word.chars().collect();
        let mut at = 0;
        while at < chars.len() {
            let per = per_of(if lines.is_empty() { w - lead } else { w });
            let end = (at + per).min(chars.len());
            let piece: String = chars[at..end].iter().collect();
            at = end;
            if at < chars.len() {
                lines.push(piece);
            } else {
                cur_n = piece.chars().count();
                cur = piece;
            }
        }
    }
    if cur_n > 0 {
        lines.push(cur);
    }
    lines
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
