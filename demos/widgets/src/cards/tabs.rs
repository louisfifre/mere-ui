//! The Tabs card: the strip and the choice in rest, hover, held, pressed and disabled, then a strip of two
//! groups (components/Tabs/preview.html).

use nui_ui::{TabLook, TabOpts, Theme, Ui, choice_style, strip_style};

use super::{Block, Card, Cell, State, render};

const WORD: &str = "Label";

/// `N.tabs([o], look)`: one tab alone in its strip or choice.
fn one(look: TabLook, state: State, pressed: bool, disabled: bool) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let style = if look == TabLook::Strip { strip_style() } else { choice_style() };
        let mut id = None;
        ui.node("row", style, |ui| id = Some(ui.tab("t", WORD, look, TabOpts { pressed, disabled }).id));
        id.unwrap()
    })
}

fn row(look: TabLook) -> Vec<Cell> {
    vec![
        one(look, State::Rest, false, false),
        one(look, State::Hover, false, false),
        one(look, State::Held, false, false),
        one(look, State::Rest, true, false),
        one(look, State::Rest, false, true),
    ]
}

/// Two groups, set apart by the strip's wide gap: `[Label x3] [Label x2]`, the second tab of the first and
/// the first of the second pressed.
fn two_groups() -> Cell {
    Cell::new(State::Rest, |ui: &mut Ui, _| {
        ui.node("row", strip_style(), |ui| {
            for (g, n) in [3usize, 2].into_iter().enumerate() {
                if g > 0 {
                    ui.strip_gap();
                }
                for i in 0..n {
                    let pressed = i == if g == 0 { 1 } else { 0 };
                    ui.tab(&format!("t{g}{i}"), WORD, TabLook::Strip, TabOpts { pressed, disabled: false });
                }
            }
        })
    })
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![row(TabLook::Strip), row(TabLook::Choice)]), Block::single(two_groups())]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
