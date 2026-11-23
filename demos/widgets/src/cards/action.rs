//! The Button card: seven looks in rest, hover, held and disabled; a look without a state leaves its cell
//! empty (components/Button/preview.html).

use mere_ui::{ActionLook, ActionOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// The four columns of a row: rest, hover, held, disabled. `skip` names the columns the preview leaves empty.
fn row(word: &'static str, look: ActionLook, skip: &[usize]) -> Vec<Option<Cell>> {
    let states = [(State::Rest, false), (State::Hover, false), (State::Held, false), (State::Rest, true)];
    states
        .into_iter()
        .enumerate()
        .map(|(i, (state, disabled))| {
            (!skip.contains(&i)).then(|| {
                Cell::new(state, move |ui: &mut Ui, _| ui.action_with("b", word, look, ActionOpts { disabled, grow: 0.0 }).id)
            })
        })
        .collect()
}

pub fn blocks() -> Vec<Block> {
    vec![Block::sparse(vec![
        row("Action", ActionLook::Primary, &[]),
        row("Working\u{2026}", ActionLook::Running { share: 0.4 }, &[1, 2]),
        row("Option one", ActionLook::Plain, &[]),
        row("Action", ActionLook::Link, &[]),
        row("Action", ActionLook::Underlined, &[]),
        row("Item one", ActionLook::Pick, &[3]),
        row("none", ActionLook::PickNone, &[1, 2, 3]),
    ])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
