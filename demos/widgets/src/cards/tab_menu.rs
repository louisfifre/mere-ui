//! The TabMenu card: a strip tab with a chevron that opens a list, at rest, hovered and open; the list's first
//! row is disabled (components/TabMenu/preview.html).

use nui_base::Event;
use nui_layout::{Dim, Style};
use nui_ui::tokens::FRAME;
use nui_ui::{MENU_ROW, MenuRow, Theme, Ui, strip_style};

use super::select::{Probe, centre, click};
use super::{Block, Card, Cell, State, render};

/// Room under the tab for the open list, as the preview's `min-height` leaves it: its rows and its frame.
fn room(rows: usize) -> f32 {
    rows as f32 * MENU_ROW + 2.0 * FRAME
}

fn rows() -> Vec<MenuRow> {
    vec![
        MenuRow { text: "Option one".into(), disabled: true },
        MenuRow { text: "Option two".into(), disabled: false },
        MenuRow { text: "Option three".into(), disabled: false },
    ]
}

/// `trigger(o)`: the tab "Label" with its chevron. The three cells of the row all keep the room for the list,
/// the tab at their top, so that the tabs line up.
fn trigger(state: State, open: bool) -> Cell {
    let rows = rows();
    let probe = Probe::new();
    Cell::new(state, move |ui: &mut Ui, _| {
        let due = if open { probe.due(ui) } else { None };
        if let Some(r) = due {
            click(ui, centre(r));
        }
        let mut id = None;
        let column = Style { column: true, ..Style::default() };
        ui.node("cell", column, |ui| {
            ui.node("strip", strip_style(), |ui| {
                let r = ui.tab_menu("menu", "Label", &rows);
                probe.set(r.resp.id);
                id = Some(r.resp.id);
            });
            ui.node("room", Style { h: Dim::Px(room(rows.len())), w: Dim::Px(1.0), ..Style::default() }, |_| {});
        });
        if due.is_some() {
            // The list is open; the pointer leaves the tab so that it shows pressed, not hovered.
            ui.event(&Event::PointerLeft);
        }
        id.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![vec![trigger(State::Rest, false), trigger(State::Hover, false), trigger(State::Rest, true)]])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
