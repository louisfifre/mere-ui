//! Lines made once and assembled by several components. The line a list opens from (`Trigger::Line` of the menu,
//! which `select_menu` is built on), and the chevron a strip tab that opens a list (`Trigger::Tab`) takes. And the
//! detail line, the muted second line under a label (a menu's selection row).

use mere_base::Color;
use mere_layout::{Align, Dim, Edges, Margin, NodeId, Side, Style};

use crate::id::Id;
use crate::text::{TextLook, Tone};
use crate::tokens::{FRAME, INSET, INSET_TIGHT};
use crate::ui::{CHEVRON, Paint, Ui};

/// What a line differs by between the components that draw it.
#[derive(Copy, Clone, Debug)]
pub(crate) struct LineLook {
    /// The box's height, its frame included.
    pub height: f32,
    /// Faded to `FADE_DISABLED`.
    pub disabled: bool,
    /// Framed in the text colour instead of the edge: the line has the keyboard.
    pub focused: bool,
}

impl Ui {
    /// The chevron that says a list opens here, `CHEVRON` in size, `margin_left` after what comes before it.
    pub(crate) fn add_chevron(&mut self, margin_left: Side, color: Color) {
        let style = Style { w: Dim::Px(CHEVRON.x), h: Dim::Px(CHEVRON.y), shrink: 0.0, margin: Margin { left: margin_left, ..Margin::default() }, ..Style::default() };
        self.add("chevron", style, Paint::Chevron { color });
    }

    /// The line: a framed box as wide as its parent, its text cut at its end (as the browser's select does) and the
    /// chevron at its right, `inset` from the frame on the left and `inset-tight` on the right. Keyed `id`; the caller
    /// judges the pointer on it.
    pub(crate) fn menu_line(&mut self, key: &str, id: Id, text: &str, look: LineLook) -> NodeId {
        let t = *self.theme();
        let fade = |c| t.disabled_if(c, look.disabled);
        let frame = fade(if look.focused { t.text } else { t.edge });
        // The text is centred in the height: what is above and below it follows from the height alone.
        let style = Style {
            w: Dim::Pct(100.0),
            h: Dim::Px(look.height),
            pad: Edges::new(FRAME, FRAME + INSET_TIGHT, FRAME, FRAME + INSET),
            align_items: Align::Center,
            ..Style::default()
        };
        let node = self.open(key, id, style, Paint::Box { fill: None, frame: Some(frame) });
        self.add_ellipsis(text, TextLook::body(Tone::Text), fade(t.text), false);
        self.add_chevron(Side::Auto, fade(t.text));
        self.close();
        node
    }

    /// The detail line: `detail` and `tail` joined by " · " in `color`, `detail` kept whole and `tail` cut at its
    /// start, so that the end of a long path or address stays readable. Nothing when both are empty.
    pub(crate) fn detail_line(&mut self, detail: &str, tail: &str, color: Color) {
        if detail.is_empty() && tail.is_empty() {
            return;
        }
        // The status look (11 px on 16) keeps a two-line row on whole pixels, which the small look's 16.5 does not.
        let look = TextLook::status(Tone::Muted);
        self.node("detail", Style { min_w: Dim::Px(0.0), ..Style::default() }, |ui| {
            let lead = match (detail.is_empty(), tail.is_empty()) {
                (false, false) => format!("{detail} \u{b7} "),
                (false, true) => detail.to_string(),
                _ => String::new(),
            };
            if !lead.is_empty() {
                ui.add_text(&lead, look, color, false, Style { shrink: 0.0, ..Style::default() });
            }
            if !tail.is_empty() {
                ui.add_ellipsis_start(tail, look, color);
            }
        });
    }
}
