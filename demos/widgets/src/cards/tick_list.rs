//! The Tick list card: the line at rest and disabled, then the list open above its line (one place, two places, the
//! maintainer's four in two groups) and open under it. The Design System has no preview of it yet: this card follows
//! the Apply boards (2, 5 and 9), with their example places. An open list is opened by a press on its line, so its
//! first row has the focus, as it does when the list opens.

use std::cell::Cell as Memo;

use mere_layout::{Dim, Justify, Style};
use mere_ui::{Id, Theme, TickListOpts, TickRow, Ui};

use super::select::{Probe, centre, click};
use super::{Block, Card, Cell, State, render};

/// The settings panel's width on the boards.
const COL: f32 = 319.0;

/// One place: its row, whether it is ticked, and whether a rule goes above it.
type Place = (TickRow<'static>, bool, bool);

const PRISM: TickRow = TickRow { name: "Exploration, your game", how: "Prism Launcher", path: r"%APPDATA%\PrismLauncher\instances\Exploration", disabled: false };
const VANILLA: TickRow = TickRow { name: "Exploration, your game", how: "Minecraft Launcher", path: r"%APPDATA%\.minecraft-exploration", disabled: false };
const ATELIER: TickRow = TickRow { name: "Atelier, your game", how: "Prism Launcher", path: r"%APPDATA%\PrismLauncher\instances\Atelier", disabled: false };
const SERVER: TickRow = TickRow { name: "Test server, shared with everyone who joins", how: "Dedicated server", path: r"C:\Games\server", disabled: false };
const SOURCE: TickRow = TickRow { name: "Pack source folder, for its next version", how: "Repository", path: r"C:\Code\pack", disabled: false };

/// A tick list in a box `COL` wide and `h` tall, its line at the bottom when the list opens up, at the top when it
/// opens down. `open` presses the line once.
fn list(line: &'static str, places: &'static [Place], opts: TickListOpts, open: bool, h: f32) -> Cell {
    let probe = Probe::new();
    let shown = Memo::new(false);
    let line_id: Memo<Option<Id>> = Memo::new(None);
    Cell::new(State::Rest, move |ui: &mut Ui, _| {
        // A fresh `Ui` (the harness builds a cell in several) starts closed.
        if line_id.get().and_then(|id| ui.rect(id)).is_none() {
            shown.set(false);
        }
        if let Some(r) = if open { probe.due(ui) } else { None } {
            click(ui, centre(r));
        }
        let style = Style { column: true, w: Dim::Pct(100.0), h: Dim::Px(h), justify: if opts.up { Justify::End } else { Justify::Start }, ..Style::default() };
        let mut id = None;
        ui.node("box", style, |ui| {
            let mut o = shown.get();
            let r = ui.tick_list("place", line, &mut o, opts, |ui| {
                for (i, (row, ticked, rule)) in places.iter().enumerate() {
                    if *rule {
                        ui.tick_rule();
                    }
                    let mut t = *ticked;
                    ui.tick_row(&format!("p{i}"), *row, &mut t);
                }
                if !places.is_empty() {
                    ui.tick_rule();
                }
                ui.tick_action("choose", "Choose a folder\u{2026}");
            });
            shown.set(o);
            id = Some(r.line.id);
        });
        let id = id.unwrap();
        probe.set(id);
        line_id.set(Some(id));
        id
    })
}

pub fn blocks() -> Vec<Block> {
    const ONE: &[Place] = &[(VANILLA, true, false)];
    const TWO: &[Place] = &[(PRISM, false, false), (VANILLA, true, false)];
    const FOUR: &[Place] = &[(PRISM, true, false), (ATELIER, false, false), (SERVER, false, true), (SOURCE, false, false)];
    let up = TickListOpts { up: true, ..TickListOpts::default() };
    let off = TickListOpts { disabled: true, ..up };
    vec![
        Block::matrix(vec![vec![list("To: Exploration", ONE, up, false, 24.0), list("To: choose a folder", &[], off, false, 24.0)]]).col_width(COL),
        Block::matrix(vec![vec![
            list("To: Exploration", ONE, up, true, 120.0),
            list("To: Exploration (Minecraft Launcher)", TWO, up, true, 170.0),
            list("To: Exploration", FOUR, up, true, 260.0),
        ]])
        .col_width(COL),
        Block::matrix(vec![vec![list("To: Exploration (Minecraft Launcher)", TWO, TickListOpts::default(), true, 170.0)]]).col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
