//! The Help card: a row whose label has a mark, at rest, with the pointer on the row, and with its help open over it.
//! The Design System has no preview of it yet: the card is the kit's own picture.

use mere_layout::{Align, Dim, Position, Style};
use mere_ui::tokens::{CONTROL, GAP_TIGHT};
use mere_ui::{HELP_MARK_W, HelpText, TextLook, Theme, Tone, Ui, help_anchor, help_label};

use super::{Block, Card, Cell, State, render};

const W: f32 = 280.0;
/// The room a cell keeps under its row for the open help, which takes none of its own.
const ROOM: f32 = 112.0;
const LABEL: &str = "Temperature scale";
const HINT: &str = "How wide the warm and cold regions are. Lower: they change more often. Higher: they stretch further.";
const FACTS: &str = "Default 1, from 0.1 to 4.";

fn cell(state: State, open: bool) -> Cell {
    Cell::new(state, move |ui: &mut Ui, _| {
        let mut row = None;
        ui.node("room", Style { column: true, w: Dim::Px(W), h: Dim::Px(if open { ROOM } else { CONTROL }), ..Style::default() }, |ui| {
            ui.node("place", help_anchor(), |ui| {
                let style = Style { h: Dim::Px(CONTROL), align_items: Align::Center, ..Style::default() };
                let id = ui.hover_area("row", style, |ui, over| {
                    let (label, at) = help_label(LABEL, W - GAP_TIGHT - HELP_MARK_W);
                    ui.node("name", Style { position: Position::Relative, ..Style::default() }, |ui| {
                        ui.node("label", label, |ui| ui.paragraph(LABEL, TextLook::body(Tone::Text)));
                        ui.help_mark("?", at, over.is_some(), open);
                    });
                });
                row = Some(id);
                if open {
                    ui.help_panel("help", id, HelpText { title: LABEL, hint: HINT, facts: FACTS });
                }
            });
        });
        row.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    vec![
        Block::matrix(vec![vec![cell(State::Rest, false), cell(State::Hover, false)]]).col_width(W),
        Block::single(cell(State::Rest, true)).col_width(W),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
