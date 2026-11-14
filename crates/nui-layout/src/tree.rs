//! The arena of boxes and the public face of the crate. The algorithm itself is in `flex.rs`.

use std::collections::HashMap;

use nui_base::{Rect, Vec2};

use crate::style::{Avail, Display, Measured, Style};

/// An index into the tree's arena, handed out by `Tree::new_node`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

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

pub(crate) struct Node {
    pub name: String,
    pub style: Style,
    /// Text, a canvas, a 3D viewport: a box whose size comes from outside rather than from its children.
    pub measure: Option<Box<dyn FnMut(Avail) -> Measured>>,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
    /// Relative to the parent's border box.
    pub rect: Rect,
    pub content: Vec2,
}

#[derive(Default)]
pub struct Tree {
    pub(crate) nodes: Vec<Node>,
    /// Measurements of one `compute`, so that measuring a container and then placing it does not repeat
    /// the work of its whole subtree at every level. Emptied at the start of each `compute`.
    pub(crate) cache: HashMap<(usize, [Option<u32>; 4]), Out>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree::default()
    }

    pub fn new_node(&mut self, style: Style) -> NodeId {
        self.nodes.push(Node {
            name: String::new(),
            style,
            measure: None,
            children: Vec::new(),
            parent: None,
            rect: Rect::default(),
            content: Vec2::default(),
        });
        NodeId(self.nodes.len() - 1)
    }

    /// The name only serves the debug dump and the debug picture.
    pub fn set_name(&mut self, n: NodeId, name: &str) {
        self.nodes[n.0].name = name.to_string();
    }

    pub fn name(&self, n: NodeId) -> &str {
        &self.nodes[n.0].name
    }

    pub fn style(&self, n: NodeId) -> &Style {
        &self.nodes[n.0].style
    }

    pub fn style_mut(&mut self, n: NodeId) -> &mut Style {
        &mut self.nodes[n.0].style
    }

    pub fn children(&self, n: NodeId) -> &[NodeId] {
        &self.nodes[n.0].children
    }

    pub fn append(&mut self, parent: NodeId, child: NodeId) {
        assert!(self.nodes[child.0].parent.is_none(), "a node has one parent");
        self.nodes[child.0].parent = Some(parent);
        self.nodes[parent.0].children.push(child);
    }

    pub fn set_leaf(&mut self, n: NodeId, m: Box<dyn FnMut(Avail) -> Measured>) {
        self.nodes[n.0].measure = Some(m);
    }

    /// Lays the tree out in a box of `size`: the root fills it, and every node gets its rect.
    pub fn compute(&mut self, root: NodeId, size: Vec2) {
        self.cache.clear();
        for n in &mut self.nodes {
            n.rect = Rect::default();
            n.content = Vec2::default();
        }
        let req = Req { kw: Some(size.x), kh: Some(size.y), aw: Some(size.x), ah: Some(size.y) };
        self.layout(root, req, true);
        self.nodes[root.0].rect = Rect::new(0.0, 0.0, size.x, size.y);
    }

    /// Measures one node (nothing is stored) or places it (its children's rects are written, recursively).
    pub(crate) fn layout(&mut self, id: NodeId, req: Req, place: bool) -> Out {
        if !place {
            if let Some(o) = self.cache.get(&(id.0, req.key())) {
                return *o;
            }
        }
        let is_leaf = self.nodes[id.0].measure.is_some();
        let out = if is_leaf {
            self.leaf(id, req)
        } else {
            match self.nodes[id.0].style.display {
                Display::Grid => self.grid(id, req, place),
                _ => self.container(id, req, place),
            }
        };
        if place && !is_leaf {
            // The reach is taken after the absolute boxes are placed: they count in a scroll area's content.
            let r = self.reach(id, out.w, out.h);
            self.nodes[id.0].content = r;
        }
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

    /// How far the children and their margins reach, padding included, never less than the box itself.
    fn reach(&self, id: NodeId, w: f32, h: f32) -> Vec2 {
        let st = &self.nodes[id.0].style;
        let mut r = Vec2::new(w, h);
        for &c in &self.nodes[id.0].children {
            let n = &self.nodes[c.0];
            if n.style.display == Display::None {
                continue;
            }
            r.x = r.x.max(n.rect.x + n.rect.w + n.style.margin.right.px() + st.pad.right);
            r.y = r.y.max(n.rect.y + n.rect.h + n.style.margin.bottom.px() + st.pad.bottom);
        }
        r
    }

    /// Relative to the parent's border box (the root's is relative to the window).
    pub fn rect(&self, n: NodeId) -> Rect {
        self.nodes[n.0].rect
    }

    /// Relative to the root: the parents' offsets added up.
    pub fn abs_rect(&self, n: NodeId) -> Rect {
        let mut r = self.nodes[n.0].rect;
        let mut at = self.nodes[n.0].parent;
        while let Some(p) = at {
            let pr = self.nodes[p.0].rect;
            r.x += pr.x;
            r.y += pr.y;
            at = self.nodes[p.0].parent;
        }
        r
    }

    /// How far the children and their margins reach, padding included, never less than the box itself:
    /// what a scroll area compares with its own size.
    pub fn content_size(&self, n: NodeId) -> Vec2 {
        self.nodes[n.0].content
    }

    /// One line per node, indented by depth: `name x y w h`, relative to the parent. Readable in a test's
    /// failure message and in the log.
    pub fn dump(&self, root: NodeId) -> String {
        let mut out = String::new();
        self.dump_into(root, 0, &mut out);
        out
    }

    fn dump_into(&self, n: NodeId, depth: usize, out: &mut String) {
        let node = &self.nodes[n.0];
        let name = if node.name.is_empty() { "#" } else { node.name.as_str() };
        let r = node.rect;
        out.push_str(&format!(
            "{}{} {} {} {} {}\n",
            "  ".repeat(depth),
            name,
            num(r.x),
            num(r.y),
            num(r.w),
            num(r.h)
        ));
        for &c in &node.children {
            self.dump_into(c, depth + 1, out);
        }
    }
}

/// Two decimals at most, without a trailing zero: `296`, `31.5`.
fn num(v: f32) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
