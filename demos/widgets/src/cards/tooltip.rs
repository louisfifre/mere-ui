//! The Tooltip card: the tip on one line and wrapped at its width (components/Tooltip/preview.html).
//!
//! The kit's tip is the real one: a titled widget under a pointer that has rested for `TIP_DELAY`. It is drawn
//! under the pointer, so an invisible anchor above the cell puts the tip on the cell's top-left corner, and
//! the cell keeps the room the tip takes.

use std::cell::Cell as Memo;

use nui_base::{Event, Vec2};
use nui_layout::{Dim, Insets, Position, Style};
use nui_ui::tokens::{FRAME, INSET, INSET_TIGHT, TIP_WIDTH};
use nui_ui::{Id, TIP_DELAY, TextLook, Theme, Tone, Ui, wrap_lines};

use super::{Block, Card, Cell, State, render};

/// The tip hangs this far under the pointer.
const BELOW: f32 = 20.0;
const SHORT: &str = "A short tooltip";
const LONG: &str = "A longer tooltip that runs past the wrap width, so it goes on over a second line under the first one.";

/// The tip's box, as the kit measures it: the wrapped lines of the small text and the frame with its insets.
fn size_of(text: &str) -> Vec2 {
    let look = TextLook::small(Tone::Text);
    let lines = wrap_lines(text, look, TIP_WIDTH - 2.0 * (FRAME + INSET));
    let w = lines.iter().map(|l| look.width(l)).fold(0.0, f32::max) + 2.0 * (FRAME + INSET);
    Vec2::new(w, lines.len() as f32 * look.line + 2.0 * (FRAME + INSET_TIGHT))
}

fn tip(text: &'static str) -> Cell {
    let anchor: Memo<Option<Id>> = Memo::new(None);
    Cell::new(State::Rest, move |ui: &mut Ui, _| {
        // Each build is past the delay: the second puts the pointer on the anchor, which starts the tip's
        // wait, and the third is past it.
        ui.set_clock(ui.clock() + 2.0 * TIP_DELAY);
        let at = anchor.get().and_then(|id| ui.rect(id));
        if let (Some(r), None) = (at, ui.pointer()) {
            ui.event(&Event::PointerMoved { pos: Vec2::new(r.x, r.y) });
        }
        let room = size_of(text);
        let mut id = None;
        let cell = ui.node("room", Style { w: Dim::Px(room.x), h: Dim::Px(room.y), ..Style::default() }, |ui| {
            let hook = Style {
                position: Position::Absolute,
                inset: Insets { left: Dim::Px(0.0), top: Dim::Px(-BELOW), right: Dim::Auto, bottom: Dim::Auto },
                w: Dim::Px(2.0),
                h: Dim::Px(2.0),
                ..Style::default()
            };
            id = Some(ui.node("anchor", hook, |_| {}));
        });
        let id = id.unwrap();
        anchor.set(Some(id));
        ui.title(id, text);
        cell
    })
}

pub fn blocks() -> Vec<Block> {
    vec![Block::matrix(vec![vec![tip(SHORT)], vec![tip(LONG)]])]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
