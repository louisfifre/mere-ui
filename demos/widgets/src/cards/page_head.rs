//! The PageHead card: a page's title line, with nothing at its end, with an action, and with a title too long for
//! its room, at rest and with the pointer on the back action (the launcher's pages).

use mere_layout::{Dim, Style};
use mere_ui::{ActionLook, Icon, Theme, Ui};

use super::{Block, Card, Cell, State, render};

const BACK: &str = "M20 11v2H4v-2zM8 13v2H6v-2zm2 2v2H8v-2zm2 2v2h-2v-2zm-4-6V9H6v2zM10 15V7H8v8zm2 2V5h-2v12z";
const COL: f32 = 360.0;

fn cell(title: &'static str, action: bool, state: State) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let back = Icon::parse(BACK, 24.0).expect("the card's icon parses");
        let mut id = None;
        ui.node("wrap", Style { w: Dim::Px(COL), column: true, ..Style::default() }, |ui| {
            id = Some(
                ui.page_head("h", &back, title, |ui| {
                    if action {
                        ui.action("go", "Play", ActionLook::Primary);
                    }
                })
                .back
                .id,
            );
        });
        id.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![
        vec![cell("Exploration", false, State::Rest), cell("Exploration", false, State::Hover)],
        vec![cell("Exploration", true, State::Rest), cell("Exploration", true, State::Hover)],
        vec![cell("Add a mod to a profile with a very long name", true, State::Rest)],
    ])
    .col_width(COL)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
