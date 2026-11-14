//! The core: the tree being built, the ids, the pointer rules and the painting.

use std::collections::{HashMap, HashSet};

use nui_base::{Color, DrawList, Event, Rect, Vec2, observe};
use nui_layout::{Avail, Measured, NodeId, Overflow, Position, Style, Tree};

use crate::id::Id;
use crate::input::Input;
use crate::text::{TextLook, Tone, baseline, half_leading, text_width};
use crate::theme::Theme;

/// What the pointer did to a widget this frame.
#[derive(Copy, Clone, Debug)]
pub struct Resp {
    pub id: Id,
    /// Where the widget stood in the last frame; None on its first.
    pub rect: Option<Rect>,
    pub hovered: bool,
    pub held: bool,
    pub clicked: bool,
}

pub(crate) enum Paint {
    None,
    Box { fill: Option<Color>, frame: Option<Color> },
    Text { text: String, look: TextLook, color: Color, underline: bool },
}

pub struct Ui {
    theme: Theme,
    input: Input,
    tree: Tree,
    root: NodeId,
    /// Open containers, innermost last, with their ids.
    stack: Vec<(NodeId, Id)>,
    /// Indexed by `NodeId.0`.
    paints: Vec<Paint>,
    /// Nodes built with a key this frame.
    keyed: Vec<(Id, NodeId)>,
    seen: HashSet<Id>,
    /// Absolute rects of keyed nodes from the LAST `end_frame`: what hit tests use.
    rects: HashMap<Id, Rect>,
    /// The widget the pressed button went down on.
    active: Option<Id>,
    list: DrawList,
    size: Vec2,
}

impl Ui {
    pub fn new(theme: Theme) -> Ui {
        let mut tree = Tree::new();
        let root = tree.new_node(Style::default());
        Ui {
            theme,
            input: Input::default(),
            tree,
            root,
            stack: Vec::new(),
            paints: vec![Paint::None],
            keyed: Vec::new(),
            seen: HashSet::new(),
            rects: HashMap::new(),
            active: None,
            list: DrawList::default(),
            size: Vec2::new(0.0, 0.0),
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, t: Theme) {
        self.theme = t;
    }

    /// Takes an event; true means the screen must be drawn again.
    pub fn event(&mut self, e: &Event) -> bool {
        self.input.feed(e)
    }

    pub fn begin_frame(&mut self, size: Vec2) {
        self.tree = Tree::new();
        // Relative, so that absolute children of the root are legal.
        let style = Style { column: true, position: Position::Relative, ..Style::default() };
        self.root = self.tree.new_node(style);
        self.tree.set_name(self.root, "root");
        self.paints.clear();
        self.paints.push(Paint::None);
        self.stack.clear();
        self.stack.push((self.root, Id::ROOT));
        self.keyed.clear();
        self.seen.clear();
        self.list = DrawList::default();
        self.size = size;
    }

    /// A container called `key` under the open one; `children` builds what is inside.
    pub fn node(&mut self, key: &str, style: Style, children: impl FnOnce(&mut Ui)) -> Id {
        let id = self.key(key);
        self.open(key, id, style, Paint::None);
        children(self);
        self.close();
        id
    }

    /// Where the keyed widget stood in the last frame, in window coordinates.
    pub fn rect(&self, id: Id) -> Option<Rect> {
        self.rects.get(&id).copied()
    }

    pub fn end_frame(&mut self) -> &DrawList {
        self.tree.compute(self.root, self.size);
        let mut abs = vec![Rect::default(); self.paints.len()];
        paint_node(&self.tree, &self.paints, &mut self.list, &mut abs, self.root, Vec2::new(0.0, 0.0));
        self.rects.clear();
        for &(id, n) in &self.keyed {
            self.rects.insert(id, abs[n.0]);
        }
        // A release ends any press; a widget that vanished cannot be released on.
        if self.input.released || self.active.is_some_and(|a| !self.seen.contains(&a)) {
            self.active = None;
        }
        self.input.end_frame();
        &self.list
    }

    /// The laid-out tree as text, for tests (after `end_frame`).
    pub fn dump(&self) -> String {
        self.tree.dump(self.root)
    }

    // ---- helpers for widgets ----

    pub(crate) fn add(&mut self, name: &str, style: Style, paint: Paint) -> NodeId {
        let n = self.tree.new_node(style);
        self.tree.set_name(n, name);
        self.paints.push(paint);
        debug_assert_eq!(n.0 + 1, self.paints.len());
        let parent = self.stack.last().expect("the root is always open").0;
        self.tree.append(parent, n);
        n
    }

    /// Adds a keyed container and leaves it open: what is added next goes inside, until `close`.
    pub(crate) fn open(&mut self, name: &str, id: Id, style: Style, paint: Paint) -> NodeId {
        let n = self.add(name, style, paint);
        self.keyed.push((id, n));
        self.stack.push((n, id));
        n
    }

    pub(crate) fn close(&mut self) {
        self.stack.pop();
    }

    /// A text leaf: its size is arithmetic, and the closure owns what it needs, as the layout wants.
    pub(crate) fn add_text(&mut self, text: &str, look: TextLook, color: Color, underline: bool, style: Style) -> NodeId {
        let owned = text.to_string();
        let width = text_width(text, look.size);
        let n = self.add("text", style, Paint::Text { text: owned, look, color, underline });
        let measured = Measured { size: Vec2::new(width, look.line), baseline: baseline(look) };
        self.tree.set_leaf(n, Box::new(move |_: Avail| measured));
        n
    }

    pub(crate) fn tone_color(&self, tone: Tone) -> Color {
        match tone {
            Tone::Text => self.theme.text,
            Tone::Muted => self.theme.muted,
            Tone::Warn => self.theme.warn,
        }
    }

    pub(crate) fn key(&mut self, key: &str) -> Id {
        let id = self.stack.last().expect("the root is always open").1.child(key);
        if !self.seen.insert(id) {
            observe!(Warn, "id '{key}' used twice in one frame: the second one shares the first one's input");
        }
        id
    }

    /// microui's rules. The hit test uses the rect of the last frame, which is what the person saw and
    /// pointed at.
    pub(crate) fn interact(&mut self, id: Id, disabled: bool) -> Resp {
        let rect = self.rects.get(&id).copied();
        let over = !disabled && matches!((rect, self.input.pos), (Some(r), Some(p)) if r.contains(p));
        if over && self.input.pressed && self.active.is_none() {
            self.active = Some(id);
        }
        let is_active = self.active == Some(id);
        let clicked = is_active && self.input.released && over;
        let held = is_active && self.input.down;
        let hovered = over && (!self.input.down || is_active);
        if clicked {
            observe!(Debug, "click {:x}", id.0);
        }
        Resp { id, rect, hovered, held, clicked }
    }
}

/// Paints `n` and its children depth-first, carrying the parent's absolute origin down, and records each
/// node's absolute rect.
fn paint_node(tree: &Tree, paints: &[Paint], list: &mut DrawList, abs: &mut [Rect], n: NodeId, origin: Vec2) {
    let rel = tree.rect(n);
    let r = Rect::new(origin.x + rel.x, origin.y + rel.y, rel.w, rel.h);
    abs[n.0] = r;
    let clip = tree.style(n).overflow != Overflow::Visible;
    if clip {
        list.push_clip(r);
    }
    match &paints[n.0] {
        Paint::None => {}
        Paint::Box { fill, frame } => {
            if let Some(c) = fill {
                list.fill(r, *c);
            }
            if let Some(c) = frame {
                list.frame(r, 1.0, *c);
            }
        }
        Paint::Text { text, look, color, underline } => {
            list.text_weighted(Vec2::new(r.x, r.y + half_leading(*look)), text.clone(), look.size, *color, look.weight);
            if *underline {
                // text-underline-offset 3px from the baseline (line 152); the 1 px thickness is UNVERIFIED
                // against the browser.
                list.fill(Rect::new(r.x, r.y + baseline(*look) + 3.0, text_width(text, look.size), 1.0), *color);
            }
        }
    }
    for &c in tree.children(n) {
        paint_node(tree, paints, list, abs, c, Vec2::new(r.x, r.y));
    }
    if clip {
        list.pop_clip();
    }
}
