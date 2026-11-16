//! The ListRow card: the square swatch, the round mark and the swatch while the list is choosing, in rest,
//! hover, pressed and absent (components/ListRow/preview.html).

use mere_base::Color;
use mere_layout::{Dim, Style};
use mere_ui::{ListRow, RowOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// The preview's `col: '180px'`.
const COL: f32 = 180.0;
/// The swatches of the three rows (`G`).
const GREYS: [u32; 3] = [0x959595, 0xb1b1b1, 0x7a7a7a];

fn row(opts: RowOpts, grey: usize, state: State) -> Option<Cell> {
    Some(Cell::new(state, move |ui: &mut Ui, _| {
        let item = ListRow { swatch: Color::hex(GREYS[grey]), name: "Item", width: "240", share: "4.2%" };
        let mut id = None;
        ui.node("col", Style { column: true, w: Dim::Px(COL), ..Style::default() }, |ui| id = Some(ui.list_row("r", &item, opts).id));
        id.unwrap()
    }))
}

pub fn blocks() -> Vec<Block> {
    let plain = RowOpts::default();
    let mark = RowOpts { mark: true, ..plain };
    let choosing = RowOpts { choosing: true, ..plain };
    let absent = |o: RowOpts| RowOpts { absent: true, ..o };
    let pressed = RowOpts { pressed: true, ..choosing };
    vec![
        Block::sparse(vec![
            vec![row(plain, 0, State::Rest), row(plain, 0, State::Hover), None, row(absent(plain), 0, State::Rest)],
            vec![row(mark, 1, State::Rest), row(mark, 1, State::Hover), None, row(absent(mark), 1, State::Rest)],
            vec![row(choosing, 2, State::Rest), row(choosing, 2, State::Hover), row(pressed, 2, State::Rest), None],
        ])
        .col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
