//! The Value Row card: a panel's setting, a label with its help mark, an optional note and the control at the right,
//! the controls of every row in one column. First the row alone in each state: at rest, under the pointer (the mark
//! shows), with a note (the pack's value beside a changed field), with its help open; then a label too long for its
//! room, which wraps and keeps its mark after its last word; then a titled group of value rows, a panel's group of
//! settings. The Design System has no preview of it yet.

use mere_layout::Style;
use mere_ui::{HelpText, NumberSpec, Theme, Ui, ValueRow};

use super::selection_row::{Look, cell};
use super::{Block, Card, Cell, render};

/// A panel's width.
const COL: f32 = 280.0;
/// The control column: a number field's width.
const CONTROL: f32 = 64.0;

const SPEC: NumberSpec = NumberSpec { min: 0.0, max: 100.0, step: 1.0 };

fn row(label: &'static str, note: &'static str) -> ValueRow<'static> {
    ValueRow { label, note, mark: "?", width: COL, control_w: CONTROL, gutter: 0.0 }
}

fn one(look: Look, label: &'static str, note: &'static str, help: bool) -> Cell {
    cell(look, move |ui: &mut Ui, _| {
        let mut v = 42.0;
        let text = HelpText { title: label, hint: "How far the hills rise above the plains.", facts: "Default 42 \u{b7} from 0 to 100" };
        ui.value_row("height", row(label, note), help.then_some(text), |ui| {
            ui.number("field", &mut v, SPEC);
        })
        .row
    })
}

fn group() -> Cell {
    cell(Look::Rest, |ui: &mut Ui, _| {
        let mut id = None;
        ui.node("panel", Style { column: true, ..Style::default() }, |ui| {
            id = Some(
                ui.titled_group("terrain", "Terrain", |_| {}, |ui| {
                    for (i, (label, note)) in [("Hill height", ""), ("Sea level", "63"), ("River width", "")].into_iter().enumerate() {
                        let mut v = 20.0 + i as f32 as f64 * 10.0;
                        ui.value_row(&format!("v{i}"), row(label, note), None, |ui| {
                            ui.number("field", &mut v, SPEC);
                        });
                    }
                })
                .group,
            );
            ui.titled_group("caves", "Caves", |_| {}, |ui| {
                let mut on = true;
                ui.value_row("caves", row("Caves", ""), None, |ui| {
                    ui.toggle("field", &mut on, "on", "off");
                });
            });
        });
        id.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    let states = vec![one(Look::Rest, "Hill height", "", false), one(Look::Hover, "Hill height", "", false), one(Look::Rest, "Hill height", "63", false)];
    let long = one(Look::Hover, "Height of the hills above the plains, in blocks", "", false);
    vec![
        Block::matrix(vec![states]).col_width(COL),
        Block::matrix(vec![vec![long, one(Look::Rest, "Hill height", "", true)]]).col_width(COL),
        Block::single(group()).col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
