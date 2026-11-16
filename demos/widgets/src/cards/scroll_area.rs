//! The ScrollArea card: a 100 px box of twelve (or two) lines at the top, in the middle, and without overflow,
//! at rest and with the pointer on the thumb (components/ScrollArea/preview.html).

use std::cell::Cell as Mem;

use nui_layout::{Dim, Edges, Insets, Position, Style};
use nui_ui::tokens::{SECTION, SPACE_4};
use nui_ui::{ARROW, GUTTER, ScrollOpts, THUMB_MIN, TextLook, Theme, Tone, Ui, scroll_style};

use super::{Block, Card, Cell, State, render};

/// The preview's `col: '200px'` and `height: 100`.
const COL: f32 = 200.0;
const BOX_H: f32 = 100.0;

/// One box of `n` lines. `at` is the preview's `o.at` (a share of the range, 0 for the top); `hot` puts the
/// pointer on the thumb (`state: 'thumb-hot'`).
fn cell(n: usize, at: f32, hot: bool) -> Cell {
    // The offset the area keeps between frames: its range is only known once it has been laid out once.
    let off = Mem::new(0.0f32);
    let state = if hot { State::Hover } else { State::Rest };
    Cell::new(state, move |ui: &mut Ui, _| {
        let mut y = off.get();
        let mut resp = None;
        let wrap = Style { column: true, w: Dim::Px(COL), ..Style::default() };
        ui.node("wrap", wrap, |ui| {
            let style = Style { h: Dim::Px(BOX_H), grow: 0.0, basis: Dim::Auto, ..scroll_style() };
            let r = ui.scroll("box", &mut y, style, ScrollOpts::default(), |ui| {
                let col = Style { column: true, ..Style::default() };
                ui.node("items", col, |ui| {
                    for i in 1..=n {
                        let line = Style { pad: Edges::vh(SPACE_4, 0.0), ..Style::default() };
                        ui.node(&format!("i{i}"), line, |ui| ui.label(&format!("Item {i}"), Tone::Text));
                    }
                });
            });
            if r.max > 0.0 {
                y = at * r.max;
                off.set(y);
            }
            resp = Some(r);
        });
        let r = resp.expect("the box was built");
        // A pointer rests on the centre of the id it is given and the thumb is a thin strip in the gutter, so
        // a 1 px marker sits on the thumb's middle for the harness to find. It is worked out from the
        // content's known height, not from last frame's rects, because the harness reads it from the first frame.
        let content_h = n as f32 * (TextLook::body(Tone::Text).line + 2.0 * SPACE_4) + SECTION;
        if !(hot && content_h > BOX_H) {
            return r.id;
        }
        let track = BOX_H - 2.0 * ARROW;
        let h = (track * BOX_H / content_h).max(THUMB_MIN).min(track);
        let top = ARROW + (track - h) * at + h / 2.0;
        let at_thumb = Style {
            position: Position::Absolute,
            inset: Insets { left: Dim::Px(COL + GUTTER / 2.0), top: Dim::Px(top), right: Dim::Auto, bottom: Dim::Auto },
            w: Dim::Px(1.0),
            h: Dim::Px(1.0),
            ..Style::default()
        };
        ui.node("thumb_mark", at_thumb, |_| {})
    })
}

pub fn blocks() -> Vec<Block> {
    vec![
        Block::sparse(vec![
            vec![Some(cell(12, 0.0, false)), Some(cell(12, 0.0, true))],
            vec![Some(cell(12, 0.5, false)), Some(cell(12, 0.5, true))],
            vec![Some(cell(2, 0.0, false)), None],
        ])
        .col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
