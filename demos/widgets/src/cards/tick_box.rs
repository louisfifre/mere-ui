//! The Tick box card: the square box with its label, not ticked and ticked, in rest, hover, held and disabled. The
//! Design System has no preview of it yet: this card follows the Apply boards' box (12 px, a 1 px frame, the check
//! in the text colour).

use mere_ui::{Theme, TickOpts, Ui};

use super::{Block, Card, Cell, State, render};

fn row(ticked: bool) -> Vec<Cell> {
    [(State::Rest, false), (State::Hover, false), (State::Held, false), (State::Rest, true)]
        .into_iter()
        .map(|(state, disabled)| {
            Cell::new(state, move |ui: &mut Ui, _| {
                let mut v = ticked;
                ui.tick_box("t", "Option", &mut v, TickOpts { disabled }).id
            })
        })
        .collect()
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![row(false), row(true)])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
