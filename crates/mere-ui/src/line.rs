//! The line a list opens from, made once for every component that opens one: the select box (`select_menu`) and the
//! menu (`menu`) draw it here, and a strip tab that opens a list (`tab_menu`) takes its chevron.

use mere_base::Color;
use mere_layout::{Align, Dim, Edges, Margin, NodeId, Side, Style};

use crate::id::Id;
use crate::text::{TextLook, Tone};
use crate::tokens::{FADE_DISABLED, FRAME, INSET, INSET_TIGHT};
use crate::ui::{CHEVRON, Paint, Ui};

/// What a line differs by between the components that draw it.
#[derive(Copy, Clone, Debug)]
pub(crate) struct LineLook {
    /// The box's height, its frame included.
    pub height: f32,
    /// The padding above and below the text, inside the frame.
    pub pad_v: f32,
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
        let fade = |c| if look.disabled { t.faded(c, FADE_DISABLED) } else { c };
        let frame = fade(if look.focused { t.text } else { t.edge });
        let v = FRAME + look.pad_v;
        let style = Style {
            w: Dim::Pct(100.0),
            h: Dim::Px(look.height),
            pad: Edges::new(v, FRAME + INSET_TIGHT, v, FRAME + INSET),
            align_items: Align::Center,
            ..Style::default()
        };
        let node = self.open(key, id, style, Paint::Box { fill: None, frame: Some(frame) });
        self.add_ellipsis(text, TextLook::body(Tone::Text), fade(t.text), false);
        self.add_chevron(Side::Auto, fade(t.text));
        self.close();
        node
    }
}
