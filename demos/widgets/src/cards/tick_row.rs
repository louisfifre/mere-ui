//! The Tick row card: one line and two lines, not ticked and ticked, under the pointer, focused and disabled, and a
//! long name cut at its end over a long folder cut at its start. The Design System has no preview of it yet: this
//! card follows the rows of the Apply boards.

use mere_layout::{Dim, Style};
use mere_ui::{Theme, TickRow, Ui};

use super::select::{Probe, centre, click};
use super::{Block, Card, Cell, State, render};

/// The settings panel's width on the boards.
const COL: f32 = 319.0;

const NAME: &str = "Exploration, your game";
const HOW: &str = "Prism Launcher";
const PATH: &str = r"%APPDATA%\PrismLauncher\instances\Exploration";

/// A row in a box `COL` wide. `focus` presses it once, so that it takes the focus; the value shown is `ticked`
/// whatever that press did, each build starting from it.
fn row(state: State, row: TickRow<'static>, ticked: bool, focus: bool) -> Cell {
    let probe = Probe::new();
    Cell::new(state, move |ui: &mut Ui, _| {
        if let Some(r) = if focus { probe.due(ui) } else { None } {
            click(ui, centre(r));
        }
        let mut v = ticked;
        let mut id = None;
        // A column, so that the row takes the box's whole width as it does in a list.
        ui.node("col", Style { column: true, w: Dim::Pct(100.0), ..Style::default() }, |ui| id = Some(ui.tick_row("r", row, &mut v).id));
        let id = id.unwrap();
        probe.set(id);
        id
    })
}

pub fn blocks() -> Vec<Block> {
    let one = TickRow { name: "Atelier, your game", ..TickRow::default() };
    let two = TickRow { name: NAME, how: HOW, path: PATH, disabled: false };
    let off = TickRow { disabled: true, ..two };
    let long = TickRow {
        name: "Exploration with the long winter settings, your game",
        how: "Minecraft Launcher",
        path: r"C:\Users\someone\AppData\Roaming\.minecraft-exploration",
        disabled: false,
    };
    vec![
        Block::matrix(vec![
            vec![row(State::Rest, one, false, false), row(State::Rest, one, true, false)],
            vec![row(State::Rest, two, false, false), row(State::Rest, two, true, false)],
            vec![row(State::Hover, two, false, false), row(State::Rest, two, false, true)],
            vec![row(State::Rest, off, false, false), row(State::Rest, off, true, false)],
            vec![row(State::Rest, long, true, false), row(State::Rest, long, false, true)],
        ])
        .col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
