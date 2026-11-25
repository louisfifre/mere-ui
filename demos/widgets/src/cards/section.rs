//! The Section card: a section whose head carries actions, which does not fold, beside a fold, in a 360 px column,
//! so that the two heads can be compared (the launcher's Home and profile page).

use mere_layout::{Dim, Style};
use mere_ui::{ActionLook, Theme, Tone, Ui};

use super::{Block, Card, Cell, State, render};

const COL: f32 = 360.0;

fn cell(actions: usize, state: State) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let mut id = None;
        ui.node("wrap", Style { w: Dim::Px(COL), column: true, ..Style::default() }, |ui| {
            id = Some(ui.section(
                "s",
                "Mods \u{b7} 14",
                |ui| {
                    if actions == 2 {
                        ui.action("all", "Update all (3)", ActionLook::Underlined);
                    }
                    ui.action("add", "Add a mod", ActionLook::Underlined);
                },
                |ui| ui.label("Content", Tone::Text),
            ));
        });
        id.unwrap()
    })
}

fn folded(state: State) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let mut id = None;
        ui.node("wrap", Style { w: Dim::Px(COL), column: true, ..Style::default() }, |ui| {
            let mut open = true;
            id = Some(ui.fold("f", "Game", &mut open, |ui| ui.label("Content", Tone::Text)).id);
        });
        id.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![vec![cell(1, State::Rest)], vec![cell(2, State::Rest)], vec![folded(State::Rest)]]).col_width(COL)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
