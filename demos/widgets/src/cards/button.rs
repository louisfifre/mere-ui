//! The Button card: seven looks in rest, hover, held and disabled; a look without a state leaves its cell
//! empty (components/Button/preview.html).

use mere_ui::{ButtonLook, ButtonOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// The four columns of a row: rest, hover, held, disabled. `skip` names the columns the preview leaves empty.
fn row(word: &'static str, look: ButtonLook, skip: &[usize]) -> Vec<Option<Cell>> {
    let states = [(State::Rest, false), (State::Hover, false), (State::Held, false), (State::Rest, true)];
    states
        .into_iter()
        .enumerate()
        .map(|(i, (state, disabled))| {
            (!skip.contains(&i)).then(|| {
                Cell::new(state, move |ui: &mut Ui, _| ui.button_with("b", word, look, ButtonOpts { disabled, grow: 0.0 }).id)
            })
        })
        .collect()
}

pub fn blocks() -> Vec<Block> {
    vec![Block::sparse(vec![
        row("Action", ButtonLook::Primary, &[]),
        row("Working\u{2026}", ButtonLook::Running { share: 0.4 }, &[1, 2]),
        row("Option one", ButtonLook::Plain, &[]),
        row("Action", ButtonLook::Link, &[]),
        row("Action", ButtonLook::Underlined, &[]),
        row("Item one", ButtonLook::Pick, &[3]),
        row("none", ButtonLook::PickNone, &[1, 2, 3]),
    ])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
