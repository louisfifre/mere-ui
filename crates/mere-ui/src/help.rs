//! The help of a row: a mark after the row's label that opens a panel over the row, and the panel, which stays open
//! until its cross closes it and moves nothing.
//!
//! The application keeps which help is open and builds, inside one `help_anchor` box, the row and then, while it is
//! open, the panel:
//!
//! ```ignore
//! ui.node("place", help_anchor(), |ui| {
//!     let row = ui.hover_area("row", row_style, |ui, over| {
//!         let (label, mark) = help_label("Label", room);
//!         ui.node("name", Style { position: Position::Relative, ..Style::default() }, |ui| {
//!             ui.node("label", label, |ui| ui.paragraph("Label", TextLook::body(Tone::Text)));
//!             if ui.help_mark("?", mark, over.is_some(), open).clicked { open = !open; }
//!         });
//!     });
//!     if open && ui.help_panel("help", row, HelpText { title: "Label", hint: "What it does.", facts: "" }).closed {
//!         open = false;
//!     }
//! });
//! ```

use std::sync::OnceLock;

use mere_base::{Cmd, Cursor, Rect, Vec2, Weight, observe};
use mere_layout::{Align, Dim, Edges, Insets, Margin, Position, Style};

use crate::icon::Icon;
use crate::id::Id;
use crate::text::{TextLook, Tone, half_leading, wrap_lines};
use crate::tokens::{FRAME, GAP, GAP_TIGHT, ICON, INSET, INSET_TIGHT};
use crate::ui::{Resp, Ui};

/// The hit box of the mark after a label, and of a small icon: a few pixels each side of the glyph to aim at. A label's
/// room is measured without it, so that the mark always fits after the last word.
pub const HELP_MARK_W: f32 = ICON;
/// How far in the mark's glyph sits.
const MARK_IN: f32 = INSET_TIGHT;

/// The close cross: an X of 8 pixels in a 12 pixel square, one pixel per unit, so that it stays sharp.
const CROSS: &str = "M2 2h1v1h-1zM9 2h1v1h-1zM3 3h1v1h-1zM8 3h1v1h-1zM4 4h1v1h-1zM7 4h1v1h-1zM5 5h1v1h-1zM6 5h1v1h-1z\
M6 6h1v1h-1zM5 6h1v1h-1zM7 7h1v1h-1zM4 7h1v1h-1zM8 8h1v1h-1zM3 8h1v1h-1zM9 9h1v1h-1zM2 9h1v1h-1z";
/// The side of a small icon's square.
pub const SMALL_ICON: f32 = 12.0;

fn cross_icon() -> &'static Icon {
    static ICON: OnceLock<Icon> = OnceLock::new();
    ICON.get_or_init(|| {
        Icon::parse(CROSS, SMALL_ICON).unwrap_or_else(|e| {
            observe!(Error, "the help's cross: {e}");
            Icon { grid: SMALL_ICON, figures: Vec::new() }
        })
    })
}

/// What a help says: the title (the row's label), the hint under it, and a last smaller line of facts, empty for none.
#[derive(Copy, Clone, Debug)]
pub struct HelpText<'a> {
    pub title: &'a str,
    pub hint: &'a str,
    pub facts: &'a str,
}

/// The panel's parts, and whether its cross was clicked in this frame.
#[derive(Copy, Clone, Debug)]
pub struct HelpResp {
    pub panel: Id,
    pub title: Id,
    pub close: Id,
    pub hint: Id,
    /// None when the help has no facts.
    pub facts: Option<Id>,
    pub closed: bool,
}

/// What holds a row and the help that opens over it: the help is placed against this box and takes no room in it,
/// so opening it moves nothing.
pub fn help_anchor() -> Style {
    Style { column: true, shrink: 0.0, position: Position::Relative, ..Style::default() }
}

/// The box of a row's label and where its mark goes. The label wraps in `room` and its box is held to its widest
/// line (a shrunk box would keep the whole room and leave a gap); the mark follows the end of the last line, as a
/// word would. `room` is what the label has without the mark (`HELP_MARK_W` and the gap before it), so it always fits.
pub fn help_label(label: &str, room: f32) -> (Style, Vec2) {
    let body = TextLook::body(Tone::Text);
    let lines = wrap_lines(label, body, room);
    let w = lines.iter().map(|l| body.width(l)).fold(0.0, f32::max);
    let last = lines.last().map_or(0.0, |l| body.width(l));
    let mark = Vec2::new(last + GAP_TIGHT, (lines.len().max(1) - 1) as f32 * body.line);
    (Style { column: true, w: Dim::Px(w), shrink: 0.0, ..Style::default() }, mark)
}

impl Ui {
    /// The mark after a row's label (`mark`, the application's own glyph: a "?"): hidden until the pointer is on its
    /// row (`row`), then muted; in the text colour under the pointer and while its help is open, with the hand of what
    /// can be clicked. `at` is its place in the label's box (`help_label`), which is a positioned box. Hidden, it keeps
    /// its place, so nothing moves. A click is the application's to act on: it opens the help, or closes the open one.
    pub fn help_mark(&mut self, mark: &str, at: Vec2, row: bool, open: bool) -> Resp {
        let look = TextLook::body(Tone::Muted);
        let t = *self.theme();
        let style = Style {
            position: Position::Absolute,
            inset: Insets { left: Dim::Px(at.x), top: Dim::Px(at.y), ..Insets::default() },
            w: Dim::Px(HELP_MARK_W),
            h: Dim::Px(look.line),
            ..Style::default()
        };
        self.canvas("help", style, |ui, r| {
            let lit = r.hovered || r.held;
            if lit {
                ui.want_cursor(Cursor::Hand);
            }
            if !(row || lit || open) {
                return Vec::new();
            }
            let color = if lit || open { t.text } else { t.muted };
            vec![Cmd::Text { at: Vec2::new(MARK_IN, half_leading(look)), text: mark.to_string(), size: look.size, color, weight: look.weight }]
        })
    }

    /// A small icon to click beside a text (the cross that closes a help, an arrow that puts a group back): as wide as
    /// a help mark and `line` tall (the line of the text it sits beside, so that it adds no height), the icon centred
    /// in a `SMALL_ICON` square on whole pixels, muted, in the text colour under the pointer.
    pub fn small_icon(&mut self, key: &str, icon: &Icon, line: f32, margin: Margin) -> Resp {
        let t = *self.theme();
        let style = Style { w: Dim::Px(HELP_MARK_W), h: Dim::Px(line), shrink: 0.0, margin, ..Style::default() };
        self.canvas(key, style, |ui, r| {
            let lit = r.hovered || r.held;
            if lit {
                ui.want_cursor(Cursor::Hand);
            }
            let at = Rect::new(((HELP_MARK_W - SMALL_ICON) / 2.0).floor(), ((line - SMALL_ICON) / 2.0).floor(), SMALL_ICON, SMALL_ICON);
            vec![Cmd::Shape { rect: at, grid: icon.grid, figures: icon.figures.clone(), color: if lit { t.text } else { t.muted }, crisp: true }]
        })
    }

    /// A help open over its row until it is closed: a framed box on the ground colour that covers the row and what
    /// follows it without moving anything (`Ui::overlay`), built after the row inside the row's `help_anchor`. Its top
    /// is the row's top, so that the cross is at the end of the line whose mark opened it; only a panel that would
    /// pass the window's bottom is raised, by what it passes. The title has the close cross on its first line at the
    /// top right and wraps in the room the cross leaves; the hint and the facts take the panel's whole width under
    /// them. `row` is the row's id: the panel is placed on where the row stood in the last frame, which is where its
    /// mark was clicked.
    pub fn help_panel(&mut self, key: &str, row: Id, text: HelpText) -> HelpResp {
        let title_look = TextLook { weight: Weight::SemiBold, ..TextLook::body(Tone::Text) };
        let hint_look = TextLook::body(Tone::Muted);
        let facts_look = TextLook::small(Tone::Muted);
        let pad = FRAME + INSET;
        // The height is counted here, as the layout does not give it before the panel is laid out.
        let raise = self.rect(row).map_or(0.0, |r| {
            let inner = r.w - 2.0 * pad;
            let lines = |s: &str, look: TextLook, w: f32| wrap_lines(s, look, w).len().max(1) as f32 * look.line;
            let head = lines(text.title, title_look, inner - GAP - HELP_MARK_W).max(title_look.line);
            let facts_h = if text.facts.is_empty() { 0.0 } else { GAP_TIGHT + lines(text.facts, facts_look, inner) };
            let h = 2.0 * pad + head + GAP_TIGHT + lines(text.hint, hint_look, inner) + facts_h;
            (r.y + h - self.size().y).max(0.0).ceil()
        });
        let zero = Dim::Px(0.0);
        let style = Style {
            position: Position::Absolute,
            inset: Insets { left: zero, right: zero, top: Dim::Px(-raise), ..Insets::default() },
            column: true,
            pad: Edges::new(pad, pad, pad, pad),
            gap: Vec2::new(0.0, GAP_TIGHT),
            ..Style::default()
        };
        let mut resp = HelpResp { panel: Id::ROOT, title: Id::ROOT, close: Id::ROOT, hint: Id::ROOT, facts: None, closed: false };
        resp.panel = self.overlay(key, style, |ui| {
            ui.node("head", Style { align_items: Align::Start, gap: Vec2::new(GAP, 0.0), ..Style::default() }, |ui| {
                let title = Style { column: true, grow: 1.0, shrink: 1.0, basis: Dim::Px(0.0), min_w: Dim::Px(0.0), ..Style::default() };
                resp.title = ui.node("title", title, |ui| ui.paragraph(text.title, title_look));
                let r = ui.small_icon("close", cross_icon(), title_look.line, Margin::default());
                resp.close = r.id;
                resp.closed = r.clicked;
            });
            resp.hint = ui.node("text", Style { column: true, ..Style::default() }, |ui| ui.paragraph(text.hint, hint_look));
            if !text.facts.is_empty() {
                resp.facts = Some(ui.node("facts", Style { column: true, ..Style::default() }, |ui| ui.paragraph(text.facts, facts_look)));
            }
        });
        resp
    }
}
