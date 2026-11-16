//! The Toggle card: the square switch with its word, on and off, default and wide, in rest, hover, held and
//! disabled (components/Toggle/preview.html).

use nui_layout::{Dim, Edges};
use nui_ui::{Theme, ToggleOpts, Ui};

use super::{Block, Card, Cell, State, render};

/// The preview's `minW: '8ch', pad: [5, 9]`: the launcher's `min-width: 8ch` is 57.6 px at its body size. The
/// switch stays at the box's left edge, inside the padding.
fn wide() -> ToggleOpts {
    ToggleOpts { min_w: Dim::Px(57.6), pad: Edges::vh(5.0, 9.0), disabled: false }
}

fn row(on: bool, base: ToggleOpts) -> Vec<Cell> {
    [(State::Rest, false), (State::Hover, false), (State::Held, false), (State::Rest, true)]
        .into_iter()
        .map(|(state, disabled)| {
            Cell::new(state, move |ui: &mut Ui, _| {
                let mut v = on;
                ui.toggle_with("t", &mut v, "on", "off", ToggleOpts { disabled, ..base }).id
            })
        })
        .collect()
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![
        row(true, ToggleOpts::default()),
        row(false, ToggleOpts::default()),
        row(true, wide()),
        row(false, wide()),
    ])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
