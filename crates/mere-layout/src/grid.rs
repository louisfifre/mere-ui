//! The grid container of design.md section 3.3, reduced to what the pages use: explicit columns (a length,
//! `auto`, `minmax(0, Nfr)`), implicit rows sized by their items, row-major placement, `place-items`.
//!
//! Not built: explicit rows, spans other than 1 and `1 / -1`, named areas, dense packing, auto margins (they
//! count as 0 here), percentage heights of items (treated as auto), bare `1fr` (`minmax(auto, 1fr)`, which the
//! launcher's page needs: it comes with step 21), `align-content` and `justify-content` other than `normal`,
//! baseline alignment in the column axis (it acts as `start`). Overflow is ignored when contributions are
//! measured: a scroll box in a grid contributes its content like any other box.

use mere_base::Rect;

use crate::style::{Align, Dim, Display, Position, Track};
use crate::tree::{NodeId, Out, Req, Tree};

const INF: f32 = f32::INFINITY;

struct Cell {
    id: NodeId,
    row: usize,
    col: usize,
    all: bool,
    ml: f32,
    mr: f32,
    mt: f32,
    mb: f32,
    align: Align,
    jalign: Align,
    /// Width of the area the item sits in (its column, or all of them).
    cw: f32,
    iw: f32,
    ih: f32,
    kh: Option<f32>,
    baseline: f32,
    x: f32,
    y: f32,
}

impl Cell {
    fn mx(&self) -> f32 {
        self.ml + self.mr
    }
    fn my(&self) -> f32 {
        self.mt + self.mb
    }
}

impl Tree {
    /// An item's width contribution to its column: the styled width, or what the item measures to when told
    /// it has `avail` room (`None` is max-content, `Some(0.0)` is min-content), margins included.
    fn contribution(&mut self, id: NodeId, avail: Option<f32>) -> f32 {
        let cs = self.nodes[id.0].style;
        let kh = if let Dim::Px(h) = cs.h { Some(h) } else { None };
        let mut w = match cs.w {
            Dim::Px(w) => w,
            _ => self.layout(id, Req { kw: None, kh, aw: avail, ah: None }, false).w,
        };
        if let Dim::Px(m) = cs.min_w {
            w = w.max(m);
        }
        if let Dim::Px(m) = cs.max_w {
            w = w.min(m);
        }
        w + cs.margin.left.px() + cs.margin.right.px()
    }

    pub(crate) fn grid(&mut self, id: NodeId, req: Req, place: bool) -> Out {
        let st = self.nodes[id.0].style;
        let (px, py) = (st.pad.left + st.pad.right, st.pad.top + st.pad.bottom);
        let sub = |v: Option<f32>, p: f32| v.map(|v| (v - p).max(0.0));
        let (ikw, ikh) = (sub(req.kw, px), sub(req.kh, py));
        let iaw = ikw.or(sub(req.aw, px));
        let (gx, gy) = (st.gap.x, st.gap.y);

        // Step 1: in-flow children, `display: none` and absolute boxes out, stable sort by `order`.
        let mut kids: Vec<NodeId> = self.nodes[id.0]
            .children
            .iter()
            .copied()
            .filter(|c| {
                let s = &self.nodes[c.0].style;
                s.display != Display::None && s.position != Position::Absolute
            })
            .collect();
        kids.sort_by_key(|c| self.nodes[c.0].style.order);

        // Step 2: no template is one implicit auto column.
        let tracks: Vec<Track> = if st.cols.as_slice().is_empty() { vec![Track::Auto] } else { st.cols.as_slice().to_vec() };
        let n = tracks.len();

        // Step 3: row-major placement.
        let mut cells: Vec<Cell> = Vec::with_capacity(kids.len());
        let (mut r, mut c) = (0usize, 0usize);
        for kid in kids {
            let cs = self.nodes[kid.0].style;
            let align = cs.align_self.unwrap_or(st.align_items);
            let jalign = match cs.justify_self.unwrap_or(st.justify_items) {
                Align::Baseline => Align::Start,
                a => a,
            };
            let (row, col) = if cs.col_span_all {
                if c > 0 {
                    r += 1;
                    c = 0;
                }
                let p = (r, 0);
                r += 1;
                p
            } else {
                let p = (r, c);
                c += 1;
                if c == n {
                    r += 1;
                    c = 0;
                }
                p
            };
            cells.push(Cell {
                id: kid,
                row,
                col,
                all: cs.col_span_all,
                ml: cs.margin.left.px(),
                mr: cs.margin.right.px(),
                mt: cs.margin.top.px(),
                mb: cs.margin.bottom.px(),
                align,
                jalign,
                cw: 0.0,
                iw: 0.0,
                ih: 0.0,
                kh: None,
                baseline: 0.0,
                x: 0.0,
                y: 0.0,
            });
        }
        let rows = cells.iter().map(|c| c.row + 1).max().unwrap_or(0);

        // Step 4: what the single-span items ask of their column.
        let mut maxc = vec![0.0f32; n];
        let mut minc = vec![0.0f32; n];
        for i in 0..cells.len() {
            if cells[i].all {
                continue;
            }
            let (id, col) = (cells[i].id, cells[i].col);
            let hi = self.contribution(id, None);
            let lo = self.contribution(id, Some(0.0));
            maxc[col] = maxc[col].max(hi);
            minc[col] = minc[col].max(lo);
        }

        // Step 5: the inner width. A grid that sizes itself shrinks to the room offered, never grows into it.
        let gaps_w = gx * n.saturating_sub(1) as f32;
        let wid = match ikw {
            Some(w) => w,
            None => {
                let total: f32 = tracks
                    .iter()
                    .enumerate()
                    .map(|(i, t)| match t {
                        Track::Px(v) => *v,
                        _ => maxc[i],
                    })
                    .sum::<f32>()
                    + gaps_w;
                total.min(iaw.unwrap_or(INF))
            }
        };

        // Step 6: track sizes.
        let mut size = vec![0.0f32; n];
        let mut growth = vec![0.0f32; n];
        for (i, t) in tracks.iter().enumerate() {
            match t {
                Track::Px(v) => size[i] = *v,
                Track::Auto => {
                    size[i] = minc[i];
                    growth[i] = maxc[i].max(minc[i]);
                }
                Track::MinMaxFr0(_) => {}
            }
        }
        // Maximize: auto tracks grow towards their max-content, sharing the room equally.
        for _ in 0..n {
            let free = wid - size.iter().sum::<f32>() - gaps_w;
            if free <= 1e-4 {
                break;
            }
            let growing: Vec<usize> =
                (0..n).filter(|&i| matches!(tracks[i], Track::Auto) && size[i] < growth[i]).collect();
            if growing.is_empty() {
                break;
            }
            let share = free / growing.len() as f32;
            for i in growing {
                size[i] += share.min(growth[i] - size[i]);
            }
        }
        // Flexible: what is left, in proportion; factors adding up to less than 1 take only that share.
        let fr_sum: f32 = tracks.iter().map(|t| if let Track::MinMaxFr0(f) = t { *f } else { 0.0 }).sum();
        if tracks.iter().any(|t| matches!(t, Track::MinMaxFr0(_))) {
            let left = wid - size.iter().sum::<f32>() - gaps_w;
            let fr = left.max(0.0) / fr_sum.max(1.0);
            for (i, t) in tracks.iter().enumerate() {
                if let Track::MinMaxFr0(f) = t {
                    size[i] = f * fr;
                }
            }
        }
        // `justify-content: normal` stretches the auto tracks over what is still free.
        let left2 = wid - size.iter().sum::<f32>() - gaps_w;
        let autos = tracks.iter().filter(|t| matches!(t, Track::Auto)).count();
        if left2 > 1e-4 && autos > 0 {
            for (i, t) in tracks.iter().enumerate() {
                if matches!(t, Track::Auto) {
                    size[i] += left2 / autos as f32;
                }
            }
        }
        let col_x: Vec<f32> = (0..n).map(|i| size[..i].iter().sum::<f32>() + gx * i as f32).collect();
        let total_w = size.iter().sum::<f32>() + gaps_w;

        // Step 7: each item's width, then its height at that width.
        for cell in cells.iter_mut() {
            let cs = self.nodes[cell.id.0].style;
            cell.cw = if cell.all { total_w } else { size[cell.col] };
            let (cw, mx) = (cell.cw, cell.mx());
            let styled_h = if let Dim::Px(h) = cs.h { Some(h) } else { None };
            let iw = if let Some(w) = cs.w.resolve(Some(cw)) {
                w
            } else if let (Some(a), Some(h)) = (cs.aspect, styled_h) {
                h * a
            } else if cell.jalign == Align::Stretch {
                cw - mx
            } else {
                let o = self.layout(cell.id, Req { kw: None, kh: None, aw: Some(cw - mx), ah: None }, false);
                o.w.min(cw - mx)
            };
            let mut iw = iw;
            if let Some(m) = cs.min_w.resolve(Some(cw)) {
                iw = iw.max(m);
            }
            if let Some(m) = cs.max_w.resolve(Some(cw)) {
                iw = iw.min(m);
            }
            cell.iw = iw.max(0.0);
            let kh = styled_h.or(cs.aspect.map(|a| cell.iw / a));
            cell.kh = kh.map(|h| clamp_px(h, cs.min_h, cs.max_h));
            let o = self.layout(
                cell.id,
                Req { kw: Some(cell.iw), kh: cell.kh, aw: Some(cell.iw), ah: None },
                false,
            );
            cell.ih = cell.kh.unwrap_or_else(|| clamp_px(o.h, cs.min_h, cs.max_h));
            cell.baseline = o.baseline;
        }

        // Step 8: row heights, aligning baselines inside a row.
        let mut row_h = vec![0.0f32; rows];
        let mut row_above = vec![0.0f32; rows];
        for (ri, (rh, ra)) in row_h.iter_mut().zip(row_above.iter_mut()).enumerate() {
            let (mut plain, mut above, mut below) = (0.0f32, 0.0f32, 0.0f32);
            for cell in cells.iter().filter(|c| c.row == ri) {
                let outer = cell.ih + cell.my();
                if cell.align == Align::Baseline {
                    let a = cell.mt + cell.baseline;
                    above = above.max(a);
                    below = below.max(outer - a);
                } else {
                    plain = plain.max(outer);
                }
            }
            *rh = plain.max(above + below);
            *ra = above;
        }

        // Step 9: a definite height stretches the rows (`align-content: normal`).
        let gaps_h = gy * rows.saturating_sub(1) as f32;
        if let (Some(h), true) = (ikh, rows > 0) {
            let free = h - row_h.iter().sum::<f32>() - gaps_h;
            if free > 0.0 {
                for rh in row_h.iter_mut() {
                    *rh += free / rows as f32;
                }
            }
        }
        let row_y: Vec<f32> = (0..rows).map(|i| row_h[..i].iter().sum::<f32>() + gy * i as f32).collect();

        // Step 10: final boxes.
        for cell in cells.iter_mut() {
            let cs = self.nodes[cell.id.0].style;
            let rh = row_h[cell.row];
            if cell.align == Align::Stretch && cell.kh.is_none() {
                cell.ih = clamp_px(rh - cell.my(), cs.min_h, cs.max_h);
            }
            let free_y = rh - (cell.ih + cell.my());
            let dy = match cell.align {
                Align::Start | Align::Stretch => cell.mt,
                Align::Center => free_y / 2.0 + cell.mt,
                Align::End => free_y + cell.mt,
                Align::Baseline => row_above[cell.row] - cell.baseline,
            };
            cell.y = row_y[cell.row] + dy;
            let free_x = cell.cw - (cell.iw + cell.mx());
            let dx = match cell.jalign {
                Align::Center => free_x / 2.0 + cell.ml,
                Align::End => free_x + cell.ml,
                _ => cell.ml,
            };
            cell.x = if cell.all { 0.0 } else { col_x[cell.col] } + dx;
        }

        // Step 11: the container's own size and baseline.
        let inner_w = ikw.unwrap_or(total_w);
        let inner_h = ikh.unwrap_or(row_h.iter().sum::<f32>() + gaps_h);
        let (w, h) = (req.kw.unwrap_or(inner_w + px), req.kh.unwrap_or(inner_h + py));
        let baseline = match cells.first() {
            Some(f) => st.pad.top + f.y + f.baseline,
            None => h,
        };

        // Step 12.
        if place {
            for cell in &cells {
                self.layout(
                    cell.id,
                    Req { kw: Some(cell.iw), kh: Some(cell.ih), aw: Some(cell.iw), ah: Some(cell.ih) },
                    true,
                );
                self.nodes[cell.id.0].rect = Rect::new(st.pad.left + cell.x, st.pad.top + cell.y, cell.iw, cell.ih);
            }
        }
        Out { w, h, baseline }
    }
}

/// Clamps by a minimum and a maximum that are lengths; percentages and `auto` do not apply here.
fn clamp_px(v: f32, min: Dim, max: Dim) -> f32 {
    let v = if let Dim::Px(m) = max { v.min(m) } else { v };
    if let Dim::Px(m) = min { v.max(m) } else { v }
}
