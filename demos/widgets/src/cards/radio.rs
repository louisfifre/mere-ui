//! The Radio card: the round button with its label, unselected and selected, in rest, hover, held and disabled, then
//! a group of three with the middle one selected. The Design System has no preview of it yet: the ring has the
//! checkbox's size and the dot its check's colour.

use mere_layout::Style;
use mere_ui::{Selector, SelectorOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

fn row(selected: bool) -> Vec<Cell> {
    [(State::Rest, false), (State::Hover, false), (State::Held, false), (State::Rest, true)]
        .into_iter()
        .map(|(state, disabled)| Cell::new(state, move |ui: &mut Ui, _| ui.selector("r", "Option", Selector::Single, selected, SelectorOpts { disabled }).id))
        .collect()
}

/// Three sizes in a column, Medium selected.
fn group() -> Cell {
    Cell::new(State::Rest, |ui: &mut Ui, _| {
        let mut first = None;
        ui.node("group", Style { column: true, ..Style::default() }, |ui| {
            for (i, size) in ["Small", "Medium", "Large"].into_iter().enumerate() {
                let id = ui.selector(&format!("s{i}"), size, Selector::Single, i == 1, SelectorOpts::default()).id;
                first.get_or_insert(id);
            }
        });
        first.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![row(false), row(true)]), Block::single(group())]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
