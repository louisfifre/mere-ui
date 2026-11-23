//! The Immediate Row card: a choice that applies as soon as it is pressed (the select's options, the sound output of a
//! system flyout), with no selector and nothing to validate after. The whole row is the target, which suits touch;
//! the selected row is lit as under the pointer and marked by the indicator on its side. Shown in each state,
//! unselected then selected, without then with a detail line; then a list of three, the middle one selected. The
//! Design System has no preview of it yet.

use mere_layout::Style;
use mere_ui::{ImmediateRow, Theme, Ui};

use super::selection_row::{LOOKS, Look, cell};
use super::{Block, Card, Cell, render};

const COL: f32 = 240.0;

fn line(selected: bool, detail: bool) -> Vec<Cell> {
    LOOKS
        .iter()
        .map(|&look| {
            cell(look, move |ui, disabled| {
                let (detail, detail_tail) = if detail { ("USB", "Speakers (High Definition Audio)") } else { ("", "") };
                ui.immediate_row("r", ImmediateRow { label: "Speakers", detail, detail_tail, selected, disabled }).id
            })
        })
        .collect()
}

fn list() -> Cell {
    cell(Look::Rest, |ui: &mut Ui, _| {
        let mut first = None;
        ui.node("list", Style { column: true, ..Style::default() }, |ui| {
            for (i, label) in ["Headphones", "Speakers", "Monitor"].into_iter().enumerate() {
                let id = ui.immediate_row(&format!("r{i}"), ImmediateRow { label, selected: i == 1, ..ImmediateRow::default() }).id;
                first.get_or_insert(id);
            }
        });
        first.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    let rows = vec![line(false, false), line(true, false), line(false, true), line(true, true)];
    vec![Block::matrix(rows).col_width(COL), Block::single(list()).col_width(COL)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
