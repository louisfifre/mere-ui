//! The value row, the molecule of a panel's settings: a label (wrapped, never cut, with its help mark after its last
//! word), an optional note (the value something else holds, said beside a control that differs from it), and the
//! control at the right, on one grid so that the controls of every row stand in one column:
//!
//! - the label column takes what is left (`1fr`), the note column what its words need, the control column the
//!   width the caller gives every row alike;
//! - the columns are `gap` apart, the row has `inset-hair` above and below, and the right padding the caller keeps
//!   for a scrollbar's gutter.
//!
//! The help is the kit's (help.rs): the mark shows while the pointer is on the row, and a press on it is reported;
//! the caller keeps which help is open and passes its text, and the panel opens over the row. Like every row, a value
//! row has no frame and no fill.

use mere_layout::{Align, Dim, Edges, Position, Style, Track, Tracks};
use mere_base::Vec2;

use crate::help::{HELP_MARK_W, HelpResp, HelpText, help_anchor, help_label};
use crate::id::Id;
use crate::text::{TextLook, Tone};
use crate::tokens::{GAP, GAP_TIGHT, INSET_HAIR};
use crate::ui::{Resp, Ui};

/// A value row.
#[derive(Copy, Clone, Debug, Default)]
pub struct ValueRow<'a> {
    pub label: &'a str,
    /// Said in the muted colour between the label and the control; empty for none.
    pub note: &'a str,
    /// The help mark's text (the application's "?").
    pub mark: &'a str,
    /// The row's width as the caller laid it out (a panel's): what the label's lines are broken at, since the layout
    /// gives no width before it runs.
    pub width: f32,
    /// The control column's width, the same for every row of a panel.
    pub control_w: f32,
    /// Room kept at the right for a scrollbar's gutter.
    pub gutter: f32,
}

/// The parts of a value row.
#[derive(Copy, Clone, Debug)]
pub struct ValueResp {
    /// The row: what the pointer shows the mark on and the help panel opens over.
    pub row: Id,
    pub label: Id,
    pub mark: Resp,
    /// The note, when there is one.
    pub note: Option<Id>,
    /// The help panel, while it is open.
    pub help: Option<HelpResp>,
}

/// The value rows' grid: the label, the note and the control.
pub fn value_grid(control_w: f32, gutter: f32) -> Style {
    Style {
        display: mere_layout::Display::Grid,
        cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Px(control_w)]),
        gap: Vec2::new(GAP, 0.0),
        align_items: Align::Center,
        pad: Edges::new(INSET_HAIR, gutter, INSET_HAIR, 0.0),
        ..Style::default()
    }
}

impl Ui {
    /// A value row: `control` builds the control in its column (a number field, a toggle). `help` is the text of its
    /// help while it is open, the mark then drawn open; `ValueResp::mark` tells a press on the mark, which the caller
    /// turns into opening or closing it.
    pub fn value_row(&mut self, key: &str, row: ValueRow, help: Option<HelpText>, control: impl FnOnce(&mut Ui)) -> ValueResp {
        let body = TextLook::body(Tone::Text);
        let muted = TextLook::body(Tone::Muted);
        // What the label has beside its mark: the row less its gutter, the gaps, the control, the note and the mark.
        let note_w = if row.note.is_empty() { 0.0 } else { muted.width(row.note) };
        let room = row.width - row.gutter - 2.0 * GAP - row.control_w - GAP_TIGHT - HELP_MARK_W - note_w;
        let open = help.is_some();
        let mut out = None;
        self.node(&format!("{key}Place"), help_anchor(), |ui| {
            let (mut label, mut mark, mut note) = (Id::ROOT, None, None);
            let id = ui.hover_area(key, value_grid(row.control_w, row.gutter), |ui, over| {
                let name = Style { position: Position::Relative, min_w: Dim::Px(0.0), ..Style::default() };
                let (place, at) = help_label(row.label, room);
                ui.node("name", name, |ui| {
                    label = ui.node("label", place, |ui| ui.paragraph(row.label, body));
                    mark = Some(ui.help_mark(row.mark, at, over.is_some(), open));
                });
                // Without a note the column keeps an empty text, so that it is there in every row alike.
                let n = ui.node("note", Style::default(), |ui| ui.text(row.note, muted));
                note = (!row.note.is_empty()).then_some(n);
                control(ui);
            });
            let help = help.map(|text| ui.help_panel(&format!("{key}Help"), id, text));
            out = Some(ValueResp { row: id, label, mark: mark.expect("the mark is built"), note, help });
        });
        out.expect("the row is built")
    }
}
