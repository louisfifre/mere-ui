//! The Checkbox card: the square box with its label, unchecked and checked, in rest, hover, held and disabled. The
//! Design System has no preview of it yet: this card follows the boards' box (12 px, a 1 px frame, the check in the
//! text colour).

use mere_ui::{CheckboxOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

fn row(checked: bool) -> Vec<Cell> {
    [(State::Rest, false), (State::Hover, false), (State::Held, false), (State::Rest, true)]
        .into_iter()
        .map(|(state, disabled)| {
            Cell::new(state, move |ui: &mut Ui, _| {
                let mut v = checked;
                ui.checkbox("c", "Option", &mut v, CheckboxOpts { disabled }).id
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
