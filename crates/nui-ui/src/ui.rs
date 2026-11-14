//! The core: the tree being built, the ids, the pointer rules and the painting.

use std::collections::{HashMap, HashSet};

use nui_base::{Clipboard, Color, DrawList, Event, Rect, Vec2, observe};
use nui_layout::{Avail, Dim, Measured, NodeId, Overflow, Position, Style, Tree};

use crate::edit::MemoryClipboard;
use crate::field::{FieldPaint, Focus, Typed, paint_field};
use crate::id::Id;
use crate::input::Input;
use crate::scroll::{ScrollMetrics, ScrollPaint, thumb_rect};
use crate::text::{TextLook, Tone, baseline, ellipsize, half_leading};
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
    /// `ellipsis` cuts the text to its box with `ellipsize` when painting, the box being known only then.
    Text { text: String, look: TextLook, color: Color, underline: bool, ellipsis: bool },
    /// A text field: its own frame, clip, selection, text and caret (field.rs).
    Field(Box<FieldPaint>),
    /// A scroll area: it offsets its children and draws its thumb and fade after them (scroll.rs).
    Scroll(ScrollPaint),
}

pub struct Ui {
    theme: Theme,
    pub(crate) input: Input,
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
    /// The part of each keyed node's rect that was visible in the LAST frame (cut by every scrolling or
    /// clipping ancestor): what the pointer can reach. A row scrolled out of sight takes no hover.
    hits: HashMap<Id, Rect>,
    /// The widget the pressed button went down on.
    pub(crate) active: Option<Id>,
    /// How far below the top of the scrollbar thumb the pointer took hold of it. Only one drag at a time,
    /// like the focus, so one value serves every scroll area.
    pub(crate) grab: Option<f32>,
    /// What each scroll area measured in the LAST frame.
    pub(crate) scrolls: HashMap<Id, ScrollMetrics>,
    list: DrawList,
    size: Vec2,
    /// Keys and characters since the last frame, in arrival order, for the focused field.
    pub(crate) typed: Vec<Typed>,
    /// The one field with the keyboard, if any. At most one, so no per-field state map.
    pub(crate) focus: Option<Focus>,
    /// A focus another field took away during this frame, kept until its own field is built and can be told.
    pub(crate) lost: Option<Focus>,
    /// The fields built this frame, in build order: the order Tab walks.
    pub(crate) order: Vec<Id>,
    /// A Tab that left a field: its id and whether it was Shift+Tab. Applied after painting.
    pub(crate) tab: Option<(Id, bool)>,
    pub(crate) redraw: bool,
    pub(crate) clipboard: Box<dyn Clipboard>,
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
            hits: HashMap::new(),
            active: None,
            grab: None,
            scrolls: HashMap::new(),
            list: DrawList::default(),
            size: Vec2::new(0.0, 0.0),
            typed: Vec::new(),
            focus: None,
            lost: None,
            order: Vec::new(),
            tab: None,
            redraw: false,
            clipboard: Box::new(MemoryClipboard::default()),
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, t: Theme) {
        self.theme = t;
    }

    /// Takes an event; true means the screen must be drawn again. Keys and characters wait for the focused
    /// field, which reads them when it is built.
    pub fn event(&mut self, e: &Event) -> bool {
        match *e {
            Event::Key { key, mods } => {
                self.typed.push(Typed::Key(key, mods));
                true
            }
            Event::Char { ch } => {
                self.typed.push(Typed::Char(ch));
                true
            }
            _ => self.input.feed(e),
        }
    }

    /// The field that has the keyboard.
    pub fn focused(&self) -> Option<Id> {
        self.focus.as_ref().map(|f| f.id)
    }

    /// True when the frame just ended changed something that only the next frame shows (Tab moved the focus
    /// after the picture was made): the application should draw once more.
    pub fn want_redraw(&self) -> bool {
        self.redraw
    }

    /// Where Ctrl+C, X and V go. A memory clipboard until the application gives the system's.
    pub fn set_clipboard(&mut self, c: Box<dyn Clipboard>) {
        self.clipboard = c;
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
        self.order.clear();
        self.redraw = false;
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
        let mut vis = vec![Rect::default(); self.paints.len()];
        let window = Rect::new(0.0, 0.0, self.size.x, self.size.y);
        self.scrolls.clear();
        let mut painter = Painter {
            tree: &self.tree,
            paints: &self.paints,
            list: &mut self.list,
            abs: &mut abs,
            vis: &mut vis,
            scrolls: &mut self.scrolls,
        };
        painter.node(self.root, Vec2::new(0.0, 0.0), window);
        self.rects.clear();
        self.hits.clear();
        for &(id, n) in &self.keyed {
            self.rects.insert(id, abs[n.0]);
            self.hits.insert(id, vis[n.0]);
        }
        // A release ends any press; a widget that vanished cannot be released on.
        if self.input.released || self.active.is_some_and(|a| !self.seen.contains(&a)) {
            self.active = None;
        }
        if self.active.is_none() {
            self.grab = None;
        }
        // The gutter is reserved from what the last frame measured, so a list that just grew past its box
        // (or shrank under it) is one frame late: ask for one more.
        for (id, m) in &self.scrolls {
            if m.overflow != m.gutter {
                observe!(Debug, "scroll {:x}: overflow {}, one more frame", id.0, m.overflow);
                self.redraw = true;
            }
        }
        self.settle_focus();
        self.typed.clear();
        self.lost = None;
        self.input.end_frame();
        &self.list
    }

    /// What the frame decided about the focus once every field was built: a Tab moves it to the next field
    /// built (wrapping), and a focused field that was not built any more loses it.
    fn settle_focus(&mut self) {
        if let Some((from, back)) = self.tab.take() {
            let n = self.order.len();
            match self.order.iter().position(|&i| i == from) {
                Some(at) if n > 0 => {
                    let to = self.order[if back { (at + n - 1) % n } else { (at + 1) % n }];
                    observe!(Debug, "tab {} from {:x} to {:x}", if back { "back" } else { "forward" }, from.0, to.0);
                    // The new field builds its state on its first frame, with all its text selected.
                    self.focus = Some(Focus { id: to, state: None, select_all: true });
                    self.redraw = true;
                }
                _ => observe!(Warn, "tab from {:x} found no field to go to", from.0),
            }
        } else if let Some(f) = &self.focus {
            if !self.order.contains(&f.id) {
                observe!(Debug, "focus lost: {:x} was not built this frame", f.id.0);
                self.focus = None;
            }
        }
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

    /// Changes a node's style after it was added: a widget learns its state (a fold's click) only once its
    /// container is already open.
    pub(crate) fn restyle(&mut self, n: NodeId, style: Style) {
        *self.tree.style_mut(n) = style;
    }

    /// Replaces a node's paint once the widget knows its final state (a scroll area's offset, after its body).
    pub(crate) fn set_paint(&mut self, n: NodeId, paint: Paint) {
        self.paints[n.0] = paint;
    }

    pub(crate) fn close(&mut self) {
        self.stack.pop();
    }

    /// A keyed leaf whose size is given: a text field.
    pub(crate) fn add_keyed_leaf(&mut self, name: &str, id: Id, style: Style, paint: Paint, measured: Measured) -> NodeId {
        let n = self.add(name, style, paint);
        self.keyed.push((id, n));
        self.tree.set_leaf(n, Box::new(move |_: Avail| measured));
        n
    }

    /// A text leaf: its size is arithmetic, and the closure owns what it needs, as the layout wants.
    pub(crate) fn add_text(&mut self, text: &str, look: TextLook, color: Color, underline: bool, style: Style) -> NodeId {
        self.add_text_with(text, look, color, underline, false, style)
    }

    /// A text leaf that may shrink below its text (`min-width: 0`) and is cut with an ellipsis when painted.
    pub(crate) fn add_ellipsis(&mut self, text: &str, look: TextLook, color: Color, underline: bool) -> NodeId {
        let style = Style { min_w: Dim::Px(0.0), ..Style::default() };
        self.add_text_with(text, look, color, underline, true, style)
    }

    fn add_text_with(&mut self, text: &str, look: TextLook, color: Color, underline: bool, ellipsis: bool, style: Style) -> NodeId {
        let owned = text.to_string();
        let width = look.width(text);
        let n = self.add("text", style, Paint::Text { text: owned, look, color, underline, ellipsis });
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
        let reach = self.hits.get(&id).copied();
        let over = !disabled && matches!((reach, self.input.pos), (Some(r), Some(p)) if r.contains(p));
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

/// What painting needs besides the node: the tree and paints to read, the list and the two rect tables to
/// write. Gathered here so the recursion carries only the node, its origin and the clip.
struct Painter<'a> {
    tree: &'a Tree,
    paints: &'a [Paint],
    list: &'a mut DrawList,
    /// Absolute rect of every node.
    abs: &'a mut [Rect],
    /// `abs` cut to the clips of the ancestors: the part that can be seen and pointed at.
    vis: &'a mut [Rect],
    /// Filled for every scroll area painted.
    scrolls: &'a mut HashMap<Id, ScrollMetrics>,
}

impl Painter<'_> {
    /// Paints `n` and its children depth-first, carrying the parent's absolute origin and the visible clip
    /// down, and records each node's absolute and visible rects.
    fn node(&mut self, n: NodeId, origin: Vec2, clip: Rect) {
        let rel = self.tree.rect(n);
        let r = Rect::new(origin.x + rel.x, origin.y + rel.y, rel.w, rel.h);
        self.abs[n.0] = r;
        self.vis[n.0] = r.intersect(&clip);
        let clips = self.tree.style(n).overflow != Overflow::Visible;
        let child_clip = if clips { r.intersect(&clip) } else { clip };
        if clips {
            self.list.push_clip(r);
        }
        // A copy of the shared slice, so that what is matched does not keep `self` borrowed.
        let paints = self.paints;
        let mut kid_origin = Vec2::new(r.x, r.y);
        let mut scrolled = None;
        match &paints[n.0] {
            Paint::None => {}
            Paint::Scroll(s) => {
                let content_h = self.tree.content_size(n).y;
                let max = (content_h - r.h).max(0.0);
                let off = s.y.clamp(0.0, max);
                // The children are laid out from the top; moving their origin puts the offset into the
                // rects recorded for them, so hit tests see where they are drawn.
                kid_origin.y -= off;
                scrolled = Some((s, content_h, max, off));
            }
            Paint::Box { fill, frame } => {
                if let Some(c) = fill {
                    self.list.fill(r, *c);
                }
                if let Some(c) = frame {
                    self.list.frame(r, 1.0, *c);
                }
            }
            Paint::Field(f) => paint_field(self.list, f, r, self.tree.style(n).pad),
            Paint::Text { text, look, color, underline, ellipsis } => {
                let y = r.y + half_leading(*look);
                let drawn = if *ellipsis { ellipsize(text, *look, r.w) } else { text.clone() };
                // A box narrower than the ellipsis cell still shows one: cut it to the box.
                let cut = *ellipsis && drawn == "\u{2026}" && r.w < look.step();
                if cut {
                    self.list.push_clip(r);
                }
                if look.tracking == 0.0 {
                    self.list.text_weighted(Vec2::new(r.x, y), drawn.clone(), look.size, *color, look.weight);
                } else {
                    // Plex Mono is monospaced and has no kerning, so one glyph per step equals a spaced string.
                    let mut buf = [0u8; 4];
                    for (i, ch) in drawn.chars().enumerate() {
                        let x = r.x + i as f32 * look.step();
                        self.list.text_weighted(Vec2::new(x, y), &*ch.encode_utf8(&mut buf), look.size, *color, look.weight);
                    }
                }
                if cut {
                    self.list.pop_clip();
                }
                if *underline {
                    // text-underline-offset 3px from the baseline (line 152); the 1 px thickness is UNVERIFIED
                    // against the browser.
                    let w = look.width(&drawn).min(r.w);
                    self.list.fill(Rect::new(r.x, r.y + baseline(*look) + 3.0, w, 1.0), *color);
                }
            }
        }
        // CSS paints positioned boxes (z-index auto) after the in-flow ones: the pressed tab's hairline must
        // cover the grey one of the neighbour it overlaps.
        let tree = self.tree;
        let kids = tree.children(n);
        let flow = |c: &&NodeId| tree.style(**c).position == Position::Static;
        for &c in kids.iter().filter(flow) {
            self.node(c, kid_origin, child_clip);
        }
        for &c in kids.iter().filter(|c| !flow(c)) {
            self.node(c, kid_origin, child_clip);
        }
        if let Some((s, content_h, max, off)) = scrolled {
            let overflow = content_h > r.h + 0.5;
            // The thumb, then the mask over it: the CSS mask fades the scrollbar with the content.
            if overflow {
                self.list.fill(thumb_rect(r, content_h, max, off), s.thumb);
            }
            // The ground is opaque behind the list, so laying it over in 1 px bands of rising opacity equals
            // the page's `mask-image` gradient, quantised per band.
            for i in 0..s.fade as usize {
                let band = Rect::new(r.x, r.y + r.h - s.fade + i as f32, r.w, 1.0);
                self.list.fill(band, s.ground.with_alpha((i as f32 + 0.5) / s.fade));
            }
            self.scrolls.insert(
                s.id,
                ScrollMetrics { visible: self.vis[n.0], view: r, content_h, max, off, overflow, gutter: s.gutter },
            );
        }
        if clips {
            self.list.pop_clip();
        }
    }
}
