//! Absolutely positioned boxes (design.md section 3.3, "Absolute"), reduced to what the pages do: every
//! absolute box is a direct child of a positioned box, so the containing block is the parent's padding box,
//! which is its border box here since there are no borders. They are placed after the parent's own layout,
//! never take part in measuring, and never move their in-flow siblings.

use mere_base::Rect;

use crate::style::{Display, Position};
use crate::tree::{NodeId, Req, Tree};

impl Tree {
    /// Places the absolute children of `id`, whose border box is `w` x `h`.
    pub(crate) fn place_absolute(&mut self, id: NodeId, w: f32, h: f32) {
        let kids: Vec<NodeId> = self.nodes[id.0]
            .children
            .iter()
            .copied()
            .filter(|c| {
                let s = &self.nodes[c.0].style;
                s.display != Display::None && s.position == Position::Absolute
            })
            .collect();
        if kids.is_empty() {
            return;
        }
        let pad = self.nodes[id.0].style.pad;
        if self.nodes[id.0].style.position == Position::Static && self.nodes[id.0].parent.is_some() {
            mere_base::observe!(
                Warn,
                "{}: absolute box in a static parent, placed against that parent anyway",
                self.nodes[id.0].name
            );
        }
        for kid in kids {
            let cs = self.nodes[kid.0].style;
            let (l, t) = (cs.inset.left.resolve(Some(w)), cs.inset.top.resolve(Some(h)));
            let (r, b) = (cs.inset.right.resolve(Some(w)), cs.inset.bottom.resolve(Some(h)));
            let (ml, mr, mt, mb) = (cs.margin.left.px(), cs.margin.right.px(), cs.margin.top.px(), cs.margin.bottom.px());
            let styled_h = cs.h.resolve(Some(h));

            let cw = if let Some(v) = cs.w.resolve(Some(w)) {
                v
            } else if let (Some(l), Some(r)) = (l, r) {
                (w - l - r - ml - mr).max(0.0)
            } else if let (Some(a), Some(sh)) = (cs.aspect, styled_h) {
                sh * a
            } else {
                // Shrink-to-fit: the content's width, within the room between the insets.
                let room = (w - l.unwrap_or(0.0) - r.unwrap_or(0.0) - ml - mr).max(0.0);
                self.layout(kid, Req { kw: None, kh: None, aw: Some(room), ah: None }, false).w
            };
            let cw = clamp(cw, cs.min_w.resolve(Some(w)), cs.max_w.resolve(Some(w)));

            let ch = if let Some(v) = styled_h {
                v
            } else if let (Some(t), Some(b)) = (t, b) {
                (h - t - b - mt - mb).max(0.0)
            } else if let Some(a) = cs.aspect {
                cw / a
            } else {
                self.layout(kid, Req { kw: Some(cw), kh: None, aw: Some(cw), ah: None }, false).h
            };
            let ch = clamp(ch, cs.min_h.resolve(Some(h)), cs.max_h.resolve(Some(h)));

            // With neither side set, the box stays where it would have been in flow; the pages always set one,
            // so that position is simplified to the padding start.
            let x = match (l, r) {
                (Some(l), _) => l + ml,
                (None, Some(r)) => w - r - mr - cw,
                (None, None) => pad.left + ml,
            };
            let y = match (t, b) {
                (Some(t), _) => t + mt,
                (None, Some(b)) => h - b - mb - ch,
                (None, None) => pad.top + mt,
            };
            self.layout(kid, Req { kw: Some(cw), kh: Some(ch), aw: Some(cw), ah: Some(ch) }, true);
            self.nodes[kid.0].rect = Rect::new(x, y, cw, ch);
        }
    }
}

fn clamp(v: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let v = if let Some(m) = max { v.min(m) } else { v };
    let v = if let Some(m) = min { v.max(m) } else { v };
    v.max(0.0)
}
