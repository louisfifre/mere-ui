//! The Action Row card: an action in a list ("Choose a folder…"), no selector, its text in the text column so that it
//! lines up with the labels of the rows around it; underlined under the pointer, filled when focused. Shown alone in
//! each state, then under two selection rows to show the line-up. The Design System has no preview of it yet.

use mere_layout::Style;
use mere_ui::{SelectionRow, Selector, Theme, Ui};

use super::selection_row::{LOOKS, Look, cell};
use super::{Block, Card, render};

const COL: f32 = 240.0;

pub fn blocks() -> Vec<Block> {
    // An action row has no disabled look: an action that cannot run is left out of its list.
    let looks: Vec<_> = LOOKS.iter().copied().filter(|l| !matches!(l, Look::Disabled)).map(|look| cell(look, |ui, _| ui.action_row("a", "Choose a folder\u{2026}").id)).collect();
    let together = cell(Look::Rest, |ui: &mut Ui, _| {
        let mut id = None;
        ui.node("list", Style { column: true, ..Style::default() }, |ui| {
            ui.selection_row("o0", SelectionRow { label: "Documents", selector: Selector::Single, selected: true, ..SelectionRow::default() });
            ui.selection_row("o1", SelectionRow { label: "Projects", selector: Selector::Single, ..SelectionRow::default() });
            id = Some(ui.action_row("a", "Choose a folder\u{2026}").id);
        });
        id.unwrap()
    });
    vec![Block::matrix(vec![looks]).col_width(COL), Block::single(together).col_width(COL)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
