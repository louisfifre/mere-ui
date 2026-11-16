//! The TrendRow card: the default row, a name too long for its column, and a thicker stroke
//! (components/TrendRow/preview.html). The trend row has no states.

use nui_base::{Cmd, Vec2};
use nui_layout::{Dim, Style};
use nui_ui::{Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// The preview's width `W`: the row's and the plot's.
const W: f32 = 480.0;
const POINTS: usize = 8;

/// The preview's `pts(k)`: eight points over the width, heights between 4 and 25.
fn points(k: usize) -> Vec<Vec2> {
    (0..POINTS).map(|j| Vec2::new((j as f32 * W / (POINTS - 1) as f32).round(), (4 + (k * 5 + j * 9) % 22) as f32)).collect()
}

fn row(name: &'static str, value: &'static str, k: usize, stroke: f32) -> Vec<Cell> {
    vec![Cell::new(State::Rest, move |ui: &mut Ui, _| {
        let plot = vec![Cmd::Polyline { points: points(k), width: stroke, color: ui.theme().text }];
        let mut id = None;
        ui.node("col", Style { column: true, w: Dim::Px(W), ..Style::default() }, |ui| id = Some(ui.trend_row("t", name, value, plot)));
        id.unwrap()
    })]
}

pub fn blocks() -> Vec<Block> {
    vec![
        Block::matrix(vec![
            row("Label", "12.3%", 1, 1.5),
            row("A label long enough that it has to be shortened to leave room for the value at the end", "4.2%", 2, 1.5),
            row("Label", "1,250", 3, 3.0),
        ])
        .col_width(W),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
