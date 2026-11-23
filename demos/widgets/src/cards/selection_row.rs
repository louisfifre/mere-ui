//! The Selection Row card: a selector facing its label, and the muted detail line under the label or not. First both
//! kinds, without then with a detail line, in each state (rest, hover, held, focused, disabled), unselected then
//! selected; then the two-line row at three widths, which shows that the selector stays on the label's line and the
//! detail is cut at its start whatever the width. A row has no frame and no fill: the card shows it on the ground, as
//! a menu or a panel holds it. The Design System has no preview of it yet.

use mere_base::{Event, Vec2};
use mere_layout::{Dim, Style};
use mere_ui::{Id, SelectionRow, Selector, Theme, Ui};

use super::select::{Probe, centre, click};
use super::{Block, Card, Cell, State, render};

/// A panel's width.
const COL: f32 = 240.0;

/// The states a row is shown in: the harness's three, the focus (given by a press, the pointer then gone), and
/// disabled.
#[derive(Copy, Clone)]
pub(super) enum Look {
    Rest,
    Hover,
    Held,
    Focused,
    Disabled,
}

pub(super) const LOOKS: [Look; 5] = [Look::Rest, Look::Hover, Look::Held, Look::Focused, Look::Disabled];

/// A cell for what `build` makes (given whether it is disabled) in `look`. The focus is a press on it on the second
/// build, the pointer then moved away, so that the picture shows the focus without the hover.
pub(super) fn cell(look: Look, build: impl Fn(&mut Ui, bool) -> Id + 'static) -> Cell {
    let state = match look {
        Look::Hover => State::Hover,
        Look::Held => State::Held,
        _ => State::Rest,
    };
    let probe = Probe::new();
    Cell::new(state, move |ui: &mut Ui, _| {
        let due = if matches!(look, Look::Focused) { probe.due(ui) } else { None };
        if let Some(r) = due {
            click(ui, centre(r));
        }
        // As wide as the column, as a menu's list or a panel stretches its rows.
        let mut id = None;
        ui.node("col", Style { column: true, w: Dim::Pct(100.0), ..Style::default() }, |ui| id = Some(build(ui, matches!(look, Look::Disabled))));
        let id = id.unwrap();
        probe.set(id);
        if due.is_some() {
            ui.event(&Event::PointerMoved { pos: Vec2::new(-100.0, -100.0) });
        }
        id
    })
}

fn row(selector: Selector, selected: bool, detail: bool) -> SelectionRow<'static> {
    let (detail, detail_tail) = if detail { ("Archive", r"C:\Code\archive\2024\reports") } else { ("", "") };
    SelectionRow { label: "Reports", detail, detail_tail, selector, selected, disabled: false }
}

fn line(selector: Selector, selected: bool, detail: bool) -> Vec<Cell> {
    LOOKS.iter().map(|&look| cell(look, move |ui, disabled| ui.selection_row("r", SelectionRow { disabled, ..row(selector, selected, detail) }).id)).collect()
}

/// The two-line row of each kind at width `w`.
fn wide(w: f32) -> Block {
    let cells = [Selector::Multiple, Selector::Single].map(|s| cell(Look::Rest, move |ui, _| ui.selection_row("r", row(s, true, true)).id));
    Block::matrix(vec![cells.into_iter().collect()]).col_width(w)
}

pub fn blocks() -> Vec<Block> {
    let mut rows = Vec::new();
    for selector in [Selector::Multiple, Selector::Single] {
        for detail in [false, true] {
            rows.push(line(selector, false, detail));
            rows.push(line(selector, true, detail));
        }
    }
    vec![Block::matrix(rows).col_width(COL), wide(140.0), wide(200.0), wide(320.0)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
