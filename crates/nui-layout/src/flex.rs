//! The flex algorithm of design.md section 3.3, steps 1 to 10, after the W3C flexbox steps that Taffy follows.
//! Everything is border-box. A node is laid out by `Tree::layout`, which either measures it (nothing is
//! stored) or places it (its children's rects are written, recursively).

use nui_base::{Rect, Vec2};

use crate::style::{Align, Avail, Display, Justify, Side};
use crate::tree::{NodeId, Tree};

const INF: f32 = f32::INFINITY;

/// What a parent tells a child: sizes it must have (`k*`, border-box) and room it may use (`a*`). A known
/// size is imposed by flexing or stretching; room only matters to content that sizes itself (wrapping text,
/// a row that shrinks to fit).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Req {
    pub kw: Option<f32>,
    pub kh: Option<f32>,
    pub aw: Option<f32>,
    pub ah: Option<f32>,
}

impl Req {
    pub(crate) fn key(self) -> [Option<u32>; 4] {
        let k = |v: Option<f32>| v.map(|v| (v + 0.0).to_bits());
        [k(self.kw), k(self.kh), k(self.aw), k(self.ah)]
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Out {
    pub w: f32,
    pub h: f32,
    /// From the top of the border box to the first line's baseline.
    pub baseline: f32,
}

/// Swaps a pair when the container is a column, so that one code path serves both directions:
/// `ax(col, w, h)` gives (main, cross) and `ax(col, main, cross)` gives (w, h).
fn ax<T>(col: bool, a: T, b: T) -> (T, T) {
    if col { (b, a) } else { (a, b) }
}

struct Item {
    id: NodeId,
    align: Align,
    /// Main-axis and cross-axis margins, (start, end).
    mm: (Side, Side),
    mc: (Side, Side),
    stretch: bool,
    cross_known: Option<f32>,
    min_cross: f32,
    max_cross: f32,
    min: f32,
    max: f32,
    grow: f32,
    shrink: f32,
    base: f32,
    hyp: f32,
    main: f32,
    cross: f32,
    baseline: f32,
    main_pos: f32,
    cross_pos: f32,
}

impl Item {
    fn mm_px(&self) -> f32 {
        self.mm.0.px() + self.mm.1.px()
    }
    fn mc_px(&self) -> f32 {
        self.mc.0.px() + self.mc.1.px()
    }
    fn clamp_cross(&self, v: f32) -> f32 {
        v.min(self.max_cross).max(self.min_cross)
    }
}

impl Out {
    fn split(&self, col: bool) -> (f32, f32) {
        ax(col, self.w, self.h)
    }
}

impl Tree {
    pub(crate) fn layout(&mut self, id: NodeId, req: Req, place: bool) -> Out {
        if !place {
            if let Some(o) = self.cache.get(&(id.0, req.key())) {
                return *o;
            }
        }
        let out = if self.nodes[id.0].measure.is_some() { self.leaf(id, req) } else { self.container(id, req, place) };
        if !place {
            self.cache.insert((id.0, req.key()), out);
        }
        out
    }

    fn leaf(&mut self, id: NodeId, req: Req) -> Out {
        let pad = self.nodes[id.0].style.pad;
        let (px, py) = (pad.left + pad.right, pad.top + pad.bottom);
        let sub = |v: Option<f32>, p: f32| v.map(|v| (v - p).max(0.0));
        let avail = Avail { w: sub(req.kw, px).or(sub(req.aw, px)), h: sub(req.kh, py).or(sub(req.ah, py)) };
        // The closure is taken out of the node while it runs, so that it may not reach the tree.
        let mut f = self.nodes[id.0].measure.take().expect("a leaf has a measure function");
        let m = f(avail);
        self.nodes[id.0].measure = Some(f);
        Out { w: req.kw.unwrap_or(m.size.x + px), h: req.kh.unwrap_or(m.size.y + py), baseline: m.baseline + pad.top }
    }

    /// Measures one child as a flex item would be: `main_*` and `cross_*` instead of width and height.
    fn probe(&mut self, id: NodeId, col: bool, known: (Option<f32>, Option<f32>), avail: (Option<f32>, Option<f32>)) -> Out {
        let (kw, kh) = ax(col, known.0, known.1);
        let (aw, ah) = ax(col, avail.0, avail.1);
        self.layout(id, Req { kw, kh, aw, ah }, false)
    }

    fn container(&mut self, id: NodeId, req: Req, place: bool) -> Out {
        let st = self.nodes[id.0].style;
        let (px, py) = (st.pad.left + st.pad.right, st.pad.top + st.pad.bottom);
        // Step 1 of the page's `display: block`: a column of full-width boxes, no gap.
        let (col, wrap, gap, items_align, justify) = if st.display == Display::Block {
            (true, false, Vec2::new(0.0, 0.0), Align::Stretch, Justify::Start)
        } else {
            (st.column, st.wrap, st.gap, st.align_items, st.justify)
        };
        let sub = |v: Option<f32>, p: f32| v.map(|v| (v - p).max(0.0));
        // Step 2: inner sizes, definite when the style or the parent gave them.
        let (ikw, ikh) = (sub(req.kw, px), sub(req.kh, py));
        let (iaw, iah) = (ikw.or(sub(req.aw, px)), ikh.or(sub(req.ah, py)));
        let (kmain, kcross) = ax(col, ikw, ikh);
        let (amain, across) = ax(col, iaw, iah);
        let (gap_main, gap_line) = if col { (gap.y, gap.x) } else { (gap.x, gap.y) };

        // Step 1: in-flow children, `display: none` out, stable sort by `order`.
        let mut kids: Vec<NodeId> =
            self.nodes[id.0].children.iter().copied().filter(|c| self.nodes[c.0].style.display != Display::None).collect();
        kids.sort_by_key(|c| self.nodes[c.0].style.order);

        // Steps 3 and 4: flex base size, automatic minimum, hypothetical main size.
        let mut items: Vec<Item> = Vec::with_capacity(kids.len());
        for kid in kids {
            let cs = self.nodes[kid.0].style;
            let m = cs.margin;
            let (mm, mc) = if col { ((m.top, m.bottom), (m.left, m.right)) } else { ((m.left, m.right), (m.top, m.bottom)) };
            let mut sw = cs.w.resolve(ikw);
            let mut sh = cs.h.resolve(ikh);
            if let Some(a) = cs.aspect {
                if sw.is_none() && sh.is_some() {
                    sw = sh.map(|h| h * a);
                } else if sh.is_none() && sw.is_some() {
                    sh = sw.map(|w| w / a);
                }
            }
            let (styled_main, styled_cross) = ax(col, sw, sh);
            let (min_main_spec, min_cross_spec) = ax(col, cs.min_w.resolve(ikw), cs.min_h.resolve(ikh));
            let (max_main, max_cross) = ax(col, cs.max_w.resolve(ikw).unwrap_or(INF), cs.max_h.resolve(ikh).unwrap_or(INF));
            let min_cross = min_cross_spec.unwrap_or(0.0);
            let align = cs.align_self.unwrap_or(items_align);
            let align = if align == Align::Baseline && col { Align::Start } else { align };
            let mc_auto = matches!(mc.0, Side::Auto) || matches!(mc.1, Side::Auto);
            let mc_sum = mc.0.px() + mc.1.px();
            let stretch = align == Align::Stretch && styled_cross.is_none() && !mc_auto;
            // A wrapping container has no single cross size to stretch to: its lines decide, below.
            let stretched = if stretch && !wrap { kcross.map(|c| (c - mc_sum).max(0.0)) } else { None };
            let cross_known = styled_cross.or(stretched).map(|c| c.min(max_cross).max(min_cross));

            let basis = cs.basis.resolve(kmain);
            let mut content: Option<f32> = None;
            let mut content_main = |this: &mut Tree| -> f32 {
                *content.get_or_insert_with(|| this.probe(kid, col, (None, cross_known), (None, across)).split(col).0)
            };
            let base = match basis.or(styled_main) {
                Some(v) => v,
                None => content_main(self),
            };
            // Step 4. `Auto` minimum: the content's min size, at most the specified size. A column's
            // content height is already its min-content; a row's is its width when told it has no room.
            let min = match min_main_spec {
                Some(v) => v,
                None => {
                    let min_content = if col {
                        content_main(self)
                    } else {
                        self.probe(kid, col, (None, cross_known), (Some(0.0), across)).split(col).0
                    };
                    min_content.min(styled_main.unwrap_or(INF))
                }
            };
            let hyp = base.min(max_main).max(min);
            items.push(Item {
                id: kid,
                align,
                mm,
                mc,
                stretch,
                cross_known,
                min_cross,
                max_cross,
                min,
                max: max_main,
                grow: cs.grow,
                shrink: cs.shrink,
                base,
                hyp,
                main: hyp,
                cross: 0.0,
                baseline: 0.0,
                main_pos: 0.0,
                cross_pos: 0.0,
            });
        }

        // Step 5: lines. The room a line may take is the container's main size, or, when that is open,
        // the room offered from outside for a row (a column has no height limit).
        let n = items.len();
        let sum_hyp: f32 = items.iter().map(|i| i.hyp + i.mm_px()).sum::<f32>() + gap_main * n.saturating_sub(1) as f32;
        let limit = kmain.or(if col { None } else { amain });
        let room = match kmain {
            Some(m) => Some(m),
            None if wrap => limit,
            None if col => None,
            // A row that sizes itself shrinks to the room offered, never grows into it.
            None => Some(sum_hyp.min(amain.unwrap_or(INF))),
        };
        let mut lines: Vec<(usize, usize)> = Vec::new();
        let (mut start, mut acc) = (0, 0.0f32);
        for (i, it) in items.iter().enumerate() {
            let outer = it.hyp + it.mm_px();
            if wrap && i > start && acc + gap_main + outer > limit.unwrap_or(INF) + 1e-4 {
                lines.push((start, i));
                start = i;
                acc = outer;
            } else {
                acc += if i > start { gap_main } else { 0.0 } + outer;
            }
        }
        if n > 0 {
            lines.push((start, n));
        }

        // Step 6: flexible lengths per line.
        for &(s, e) in &lines {
            flex_line(&mut items[s..e], room, gap_main);
        }

        // Step 7: cross size of each item at its final main size.
        for it in items.iter_mut() {
            let o = self.probe(it.id, col, (Some(it.main), it.cross_known), (Some(it.main), across));
            it.cross = it.clamp_cross(o.split(col).1);
            it.baseline = o.baseline;
        }
        let mut line_cross: Vec<(f32, f32)> = Vec::with_capacity(lines.len());
        for &(s, e) in &lines {
            let (mut max_outer, mut above, mut below) = (0.0f32, 0.0f32, 0.0f32);
            for it in &items[s..e] {
                let outer = it.cross + it.mc_px();
                if it.align == Align::Baseline {
                    let a = it.mc.0.px() + it.baseline;
                    above = above.max(a);
                    below = below.max(outer - a);
                } else {
                    max_outer = max_outer.max(outer);
                }
            }
            let natural = max_outer.max(above + below);
            // A single line takes the container's whole cross size when it has one.
            let lc = if lines.len() == 1 { kcross.unwrap_or(natural) } else { natural };
            line_cross.push((lc, above));
        }
        let cross_inner =
            kcross.unwrap_or_else(|| line_cross.iter().map(|l| l.0).sum::<f32>() + gap_line * lines.len().saturating_sub(1) as f32);

        // Step 8: stretch to the line, then position across.
        let mut cy = 0.0f32;
        for (li, &(s, e)) in lines.iter().enumerate() {
            let (lc, lb) = line_cross[li];
            for it in &mut items[s..e] {
                if it.stretch {
                    it.cross = it.clamp_cross((lc - it.mc_px()).max(0.0));
                }
                let outer = it.cross + it.mc_px();
                let free = lc - outer;
                let autos = matches!(it.mc.0, Side::Auto) as u8 + matches!(it.mc.1, Side::Auto) as u8;
                it.cross_pos = if autos > 0 && free > 0.0 {
                    let share = free / autos as f32;
                    cy + if matches!(it.mc.0, Side::Auto) { share } else { it.mc.0.px() }
                } else {
                    match it.align {
                        Align::Start | Align::Stretch => cy + it.mc.0.px(),
                        Align::Center => cy + free / 2.0 + it.mc.0.px(),
                        Align::End => cy + free + it.mc.0.px(),
                        Align::Baseline => cy + lb - it.baseline,
                    }
                };
            }
            cy += lc + gap_line;
        }

        // Step 9: main axis. Auto margins take the free space first, then `justify-content`.
        let used = |line: &[Item]| line.iter().map(|i| i.main + i.mm_px()).sum::<f32>() + gap_main * line.len().saturating_sub(1) as f32;
        let main_inner = kmain.unwrap_or_else(|| lines.iter().map(|&(s, e)| used(&items[s..e])).fold(0.0, f32::max));
        for &(s, e) in &lines {
            let free = main_inner - used(&items[s..e]);
            let autos: usize = items[s..e].iter().map(|i| matches!(i.mm.0, Side::Auto) as usize + matches!(i.mm.1, Side::Auto) as usize).sum();
            let share = if autos > 0 && free > 0.0 { free / autos as f32 } else { 0.0 };
            let mut cursor = if autos > 0 {
                0.0
            } else {
                match justify {
                    Justify::Start => 0.0,
                    Justify::Center => free / 2.0,
                    Justify::End => free,
                }
            };
            for it in &mut items[s..e] {
                let side = |s: Side| if matches!(s, Side::Auto) { share } else { s.px() };
                cursor += side(it.mm.0);
                it.main_pos = cursor;
                cursor += it.main + side(it.mm.1) + gap_main;
            }
        }

        // Step 10: the container's own size and baseline.
        let (iw, ih) = ax(col, main_inner, cross_inner);
        let (w, h) = (req.kw.unwrap_or(iw + px), req.kh.unwrap_or(ih + py));
        let baseline = match items.first() {
            Some(f) => st.pad.top + if col { f.main_pos } else { f.cross_pos } + f.baseline,
            None => h,
        };

        if place {
            let mut reach = Vec2::new(w, h);
            for it in &items {
                let (x, y) = ax(col, it.main_pos, it.cross_pos);
                let (cw, ch) = ax(col, it.main, it.cross);
                let rect = Rect::new(x + st.pad.left, y + st.pad.top, cw, ch);
                self.layout(it.id, Req { kw: Some(cw), kh: Some(ch), aw: Some(cw), ah: Some(ch) }, true);
                self.nodes[it.id.0].rect = rect;
                let m = self.nodes[it.id.0].style.margin;
                reach.x = reach.x.max(rect.x + rect.w + m.right.px() + st.pad.right);
                reach.y = reach.y.max(rect.y + rect.h + m.bottom.px() + st.pad.bottom);
            }
            self.nodes[id.0].content = reach;
        }
        Out { w, h, baseline }
    }
}

/// Step 6, W3C 9.7: grow or shrink the line's items to its room, freezing those that hit a min or max and
/// sharing what is left among the others, until none is left to freeze.
fn flex_line(items: &mut [Item], room: Option<f32>, gap: f32) {
    let n = items.len();
    let Some(room) = room else {
        for it in items.iter_mut() {
            it.main = it.hyp;
        }
        return;
    };
    let gaps = gap * n.saturating_sub(1) as f32;
    let growing = items.iter().map(|i| i.hyp + i.mm_px()).sum::<f32>() + gaps < room;
    let mut frozen = vec![false; n];
    for (i, it) in items.iter_mut().enumerate() {
        let factor = if growing { it.grow } else { it.shrink };
        it.main = it.base;
        // An item that would move the wrong way, or cannot flex, stays at its hypothetical size.
        if factor == 0.0 || (growing && it.base > it.hyp) || (!growing && it.base < it.hyp) {
            it.main = it.hyp;
            frozen[i] = true;
        }
    }
    let free_of = |items: &[Item], frozen: &[bool]| {
        room - gaps - items.iter().zip(frozen).map(|(it, &f)| (if f { it.main } else { it.base }) + it.mm_px()).sum::<f32>()
    };
    let initial_free = free_of(items, &frozen);
    for _ in 0..=n {
        if frozen.iter().all(|&f| f) {
            break;
        }
        let mut free = free_of(items, &frozen);
        let factor = |it: &Item| if growing { it.grow } else { it.shrink };
        let sum_f: f32 = items.iter().zip(&frozen).filter(|(_, f)| !**f).map(|(it, _)| factor(it)).sum();
        // Factors adding up to less than one share out only that fraction of the free space.
        if sum_f < 1.0 && (initial_free * sum_f).abs() < free.abs() {
            free = initial_free * sum_f;
        }
        let scaled_total: f32 = items.iter().zip(&frozen).filter(|(_, f)| !**f).map(|(it, _)| it.shrink * it.base).sum();
        let mut raw = vec![0.0f32; n];
        let mut violation = 0.0f32;
        for i in 0..n {
            if frozen[i] {
                continue;
            }
            let it = &items[i];
            raw[i] = if growing {
                if sum_f > 0.0 { it.base + free * it.grow / sum_f } else { it.base }
            } else if scaled_total > 0.0 {
                // Shrinking is weighted by the base size: a large item gives up more than a small one.
                it.base + free * (it.shrink * it.base) / scaled_total
            } else {
                it.base
            };
            let clamped = raw[i].min(it.max).max(it.min);
            violation += clamped - raw[i];
        }
        for i in 0..n {
            if frozen[i] {
                continue;
            }
            let clamped = raw[i].min(items[i].max).max(items[i].min);
            let freeze = violation == 0.0 || (violation > 0.0 && clamped > raw[i]) || (violation < 0.0 && clamped < raw[i]);
            items[i].main = clamped;
            frozen[i] = freeze;
        }
    }
}
