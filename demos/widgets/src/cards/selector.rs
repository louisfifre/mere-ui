//! The Selector card. First the selector alone, the atom, which takes no press and only shows the state it is given:
//! the multiple selector (square) then the single selector (round), unselected and selected, at rest, lit (under the
//! pointer), on the selection fill of a focused row, and disabled. Then the labelled selector, which takes the press:
//! both kinds, unselected and selected, in rest, hover, held and disabled; and a group of three single selectors with
//! the middle one selected, since a single selector is always one among others. The Design System has no preview of
//! it yet: this card follows the boards' box (12 px, a 1 px frame, the check in the text colour), the ring having its
//! size and the dot the check's colour.

use mere_layout::{Dim, Insets, Position, Style};
use mere_ui::tokens::SPACE_4;
use mere_ui::{MARK_SIZE, MarkOpts, Selector, SelectorOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// The selector alone in each look it can be given.
fn marks(kind: Selector, selected: bool) -> Vec<Cell> {
    let looks = [
        MarkOpts::default(),
        MarkOpts { lit: true, ..MarkOpts::default() },
        MarkOpts { on_selection: true, ..MarkOpts::default() },
        MarkOpts { disabled: true, ..MarkOpts::default() },
    ];
    looks
        .into_iter()
        .map(|look| {
            Cell::new(State::Rest, move |ui: &mut Ui, _| {
                if !look.on_selection {
                    return ui.node("atom", Style::default(), |ui| ui.selector_mark(kind, selected, look));
                }
                // The selection fill a focused row gives it, `SPACE_4` around the selector.
                let fill = ui.theme().select;
                let side = Dim::Px(MARK_SIZE + 2.0 * SPACE_4);
                ui.node("atom", Style { position: Position::Relative, ..Style::default() }, |ui| {
                    ui.fill(Style { w: side, h: side, ..Style::default() }, fill);
                    // Out of the flow, so that it is painted over the fill.
                    let inset = Insets { left: Dim::Px(SPACE_4), top: Dim::Px(SPACE_4), right: Dim::Auto, bottom: Dim::Auto };
                    ui.node("over", Style { position: Position::Absolute, inset, ..Style::default() }, |ui| ui.selector_mark(kind, selected, look));
                })
            })
        })
        .collect()
}

fn labelled(kind: Selector, selected: bool) -> Vec<Cell> {
    [(State::Rest, false), (State::Hover, false), (State::Held, false), (State::Rest, true)]
        .into_iter()
        .map(|(state, disabled)| Cell::new(state, move |ui: &mut Ui, _| ui.selector("s", "Option", kind, selected, SelectorOpts { disabled }).id))
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
    let (multiple, single) = (Selector::Multiple, Selector::Single);
    vec![
        Block::matrix(vec![marks(multiple, false), marks(multiple, true), marks(single, false), marks(single, true)]),
        Block::matrix(vec![labelled(multiple, false), labelled(multiple, true), labelled(single, false), labelled(single, true)]),
        Block::single(group()),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
