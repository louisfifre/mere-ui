//! The rows of a list, the molecules of the selection family: each is a list item that a group stacks and a menu or a
//! panel holds. They share one grid, so that whatever their kind their texts line up:
//!
//! - the lead column, `MARK_SIZE` wide: the selector of a selection row, the indicator of an immediate-selection row,
//!   nothing for an action row;
//! - the text column: the label on the first line, and, on a second line under the label only, the detail line.
//!
//! The lead faces the label's line and never the whole row, so a row with a detail line has its lead where a row
//! without one has it, and turning the detail on or off adds the second line and moves nothing else. Why a grid and
//! not a full-height lead cell beside a column of texts: both draw the same pixels at every width, but in the grid the
//! lead belongs to the label's line by construction, and the second line is the grid's second row; the earlier
//! layout got the same place only from a margin that had to agree with the label's line.
//!
//! A row has no frame and no fill of its own: the focused row is filled with the selection colour, and the
//! immediate-selection row with the tint while it is selected or under the pointer, both states rather than looks.
//! The frame and the ground come from what holds the rows (a menu's list, a panel).
//!
//! Every row asks and its caller decides: `Resp.clicked` means a press, or Space once the row has the focus, asked for
//! it; the caller changes what is selected (a flip for a multiple selector, this row for a single one or an
//! immediate-selection row) or runs the action. A row takes the focus as a field does (a press, Tab); inside an open
//! menu, Up and Down move the focus between its rows.

use mere_base::{Color, Cursor};
use mere_layout::{Align, Dim, Display, Edges, Style, Track, Tracks};
use mere_base::Vec2;

use crate::scroll::THUMB_W;
use crate::selector::{MARK_SIZE, MarkOpts, Selector};
use crate::text::{BODY_LINE, TextLook, Tone, XHEIGHT_EM, baseline};
use crate::tokens::{FRAME, GAP, INSET, INSET_TIGHT};
use crate::ui::{Paint, Resp, Ui};

/// A one-line row's height: the body's line and `inset-tight` above and below. A row with a detail line adds the
/// detail's line.
pub const LIST_ROW: f32 = BODY_LINE + 2.0 * INSET_TIGHT;
/// From the top of an open list's frame to the top of its first row: the frame and `inset-tight`.
pub const LIST_PAD: f32 = FRAME + INSET_TIGHT;
/// The indicator of a selected immediate-selection row: as wide as the scrollbar's thumb, the kit's other small bar,
/// and as tall as a selector, in its place.
pub const INDICATOR_W: f32 = THUMB_W;

/// A selection row: a selector facing its label, and the muted detail line under the label or not. The detail line is
/// `detail` and `detail_tail` joined by " · ", left out when both are empty; the label is cut at its end, `detail` is
/// kept whole and `detail_tail` is cut at its start, so that the end of a long path or address stays readable.
#[derive(Copy, Clone, Debug, Default)]
pub struct SelectionRow<'a> {
    pub label: &'a str,
    pub detail: &'a str,
    pub detail_tail: &'a str,
    pub selector: Selector,
    /// Shown as selected: the caller's value, which a press asks to change.
    pub selected: bool,
    /// Faded, no press, and out of the focus's walk.
    pub disabled: bool,
}

/// An immediate-selection row: a choice that applies as soon as it is pressed, with no selector and nothing to
/// validate after. The whole row is the target; the selected one is lit and marked by the indicator on its side.
#[derive(Copy, Clone, Debug, Default)]
pub struct ImmediateRow<'a> {
    pub label: &'a str,
    pub detail: &'a str,
    pub detail_tail: &'a str,
    pub selected: bool,
    pub disabled: bool,
}

/// What a row shows in its lead column.
enum Lead {
    Selector(Selector, bool, MarkOpts),
    Indicator(Color),
    Empty,
}

/// The texts of a row and their colours.
struct Texts<'a> {
    label: &'a str,
    detail: &'a str,
    tail: &'a str,
    color: Color,
    muted: Color,
    underline: bool,
}

/// The rows' grid: the lead column and the text column, `gap` apart; `inset-tight` above and below, `inset` at the
/// sides. Its rows are implicit: the label's line, then the detail line when there is one.
fn grid() -> Style {
    let cols = [Track::Px(MARK_SIZE), Track::MinMaxFr0(1.0)];
    Style { display: Display::Grid, cols: Tracks::new(&cols), gap: Vec2::new(GAP, 0.0), align_items: Align::Start, pad: Edges::vh(INSET_TIGHT, INSET), ..Style::default() }
}

/// How far below the top of the label's line a lead `h` tall sits: its middle on the middle of the label's lowercase
/// letters (the x-height above the baseline), on a whole pixel. Worked out from the text, so that it follows the
/// label's size; at the body's size it is the boards' 4 px.
fn lead_drop(look: TextLook, h: f32) -> f32 {
    (baseline(look) - XHEIGHT_EM * look.size / 2.0 - h / 2.0).round()
}

impl Ui {
    /// One row of the family, open on `id`.
    fn row(&mut self, key: &str, id: crate::id::Id, fill: Option<Color>, lead: Lead, texts: Texts) {
        let body = TextLook::body(Tone::Text);
        if self.menu_rows.is_some() {
            // Its texts whole: a list as wide as its rows (a tab's) takes the widest.
            let status = TextLook::status(Tone::Muted);
            let detail = match (texts.detail.is_empty(), texts.tail.is_empty()) {
                (false, false) => status.width(&format!("{} \u{b7} {}", texts.detail, texts.tail)),
                _ => status.width(texts.detail) + status.width(texts.tail),
            };
            let want = 2.0 * INSET + MARK_SIZE + GAP + body.width(texts.label).max(detail);
            self.menu_want = self.menu_want.max(want);
        }
        self.open(key, id, grid(), Paint::Box { fill, frame: None });
        let cell = Style { pad: Edges::new(lead_drop(body, MARK_SIZE), 0.0, 0.0, 0.0), ..Style::default() };
        match lead {
            Lead::Selector(kind, selected, opts) => {
                self.node("lead", cell, |ui| ui.selector_mark(kind, selected, opts));
            }
            Lead::Indicator(color) => {
                let bar = Style { w: Dim::Px(INDICATOR_W), h: Dim::Px(MARK_SIZE), shrink: 0.0, ..Style::default() };
                self.node("lead", cell, |ui| ui.fill(bar, color));
            }
            Lead::Empty => {
                self.add("lead", Style::default(), Paint::None);
            }
        }
        self.add_ellipsis(texts.label, body, texts.color, texts.underline);
        if !texts.detail.is_empty() || !texts.tail.is_empty() {
            // The second line goes under the label: the lead column's cell of that line stays empty.
            self.add("under", Style::default(), Paint::None);
            self.detail_line(texts.detail, texts.tail, texts.muted);
        }
        self.close();
    }

    /// A selection row: the selector facing the label, the detail line under the label. The whole row takes the
    /// press; the selector is lit under the pointer and the focused row is filled with the selection colour. A press,
    /// or Space once focused, is `Resp.clicked`: the person asked for the row to change, which the caller applies;
    /// the `Ui` then asks for one more frame (`want_redraw`) to show it.
    pub fn selection_row(&mut self, key: &str, row: SelectionRow) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, row.disabled, Cursor::Hand);
        let focused = self.focusable(&resp, row.disabled);
        resp.clicked |= self.row_keys(id, focused);
        if resp.clicked {
            // The caller applies the change after this row was painted: one more frame shows it.
            self.redraw = true;
        }
        let t = *self.theme();
        let off = |c| t.disabled_if(c, row.disabled);
        let mark = MarkOpts { disabled: row.disabled, lit: resp.hovered || resp.held, on_selection: focused };
        let texts = Texts { label: row.label, detail: row.detail, tail: row.detail_tail, color: off(if focused { t.select_text } else { t.text }), muted: off(t.muted), underline: false };
        self.row(key, id, focused.then_some(t.select), Lead::Selector(row.selector, row.selected, mark), texts);
        resp
    }

    /// An action row ("Choose a folder…"): no selector, its text in the text column so that it lines up with the
    /// labels of the rows around it, underlined under the pointer, filled when focused. A press, or Space once
    /// focused, is `Resp.clicked`: the caller runs the action, and closes the menu that holds it if the action wants
    /// (the line then has the focus back).
    pub fn action_row(&mut self, key: &str, text: &str) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, false, Cursor::Hand);
        let focused = self.focusable(&resp, false);
        resp.clicked |= self.row_keys(id, focused);
        let t = *self.theme();
        let texts = Texts { label: text, detail: "", tail: "", color: if focused { t.select_text } else { t.text }, muted: t.muted, underline: resp.hovered };
        self.row(key, id, focused.then_some(t.select), Lead::Empty, texts);
        resp
    }

    /// An immediate-selection row: a choice that applies at once. The whole row lights with the tint under the
    /// pointer and while it is the selected one, which also has the indicator in the lead column; the focused row is
    /// filled with the selection colour instead. A press, or Space once focused, is `Resp.clicked`: the caller makes
    /// this row the selected one and applies it.
    pub fn immediate_row(&mut self, key: &str, row: ImmediateRow) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, row.disabled, Cursor::Hand);
        let focused = self.focusable(&resp, row.disabled);
        resp.clicked |= self.row_keys(id, focused);
        if resp.clicked {
            self.redraw = true;
        }
        let t = *self.theme();
        let off = |c| t.disabled_if(c, row.disabled);
        let lit = resp.hovered || resp.held || row.selected;
        let fill = if focused { Some(t.select) } else if lit { Some(off(t.tint())) } else { None };
        let ink = if focused { t.select_text } else { t.text };
        let lead = if row.selected { Lead::Indicator(off(ink)) } else { Lead::Empty };
        let texts = Texts { label: row.label, detail: row.detail, tail: row.detail_tail, color: off(ink), muted: off(t.muted), underline: false };
        self.row(key, id, fill, lead, texts);
        resp
    }
}
