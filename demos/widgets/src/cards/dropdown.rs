//! The Dropdown card: the line closed and disabled; then open in each kind of choice, several with check boxes and one
//! with radio buttons (one row disabled); then the same two with two-line rows whose long detail is cut at its start,
//! a separator and an action row; and a list hung above its line. The Design System has no preview of it yet. An open
//! list is opened by a press on its line, so its first row has the focus, as it does when the list opens.

use std::cell::Cell as Memo;

use mere_layout::{Dim, Justify, Style};
use mere_ui::{DropdownOpts, Id, Mark, OptionRow, Theme, Ui};

use super::select::{Probe, centre, click};
use super::{Block, Card, Cell, State, render};

/// A panel's width.
const COL: f32 = 280.0;

/// What a list holds, in order.
#[derive(Copy, Clone)]
enum Item {
    Row(OptionRow<'static>),
    Separator,
    Action(&'static str),
}

const fn row(label: &'static str, mark: Mark, selected: bool) -> Item {
    Item::Row(OptionRow { label, detail: "", detail_tail: "", mark, selected, disabled: false })
}

const fn off(label: &'static str, mark: Mark) -> Item {
    Item::Row(OptionRow { label, detail: "", detail_tail: "", mark, selected: false, disabled: true })
}

const fn two(label: &'static str, detail: &'static str, detail_tail: &'static str, mark: Mark, selected: bool) -> Item {
    Item::Row(OptionRow { label, detail, detail_tail, mark, selected, disabled: false })
}

const TOPPINGS: &[Item] = &[row("Cheese", Mark::Check, true), row("Tomato", Mark::Check, false), row("Basil", Mark::Check, true), row("Olives", Mark::Check, false)];
const SIZES: &[Item] = &[row("Small", Mark::Radio, false), row("Medium", Mark::Radio, true), row("Large", Mark::Radio, false), off("Extra large", Mark::Radio)];
const BACKUP: &[Item] = &[
    two("Reports", "Archive", r"D:\Archive\2024\reports\quarterly\final versions", Mark::Check, true),
    two("Sketches", "Code", r"C:\Code\sketches\2024\interface studies", Mark::Check, false),
    two("Notes", "", "", Mark::Check, true),
    Item::Separator,
    Item::Action("Browse\u{2026}"),
];
const SAVE_TO: &[Item] = &[
    two("Documents", "This computer", r"C:\Users\someone\Documents\Letters\2024\drafts", Mark::Radio, true),
    two("Projects", "This computer", r"C:\Code\projects", Mark::Radio, false),
    two("Archive", "External drive", r"D:\Archive\2024", Mark::Radio, false),
    Item::Separator,
    Item::Action("Choose a folder\u{2026}"),
];
const SORT: &[Item] = &[row("Name", Mark::Radio, true), row("Date modified", Mark::Radio, false), row("Size", Mark::Radio, false)];

/// A dropdown in a box `COL` wide and `h` tall, its line at the bottom when the list opens up, at the top when it
/// opens down. `open` presses the line once.
fn list(line: &'static str, items: &'static [Item], opts: DropdownOpts, open: bool, h: f32) -> Cell {
    let probe = Probe::new();
    let shown = Memo::new(false);
    let line_id: Memo<Option<Id>> = Memo::new(None);
    Cell::new(State::Rest, move |ui: &mut Ui, _| {
        // A fresh `Ui` (the harness builds a cell in several) starts closed.
        if line_id.get().and_then(|id| ui.rect(id)).is_none() {
            shown.set(false);
        }
        if let Some(r) = if open { probe.due(ui) } else { None } {
            click(ui, centre(r));
        }
        let style = Style { column: true, w: Dim::Pct(100.0), h: Dim::Px(h), justify: if opts.up { Justify::End } else { Justify::Start }, ..Style::default() };
        let mut id = None;
        ui.node("box", style, |ui| {
            let mut o = shown.get();
            let r = ui.dropdown("dd", line, &mut o, opts, |ui| {
                for (i, item) in items.iter().enumerate() {
                    match *item {
                        Item::Row(row) => {
                            ui.option(&format!("o{i}"), row);
                        }
                        Item::Separator => ui.dropdown_separator(),
                        Item::Action(text) => {
                            ui.dropdown_action(&format!("a{i}"), text);
                        }
                    }
                }
            });
            shown.set(o);
            id = Some(r.line.id);
        });
        let id = id.unwrap();
        probe.set(id);
        line_id.set(Some(id));
        id
    })
}

pub fn blocks() -> Vec<Block> {
    let down = DropdownOpts::default();
    let up = DropdownOpts { up: true, ..down };
    let disabled = DropdownOpts { disabled: true, ..down };
    vec![
        Block::matrix(vec![vec![list("Toppings: Cheese, Basil", TOPPINGS, down, false, 24.0), list("Size: Medium", SIZES, disabled, false, 24.0)]]).col_width(COL),
        Block::matrix(vec![vec![list("Toppings: Cheese, Basil", TOPPINGS, down, true, 140.0), list("Size: Medium", SIZES, down, true, 140.0)]]).col_width(COL),
        Block::matrix(vec![vec![list("Back up: Reports, Notes", BACKUP, down, true, 200.0), list("Save to: Documents", SAVE_TO, down, true, 200.0)]]).col_width(COL),
        Block::matrix(vec![vec![list("Sort by: Name", SORT, up, true, 112.0)]]).col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
