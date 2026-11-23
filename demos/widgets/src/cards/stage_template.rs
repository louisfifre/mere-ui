//! The Stage Template card: a stage in the centre and a panel on each side, the template only placing them. Beside
//! with both panels open (the grip of the left one under the pointer), beside with the right panel closed (the stage
//! grows into its room), and stacked, the panels under the stage. The stage is drawn as a tinted box and each panel
//! holds a titled group, so that the picture shows the places and the spacing, not an application. The Design
//! System has no preview of it yet.

use std::cell::Cell as Memo;

use mere_base::{Event, Vec2};
use mere_layout::{Dim, Edges, Style};
use mere_ui::tokens::{GAP_WIDE, SECTION};
use mere_ui::{Id, Part, SelectionRow, StagePlan, Theme, Ui};

use super::{Block, Card, Cell, State, render};

fn panel(ui: &mut Ui, side: usize) {
    let title = if side == 0 { "Settings" } else { "Figures" };
    ui.titled_group("g", title, |_| {}, |ui| {
        for (i, label) in ["First", "Second"].into_iter().enumerate() {
            ui.selection_row(&format!("r{i}"), SelectionRow { label, selected: i == 0, ..SelectionRow::default() });
        }
    });
}

/// A page `w` by `h` laid out by the template from `plan`; `grip` puts the pointer on the left grip.
fn page(w: f32, h: f32, plan: StagePlan, grip: bool) -> Cell {
    let at: Memo<Option<Id>> = Memo::new(None);
    Cell::new(State::Rest, move |ui: &mut Ui, _| {
        if grip {
            if let Some(r) = at.get().and_then(|id| ui.rect(id)) {
                ui.event(&Event::PointerMoved { pos: Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0) });
            }
        }
        let tint = ui.theme().tint();
        let mut y = [0.0; 2];
        let mut id = None;
        ui.node("window", Style { column: true, w: Dim::Px(w), h: Dim::Px(h), ..Style::default() }, |ui| {
            let r = ui.stage_template("page", plan, &mut y, |ui, part| match part {
                Part::Side(side) => panel(ui, side),
                Part::Stage => ui.fill(Style { grow: 1.0, ..Style::default() }, tint),
            });
            at.set(r.grips[0].map(|g| g.resp.id));
            id = Some(r.page);
        });
        id.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    let pad = Edges::new(GAP_WIDE, SECTION, SECTION, SECTION);
    let both = StagePlan { widths: [Some(160.0), Some(160.0)], stacked: false, stage_h: 0.0, gap: SECTION, pad };
    let left = StagePlan { widths: [Some(160.0), None], ..both };
    let stacked = StagePlan { widths: [Some(170.0), Some(170.0)], stacked: true, stage_h: 120.0, gap: GAP_WIDE, pad: Edges::new(GAP_WIDE, GAP_WIDE, GAP_WIDE, GAP_WIDE) };
    vec![
        Block::single(page(640.0, 200.0, both, true)),
        Block::single(page(640.0, 200.0, left, false)),
        Block::single(page(376.0, 300.0, stacked, false)),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
