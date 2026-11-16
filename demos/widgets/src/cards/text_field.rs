//! The TextField card: the field at control height, compact and tall; empty, rest, focused with the caret,
//! text selected and disabled (components/TextField/preview.html).

use nui_base::{Event, Key, Mods};
use nui_layout::Dim;
use nui_ui::{FieldLook, FieldOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// The preview's column width and the tall field's height.
const COL: f32 = 136.0;
const TALL: f32 = 52.0;

#[derive(Copy, Clone, PartialEq)]
enum Column {
    Empty,
    Rest,
    Caret,
    Selected,
    Disabled,
}

/// One field of the matrix. A focused field takes the keyboard with its text selected (`focus`), so the
/// caret column sends End first; the selected column leaves the selection alone.
fn field(look: FieldLook, height: Dim, column: Column) -> Cell {
    let state = if matches!(column, Column::Caret | Column::Selected) { State::Focus } else { State::Rest };
    Cell::new(state, move |ui: &mut Ui, _| {
        let mut text = if column == Column::Empty { String::new() } else { "Value".to_string() };
        if column == Column::Caret {
            ui.event(&Event::Key { key: Key::End, mods: Mods::default() });
        }
        let opts = FieldOpts {
            look,
            placeholder: if column == Column::Empty { "Placeholder" } else { "" },
            width: Dim::Pct(100.0),
            height,
            focus: matches!(column, Column::Caret | Column::Selected),
            disabled: column == Column::Disabled,
            ..FieldOpts::default()
        };
        ui.text_field("field", &mut text, opts).resp.id
    })
}

fn row(look: FieldLook, height: Dim) -> Vec<Cell> {
    [Column::Empty, Column::Rest, Column::Caret, Column::Selected, Column::Disabled].into_iter().map(|c| field(look, height, c)).collect()
}

pub fn blocks() -> Vec<Block> {
    vec![
        Block::matrix(vec![
            row(FieldLook::Text, Dim::Auto),
            row(FieldLook::Compact, Dim::Auto),
            row(FieldLook::Text, Dim::Px(TALL)),
        ])
        .col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
