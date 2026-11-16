//! The ListHeads card: the legend head, then the part head plain, pressed, as a button and first
//! (components/ListHeads/preview.html).

use nui_layout::{Dim, Style};
use nui_ui::{PartOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// The preview's `col: '240px'`.
const COL: f32 = 240.0;

fn column(ui: &mut Ui, inner: impl FnOnce(&mut Ui) -> nui_ui::Id) -> nui_ui::Id {
    let mut id = None;
    ui.node("col", Style { column: true, w: Dim::Px(COL), ..Style::default() }, |ui| id = Some(inner(ui)));
    id.unwrap()
}

fn head() -> Option<Cell> {
    Some(Cell::new(State::Rest, |ui: &mut Ui, _| column(ui, |ui| ui.list_head("h", "Label", "Value", "Share")[0])))
}

fn part(opts: PartOpts, state: State) -> Option<Cell> {
    Some(Cell::new(state, move |ui: &mut Ui, _| column(ui, |ui| ui.list_part("p", "Section", "41.2%", opts).id)))
}

pub fn blocks() -> Vec<Block> {
    let plain = PartOpts::default();
    let button = PartOpts { button: true, ..plain };
    vec![
        Block::sparse(vec![
            vec![head(), None, None],
            vec![part(plain, State::Rest), None, part(PartOpts { pressed: true, ..plain }, State::Rest)],
            vec![part(button, State::Rest), part(button, State::Hover), part(PartOpts { pressed: true, ..button }, State::Rest)],
            vec![part(PartOpts { first: true, ..button }, State::Rest), None, None],
        ])
        .col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
