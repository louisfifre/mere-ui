//! The ToolButton card: plain and with a marker, rest, hover and pressed (components/ToolButton/preview.html).

use mere_ui::{Icon, IconOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

const HOME: &str = "M4 20h16v2H4zm16-10h2v10h-2zM2 10h2v10H2zm2-2h2v2H4zm2-2h2v2H6zm2-2h2v2H8zm2-2h4v2h-4zm4 2h2v2h-2zm2 2h2v2h-2zm2 2h2v2h-2zM8 14h2v6H8zm2-2h4v2h-4zm4 2h2v6h-2z";

fn cell(state: State, pressed: bool, marker: bool) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let marker = marker.then(|| ui.theme().warn);
        let icon = Icon::parse(HOME, 24.0).expect("the card's icon parses");
        ui.tool_button("t", &icon, "Label", IconOpts { pressed, marker, ..IconOpts::default() }).id
    })
}

fn row(marker: bool) -> Vec<Cell> {
    vec![cell(State::Rest, false, marker), cell(State::Hover, false, marker), cell(State::Rest, true, marker)]
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![row(false), row(true)])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
