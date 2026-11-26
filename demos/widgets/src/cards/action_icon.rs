//! The ActionIcon card: an action with an icon before its word, in the primary and the plain look, at rest, hovered,
//! held and disabled (the page title's "Play" and "Start"). No Design System preview yet: it stands alone.

use mere_ui::{ActionLook, ActionOpts, Icon, Theme, Ui};

use super::{Block, Card, Cell, State, render};

const PLAY: &str = "M15 11h-2V9h2zm0 4h-2v-2h2zm-2 2h-2v-2h2zm0-8h-2V7h2zm-2-2H9V5h2zM9 21H7V3h2zm6-8h2v-2h-2zm-6 4h2v2H9z";

fn row(word: &'static str, look: ActionLook) -> Vec<Cell> {
    [(State::Rest, false), (State::Hover, false), (State::Held, false), (State::Rest, true)]
        .into_iter()
        .map(|(state, disabled)| {
            Cell::new(state, move |ui: &mut Ui, _| {
                let icon = Icon::parse(PLAY, 24.0).expect("the card's icon parses");
                ui.action_icon("b", &icon, word, look, ActionOpts { disabled, grow: 0.0 }).id
            })
        })
        .collect()
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![row("Play", ActionLook::Primary), row("Start", ActionLook::Plain)])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
