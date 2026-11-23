//! The Menu card: the line closed and disabled; then open in each kind of choice, several with multiple selectors and
//! one with single selectors (one row disabled); then the same two with two-line rows whose long detail is cut at its start,
//! then a second group (an action row) parted by its separator; and a list hung above its line. The Design System has no preview of it yet. An open
//! list is opened by a press on its line, so its first row has the focus, as it does when the list opens.

use std::cell::Cell as Memo;

use mere_layout::{Dim, Justify, Style};
use mere_ui::{Id, MenuOpts, SelectionRow, Selector, Theme, Trigger, Ui};

use super::select::{Probe, centre, click};
use super::{Block, Card, Cell, State, render};

/// A panel's width.
const COL: f32 = 280.0;

/// What a list holds, in order: rows, and the start of a new group.
#[derive(Copy, Clone)]
enum Item {
    Row(SelectionRow<'static>),
    Group,
    Action(&'static str),
}

const fn row(label: &'static str, selector: Selector, selected: bool) -> Item {
    Item::Row(SelectionRow { label, detail: "", detail_tail: "", selector, selected, disabled: false })
}

const fn off(label: &'static str, selector: Selector) -> Item {
    Item::Row(SelectionRow { label, detail: "", detail_tail: "", selector, selected: false, disabled: true })
}

const fn two(label: &'static str, detail: &'static str, detail_tail: &'static str, selector: Selector, selected: bool) -> Item {
    Item::Row(SelectionRow { label, detail, detail_tail, selector, selected, disabled: false })
}

const TOPPINGS: &[Item] = &[row("Cheese", Selector::Multiple, true), row("Tomato", Selector::Multiple, false), row("Basil", Selector::Multiple, true), row("Olives", Selector::Multiple, false)];
const SIZES: &[Item] = &[row("Small", Selector::Single, false), row("Medium", Selector::Single, true), row("Large", Selector::Single, false), off("Extra large", Selector::Single)];
const BACKUP: &[Item] = &[
    two("Reports", "Archive", r"C:\Code\archive\2024\reports\quarterly\final versions", Selector::Multiple, true),
    two("Sketches", "Code", r"C:\Code\sketches\2024\interface studies", Selector::Multiple, false),
    two("Notes", "", "", Selector::Multiple, true),
    Item::Group,
    Item::Action("Browse\u{2026}"),
];
const SAVE_TO: &[Item] = &[
    two("Documents", "This computer", r"C:\Users\someone\Documents\Letters\2024\drafts", Selector::Single, true),
    two("Projects", "This computer", r"C:\Code\projects", Selector::Single, false),
    two("Archive", "This computer", r"C:\Code\archive\2024", Selector::Single, false),
    Item::Group,
    Item::Action("Choose a folder\u{2026}"),
];
const SORT: &[Item] = &[row("Name", Selector::Single, true), row("Date modified", Selector::Single, false), row("Size", Selector::Single, false)];

/// A menu in a box `COL` wide and `h` tall, its line at the bottom when the list opens up, at the top when it
/// opens down. `open` presses the line once.
fn list(line: &'static str, items: &'static [Item], opts: MenuOpts, open: bool, h: f32) -> Cell {
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
            let r = ui.menu("menu", Trigger::Line(line), &mut o, opts, |ui| {
                // Each group is the rows up to the next `Item::Group`; the groups place their separators.
                for (g, group) in items.split(|i| matches!(i, Item::Group)).enumerate() {
                    ui.group(&format!("g{g}"), |ui| {
                        for (i, item) in group.iter().enumerate() {
                            match *item {
                                Item::Row(row) => {
                                    ui.selection_row(&format!("o{i}"), row);
                                }
                                Item::Action(text) => {
                                    ui.action_row(&format!("a{i}"), text);
                                }
                                Item::Group => {}
                            }
                        }
                    });
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
    let down = MenuOpts::default();
    let up = MenuOpts { up: true, ..down };
    let disabled = MenuOpts { disabled: true, ..down };
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
