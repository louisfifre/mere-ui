//! The HoverBox card: one word in a box, rest and hover, in columns 200 px wide
//! (components/HoverBox/preview.html).

use mere_layout::{Align, Dim, Edges, Style};
use mere_ui::tokens::{FADE_TINT, INSET};
use mere_ui::{TextLook, Theme, Tone, Ui, mix};

use super::{Block, Card, Cell, State, render};

fn cell(state: State) -> Cell {
    Cell::new(state, |ui: &mut Ui, _| {
        let t = *ui.theme();
        let style = Style { w: Dim::Pct(100.0), align_items: Align::Center, pad: Edges::vh(INSET, INSET), ..Style::default() };
        ui.hover_box("h", style, mix(t.text, FADE_TINT, t.ground), |ui| ui.text("Label", TextLook::body(Tone::Text)))
    })
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![vec![cell(State::Rest), cell(State::Hover)]]).col_width(200.0)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
