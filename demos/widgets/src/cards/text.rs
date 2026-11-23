//! The Text card: one line per look, in the look's own tone (components/Text/preview.html), and the small look in the
//! failure tone, which a status sentence under a control takes (`ui.paragraph(s, TextLook::small(Tone::Warn))`).

use mere_layout::Style;
use mere_ui::{TextLook, Theme, Tone, Ui};

use super::{Block, Card, Cell, State, render};

const SENTENCE: &str = "The quick brown fox jumps over the lazy dog";
const CAPS: &str = "THE QUICK BROWN FOX";

fn line(s: &'static str, look: TextLook) -> Vec<Cell> {
    vec![Cell::new(State::Rest, move |ui: &mut Ui, _| ui.node("line", Style::default(), |ui| ui.text(s, look)))]
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![
        line(SENTENCE, TextLook::body(Tone::Text)),
        line(CAPS, TextLook::heading()),
        line(SENTENCE, TextLook::status(Tone::Muted)),
        line(SENTENCE, TextLook::small(Tone::Muted)),
        line(SENTENCE, TextLook::small(Tone::Warn)),
        line(CAPS, TextLook::label()),
    ])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
