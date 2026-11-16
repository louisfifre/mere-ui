//! The NumberField card: the compact number field, one with a taller padding and one with the value on the
//! left; at rest, focused and disabled (components/NumberField/preview.html).

use mere_base::Vec2;
use mere_layout::Dim;
use mere_ui::{NumberOpts, NumberSpec, Theme, Ui};

use super::select::{Probe, click};
use super::{Block, Card, Cell, State, render};

/// The preview's column width and its spec (`min 0, max 240, step 1`).
const COL: f32 = 120.0;
const SPEC: NumberSpec = NumberSpec { min: 0.0, max: 240.0, step: 1.0 };
/// How far inside the right edge the focusing click lands: past the value, so that the caret ends the text.
const CLICK_IN: f32 = 3.0;

#[derive(Copy, Clone, PartialEq)]
enum Column {
    Rest,
    Focus,
    Disabled,
}

fn field(pad_v: f32, right: bool, column: Column) -> Cell {
    let probe = Probe::new();
    Cell::new(State::Rest, move |ui: &mut Ui, _| {
        // A number field takes the keyboard from a press only, so the focused one is clicked once.
        let due = if column == Column::Focus { probe.due(ui) } else { None };
        if let Some(r) = due {
            click(ui, Vec2::new(r.x + r.w - CLICK_IN, r.y + r.h / 2.0));
        }
        let mut v = 12.0;
        let opts = NumberOpts { width: Dim::Pct(100.0), right, pad_v, disabled: column == Column::Disabled };
        let id = ui.number_with("field", &mut v, SPEC, opts).resp.id;
        probe.set(id);
        id
    })
}

fn row(pad_v: f32, right: bool) -> Vec<Cell> {
    [Column::Rest, Column::Focus, Column::Disabled].into_iter().map(|c| field(pad_v, right, c)).collect()
}

pub fn blocks() -> Vec<Block> {
    let default = NumberOpts::default();
    vec![Block::matrix(vec![row(default.pad_v, true), row(4.0, true), row(default.pad_v, false)]).col_width(COL)]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
