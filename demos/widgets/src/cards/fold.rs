//! The Fold card: closed and open, at rest and with the pointer on the head, in a 280 px column
//! (components/Fold/preview.html).

use nui_layout::{Dim, Style};
use nui_ui::{Theme, Tone, Ui};

use super::{Block, Card, Cell, State, render};

fn cell(open: bool, state: State) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let mut id = None;
        ui.node("wrap", Style { w: Dim::Px(280.0), column: true, ..Style::default() }, |ui| {
            let mut o = open;
            id = Some(ui.fold("f", "Section", &mut o, |ui| ui.label("Content", Tone::Text)).id);
        });
        id.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![
        vec![cell(false, State::Rest), cell(false, State::Hover)],
        vec![cell(true, State::Rest), cell(true, State::Hover)],
    ])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
