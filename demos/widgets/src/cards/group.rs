//! The Group card: rows stacked with no frame and no fill, so that a group can be placed anywhere. Two groups one
//! after the other, parted by the separator the second one places itself; a titled group (a panel's settings) and
//! the titled group that follows it, parted by `gap-wide`; and a single selection row placed alone, which needs no
//! group. The Design System has no preview of it yet.

use mere_layout::Style;
use mere_ui::{ImmediateRow, SelectionRow, Selector, Theme, Ui};

use super::selection_row::{Look, cell};
use super::{Block, Card, render};

const COL: f32 = 240.0;

pub fn blocks() -> Vec<Block> {
    let two = cell(Look::Rest, |ui: &mut Ui, _| {
        let mut id = None;
        ui.node("box", Style { column: true, ..Style::default() }, |ui| {
            id = Some(ui.group("sizes", |ui| {
                for (i, label) in ["Small", "Medium", "Large"].into_iter().enumerate() {
                    ui.selection_row(&format!("s{i}"), SelectionRow { label, selector: Selector::Single, selected: i == 1, ..SelectionRow::default() });
                }
            }));
            ui.group("more", |ui| {
                ui.action_row("custom", "Custom size\u{2026}");
            });
        });
        id.unwrap()
    });
    let titled = cell(Look::Rest, |ui: &mut Ui, _| {
        let mut id = None;
        ui.node("box", Style { column: true, ..Style::default() }, |ui| {
            let view = ui.titled_group("view", "View", |_| {}, |ui| {
                ui.selection_row("grid", SelectionRow { label: "Show the grid", selected: true, ..SelectionRow::default() });
                ui.selection_row("names", SelectionRow { label: "Show the names", ..SelectionRow::default() });
            });
            id = Some(view.group);
            ui.titled_group("output", "Output", |_| {}, |ui| {
                for (i, label) in ["Headphones", "Speakers"].into_iter().enumerate() {
                    ui.immediate_row(&format!("o{i}"), ImmediateRow { label, selected: i == 0, ..ImmediateRow::default() });
                }
            });
        });
        id.unwrap()
    });
    let alone = cell(Look::Rest, |ui: &mut Ui, _| ui.selection_row("alone", SelectionRow { label: "Keep a copy", selected: true, ..SelectionRow::default() }).id);
    vec![Block::matrix(vec![vec![two, titled, alone]]).col_width(COL)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
