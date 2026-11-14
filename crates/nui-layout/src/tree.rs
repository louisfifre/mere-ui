//! The arena of boxes and the public face of the crate. The algorithm itself is in `flex.rs`.

use std::collections::HashMap;

use nui_base::{Rect, Vec2};

use crate::flex::{Out, Req};
use crate::style::{Avail, Measured, Style};

/// An index into the tree's arena, handed out by `Tree::new_node`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

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
