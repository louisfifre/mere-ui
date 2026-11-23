//! The StopButton card: the stop square in rest, hover, held and disabled
//! (components/StopButton/preview.html).

use mere_ui::{Theme, Ui};

use super::{Block, Card, Cell, State, render};

pub fn blocks() -> Vec<Block> {
    let cell = |state: State, disabled: bool| Cell::new(state, move |ui: &mut Ui, _| ui.stop_action("s", disabled).id);
    vec![Block::matrix(vec![vec![
        cell(State::Rest, false),
        cell(State::Hover, false),
        cell(State::Held, false),
        cell(State::Rest, true),
    ]])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
