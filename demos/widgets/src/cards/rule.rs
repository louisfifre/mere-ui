//! The Rule card: the one rule atom, a 1 px line, in its two kinds. A section rule (the edge colour) opens a panel's
//! section, as under a fold's head; a separator (the line colour, fainter) parts things inside a section, as between
//! two groups of a menu. Each is shown with the space it is given around it, between two texts.

use mere_layout::{Edges, Style};
use mere_ui::tokens::{INSET, INSET_TIGHT};
use mere_ui::{Rule, Theme, Tone, Ui};

use super::selection_row::{Look, cell};
use super::{Block, Card, render};

const COL: f32 = 240.0;

pub fn blocks() -> Vec<Block> {
    let cells = [Rule::Section, Rule::Separator]
        .into_iter()
        .map(|kind| {
            cell(Look::Rest, move |ui: &mut Ui, _| {
                ui.node("box", Style { column: true, ..Style::default() }, |ui| {
                    let name = if matches!(kind, Rule::Section) { "Section rule" } else { "Separator" };
                    ui.label(name, Tone::Text);
                    ui.rule(kind, Edges::vh(INSET_TIGHT, INSET));
                    ui.label("What follows", Tone::Muted);
                })
            })
        })
        .collect();
    vec![Block::matrix(vec![cells]).col_width(COL)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
