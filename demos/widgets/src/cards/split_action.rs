//! The split action card: the primary button in two parts, the word and the chevron that opens a list, in rest, hover
//! (over each part), held, a word that cannot act, both parts disabled and the running look. The Design System has no
//! preview of it yet.

use mere_layout::{Dim, Style};
use mere_ui::{ActionLook, SplitOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// A split action 200 px wide. `on_chevron` says which part the cell's pointer is on.
fn cell(state: State, look: ActionLook, opts: SplitOpts, on_chevron: bool) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let mut id = None;
        ui.node("w", Style { w: Dim::Px(200.0), ..Style::default() }, |ui| {
            let mut open = false;
            let r = ui.split_action("g", "Generate preview", look, &mut open, opts, |_| {});
            id = Some(if on_chevron { r.line.id } else { r.word.expect("a split has its word").id });
        });
        id.expect("built")
    })
}

pub fn blocks() -> Vec<Block> {
    let on = SplitOpts::default();
    let primary = ActionLook::Primary;
    vec![Block::matrix(vec![
        vec![cell(State::Rest, primary, on, false), cell(State::Hover, primary, on, false), cell(State::Hover, primary, on, true)],
        vec![cell(State::Held, primary, on, false), cell(State::Held, primary, on, true), cell(State::Rest, primary, SplitOpts { word_disabled: true, ..on }, false)],
        vec![cell(State::Rest, primary, SplitOpts { word_disabled: true, chevron_disabled: true, ..on }, false), cell(State::Rest, ActionLook::Running { share: 0.4 }, on, false)],
    ])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
