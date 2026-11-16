//! The IconButton card: the looks as rows, rest, hover, pressed and disabled as columns; a look without a
//! state leaves its cell empty (components/IconButton/preview.html).

use nui_ui::{Icon, IconLook, IconOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

const SETTINGS: &str = "M8 14H7v6H5v-6H2v-2h6v2Zm5 6h-2V10h2v10Zm9-2h-3v2h-2v-2h-1v-2h6v2Zm-3-4h-2V4h2v10ZM7 10H5V4h2v6Zm6-4h2v2H9V6h2V4h2v2Z";
const PLAY: &str = "M15 11h-2V9h2zm0 4h-2v-2h2zm-2 2h-2v-2h2zm0-8h-2V7h2zm-2-2H9V5h2zM9 21H7V3h2zm6-8h2v-2h-2zm-6 4h2v2H9z";
const PANEL_LEFT: &str = "M3 5h18v2H3zm0 12h18v2H3zM3 7h2v10H3zm16 0h2v10h-2zM8 7h2v10H8z";
const PANEL_LEFT_SHOWN: &str = "M3 5h18v2H3zm0 12h18v2H3zM3 7h2v10H3zm16 0h2v10h-2zM5 7h5v10H5z";
const MINIMISE: &str = "M5 11h14v2H5z";
const CLOSE: &str = "M5 5h2v2H5zm2 2h2v2H7zm2 2h2v2H9zm2 2h2v2h-2zm2-2h2v2h-2zm2-2h2v2h-2zm2-2h2v2h-2zM9 13h2v2H9zm-2 2h2v2H7zm-2 2h2v2H5zm8-4h2v2h-2zm2 2h2v2h-2zm2 2h2v2h-2z";

fn cell(path: &'static str, look: IconLook, state: State, pressed: bool, disabled: bool, marker: bool) -> Option<Cell> {
    Some(Cell::new(state, move |ui: &mut Ui, _| {
        let marker = marker.then(|| ui.theme().warn);
        let icon = Icon::parse(path, 24.0).expect("the card's icon parses");
        ui.icon_button("b", &icon, look, IconOpts { pressed, disabled, joined: false, marker }).id
    }))
}

/// One row: rest, hover, then pressed when the look has it, then disabled.
fn row(path: &'static str, look: IconLook, pressed: bool, marker: bool) -> Vec<Option<Cell>> {
    vec![
        cell(path, look, State::Rest, false, false, marker),
        cell(path, look, State::Hover, false, false, marker),
        if pressed { cell(path, look, State::Rest, true, false, marker) } else { None },
        cell(path, look, State::Rest, false, true, marker),
    ]
}

pub fn blocks() -> Vec<Block> {
    vec![Block::sparse(vec![
        row(SETTINGS, IconLook::Tool, true, false),
        row(SETTINGS, IconLook::Tool, true, true),
        row(SETTINGS, IconLook::Square, true, false),
        row(PLAY, IconLook::Filled, false, false),
        row(PANEL_LEFT, IconLook::Bare, false, false),
        row(MINIMISE, IconLook::Caption, false, false),
        row(PANEL_LEFT, IconLook::Caption, false, false),
        row(PANEL_LEFT_SHOWN, IconLook::Caption, false, false),
        row(CLOSE, IconLook::Close, false, false),
    ])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
