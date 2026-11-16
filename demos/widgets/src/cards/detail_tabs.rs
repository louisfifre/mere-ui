//! The DetailTabs card: one detail tab in rest, pressed and disabled, in columns 160 px wide
//! (components/DetailTabs/preview.html).

use mere_layout::{Dim, Track, Tracks};
use mere_ui::{TabOpts, Theme, Ui, details_style};

use super::{Block, Card, Cell, State, render};

/// `#details` with a single column (`gridTemplateColumns = '1fr'`), as wide as the cell.
fn cell(opts: TabOpts) -> Cell {
    Cell::new(State::Rest, move |ui: &mut Ui, _| {
        let style = mere_layout::Style { cols: Tracks::new(&[Track::MinMaxFr0(1.0)]), w: Dim::Pct(100.0), ..details_style() };
        let mut id = None;
        ui.node("details", style, |ui| id = Some(ui.detail_tab("d", "Value", "12 s", opts).id));
        id.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    vec![
        Block::matrix(vec![vec![
            cell(TabOpts::default()),
            cell(TabOpts { pressed: true, disabled: false }),
            cell(TabOpts { pressed: false, disabled: true }),
        ]])
        .col_width(160.0),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
