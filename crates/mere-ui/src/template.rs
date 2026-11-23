//! The panel and the stage template, the top of the kit's levels: a template only places the panels and the stage,
//! the application filling them. A panel is deliberately no piece of its own: a column with no frame and no fill
//! that holds groups, placed by the template.
//!
//! The stage template: a stage in the centre and up to two side panels, a panel on each side that is open.
//!
//! - Beside: the left panel, the stage and the right panel in a row, the page's padding around them; a closed panel
//!   is not built and the stage grows into its room; between an open panel and the stage, a gap with the grip that
//!   resizes the panel in its middle.
//! - Stacked (a narrow window): the stage on top at the height given, the open panels under it in a row, each a
//!   whole that scrolls (`panel_scroll`); no grip.
//!
//! The widths are the application's (they depend on what it shows: the minimum a map needs, the widths a person
//! dragged): it passes them in `StagePlan`. The grip only reports the pointer (`GripResp`); the application keeps the
//! drag and the width it gives.
//!
//! A panel has no frame and no fill: its sections (a fold's head with its section rule) and its groups part it.

use mere_base::{Cmd, Cursor, Rect, Vec2};
use mere_layout::{Dim, Edges, Justify, Margin, Style};

use crate::id::Id;
use crate::scroll::{ScrollOpts, scroll_style};
use crate::tokens::GAP_WIDE;
use crate::ui::{Resp, Ui};

/// The grip of a gap between a panel and the stage: as wide as a grip is, a line of 1 px in its middle drawn under
/// the pointer (muted) and while held (text).
pub const SPLIT_GRIP_W: f32 = 8.0;

/// What the pointer did to a grip.
#[derive(Copy, Clone, Debug)]
pub struct GripResp {
    pub resp: Resp,
    /// A double press: put the panel back to its natural width.
    pub reset: bool,
}

/// Where the template places its parts, from the application's widths.
#[derive(Copy, Clone, Debug)]
pub struct StagePlan {
    /// Each side panel's width, None when it is closed.
    pub widths: [Option<f32>; 2],
    /// The panels go under the stage instead of beside it.
    pub stacked: bool,
    /// Stacked, the stage's height.
    pub stage_h: f32,
    /// The gap between a panel and the stage: beside, the grip's gap; stacked, between every part.
    pub gap: f32,
    /// Around the page.
    pub pad: Edges,
}

/// What the template asks the application to build.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Part {
    /// A side panel's content, 0 the left one and 1 the right one, built in the panel (scrolling as one whole when
    /// stacked).
    Side(usize),
    /// The stage's content.
    Stage,
}

/// The parts the template laid out.
#[derive(Copy, Clone, Debug)]
pub struct StageResp {
    pub page: Id,
    pub stage: Id,
    /// Each open panel.
    pub sides: [Option<Id>; 2],
    /// Each open panel's grip, beside.
    pub grips: [Option<GripResp>; 2],
    /// Each panel's scroll area, stacked.
    pub scrolls: [Option<Id>; 2],
}

fn panel_style(w: f32) -> Style {
    Style { column: true, w: Dim::Px(w), shrink: 0.0, min_h: Dim::Px(0.0), ..Style::default() }
}

impl Ui {
    /// A panel's content as one whole that scrolls: the kit's scrollbar in a gutter of its own at the panel's right
    /// edge, no fade, and `gap-wide` under the last thing so that it does not touch the window's edge.
    pub fn panel_scroll(&mut self, key: &str, y: &mut f32, body: impl FnOnce(&mut Ui)) -> Id {
        let style = Style { margin: Margin::default(), pad: Edges::new(0.0, 0.0, GAP_WIDE, 0.0), ..scroll_style() };
        self.scroll(key, y, style, ScrollOpts { fade: 0.0 }, body).id
    }

    /// A gap `gap` wide with a grip in its middle, as tall as what holds it. Nothing is drawn at rest.
    pub fn split_grip(&mut self, key: &str, gap: f32) -> GripResp {
        let t = *self.theme();
        let style = Style { w: Dim::Px(gap), shrink: 0.0, justify: Justify::Center, ..Style::default() };
        let mut out = None;
        self.node(key, style, |ui| {
            let grip = Style { w: Dim::Px(SPLIT_GRIP_W), shrink: 0.0, ..Style::default() };
            let resp = ui.canvas("grip", grip, |ui, resp| {
                if resp.hovered || resp.held {
                    ui.want_cursor(Cursor::ResizeEW);
                }
                let colour = if resp.held { Some(t.text) } else if resp.hovered { Some(t.muted) } else { None };
                match (colour, resp.rect) {
                    (Some(color), Some(r)) => vec![Cmd::Fill { rect: Rect::new((r.w / 2.0).floor(), 0.0, 1.0, r.h), color }],
                    _ => Vec::new(),
                }
            });
            let reset = resp.pressed && ui.clicks() >= 2;
            out = Some(GripResp { resp, reset });
        });
        out.expect("the grip is built")
    }

    /// The stage template. `part` builds each part the template asks for, in build order: the left panel, the stage,
    /// the right panel; `scroll_y` is where each stacked panel's scroll stands, kept by the application.
    pub fn stage_template(&mut self, key: &str, plan: StagePlan, scroll_y: &mut [f32; 2], mut part: impl FnMut(&mut Ui, Part)) -> StageResp {
        let mut out = StageResp { page: Id::ROOT, stage: Id::ROOT, sides: [None; 2], grips: [None; 2], scrolls: [None; 2] };
        let fills = Style { grow: 1.0, shrink: 1.0, basis: Dim::Px(0.0), min_h: Dim::Px(0.0), w: Dim::Pct(100.0), ..Style::default() };
        if !plan.stacked {
            let page = Style { pad: plan.pad, ..fills };
            let stage = Style { column: true, grow: 1.0, shrink: 1.0, basis: Dim::Px(0.0), min_w: Dim::Px(0.0), min_h: Dim::Px(0.0), ..Style::default() };
            out.page = self.node(key, page, |ui| {
                if let Some(w) = plan.widths[0] {
                    out.sides[0] = Some(ui.node("left", panel_style(w), |ui| part(ui, Part::Side(0))));
                    out.grips[0] = Some(ui.split_grip("gap0", plan.gap));
                }
                out.stage = ui.node("stage", stage, |ui| part(ui, Part::Stage));
                if let Some(w) = plan.widths[1] {
                    out.grips[1] = Some(ui.split_grip("gap1", plan.gap));
                    out.sides[1] = Some(ui.node("right", panel_style(w), |ui| part(ui, Part::Side(1))));
                }
            });
        } else {
            let page = Style { column: true, pad: plan.pad, gap: Vec2::new(0.0, plan.gap), ..fills };
            let stage = Style { column: true, h: Dim::Px(plan.stage_h), shrink: 0.0, min_h: Dim::Px(0.0), ..Style::default() };
            out.page = self.node(key, page, |ui| {
                out.stage = ui.node("stage", stage, |ui| part(ui, Part::Stage));
                let row = Style { gap: Vec2::new(plan.gap, 0.0), ..fills };
                ui.node("panels", row, |ui| {
                    for side in 0..2 {
                        let Some(w) = plan.widths[side] else { continue };
                        let name = if side == 0 { "left" } else { "right" };
                        out.sides[side] = Some(ui.node(name, panel_style(w), |ui| {
                            out.scrolls[side] = Some(ui.panel_scroll("panel", &mut scroll_y[side], |ui| part(ui, Part::Side(side))));
                        }));
                    }
                });
            });
        }
        out
    }
}
