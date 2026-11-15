//! The core: the tree being built, the ids, the pointer rules and the painting.

use std::collections::{HashMap, HashSet};

use nui_base::{Clipboard, Cmd, Color, Cursor, DrawList, Event, Key, Rect, Vec2, observe};
use nui_layout::{Avail, Dim, Measured, NodeId, Overflow, Position, Style, Tree};

use crate::edit::MemoryClipboard;
use crate::field::{FieldPaint, Focus, Typed, paint_field};
use crate::id::Id;
use crate::input::Input;
use crate::menu::Menu;
use crate::scroll::{ArrowDir, BarHot, ScrollMetrics, ScrollPaint, arrow_rects, arrows_shown, chevron, thumb_rect, track_rect};
use crate::text::{TextLook, Tone, baseline, ellipsize, half_leading, wrap_lines, wrap_lines_after};
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
    /// The left button went down on this widget in this frame: this frame made it the active one. A drag
    /// starts here, and a widget that tells a click from a drag needs the edge, not just held.
    pub pressed: bool,
}

pub(crate) enum Paint {
    None,
    Box { fill: Option<Color>, frame: Option<Color> },
    /// `ellipsis` cuts the text to its box with `ellipsize` when painting, the box being known only then.
    /// `wrap` breaks it into lines of the box's width (`wrap_lines`); neither underline nor ellipsis then.
    Text { text: String, look: TextLook, color: Color, underline: bool, strike: bool, ellipsis: bool, wrap: bool },
    /// One flowing run: `lead` in its colour, `gap` px, then `text` wrapped on from there (`wrap_lines_after`).
    Flow { lead: String, lead_color: Color, gap: f32, text: String, look: TextLook, color: Color },
    /// The running generate button: its 1 px frame, then the bar from the padding box's left, `share` of its width.
    Running { frame: Color, bar: Color, share: f32 },
    /// The select's arrow: 8 x 5 px, painted as filled 1 px rows, as Chrome's is at 1x.
    Chevron { color: Color },
    /// A text field: its own frame, clip, selection, text and caret (field.rs).
    Field(Box<FieldPaint>),
    /// A scroll area: it offsets its children and draws its thumb and fade after them (scroll.rs).
    Scroll(ScrollPaint),
    /// Ready-made commands in the node's own coordinates (canvas): moved to the node's place when painted.
    Cmds(Vec<Cmd>),
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
    /// The caret of the focused text field as painted this frame: where the input method may write.
    pub(crate) spot: Option<Rect>,
    pub(crate) list: DrawList,
    pub(crate) size: Vec2,
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
    /// The pointer's shape for the last frame built: the innermost widget under it that asks for one.
    cursor: Cursor,
    /// The application's time in seconds, which tooltips are timed by.
    clock: f64,
    /// The titled widget under the pointer this frame (the last built), and its text.
    tip_candidate: Option<(Id, String)>,
    /// The candidate of the last frame and since when it has been one.
    tip_id: Option<Id>,
    tip_since: f64,
    /// A press hid the tip; it stays hidden until another widget becomes the candidate.
    tip_hidden: bool,
    /// The open select menu, if any (menu.rs), the row a press just picked, and the box built this frame
    /// that the menu hangs from.
    pub(crate) menu: Option<Menu>,
    pub(crate) menu_pick: Option<(Id, usize)>,
    pub(crate) menu_node: Option<NodeId>,
}

/// How long the pointer rests on a titled widget before its tip shows (the browsers' own is about this).
pub const TIP_DELAY: f64 = 0.5;
/// A tip wraps at this width.
const TIP_WRAP: f32 = 360.0;

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
            spot: None,
            list: DrawList::default(),
            size: Vec2::new(0.0, 0.0),
            typed: Vec::new(),
            focus: None,
            lost: None,
            order: Vec::new(),
            tab: None,
            redraw: false,
            clipboard: Box::new(MemoryClipboard::default()),
            cursor: Cursor::Arrow,
            clock: 0.0,
            tip_candidate: None,
            tip_id: None,
            tip_since: 0.0,
            tip_hidden: false,
            menu: None,
            menu_pick: None,
            menu_node: None,
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
            // An open menu takes Escape before any field does.
            Event::Key { key: Key::Escape, .. } if self.menu.is_some() => {
                self.menu = None;
                true
            }
            Event::Key { key, mods } => {
                self.typed.push(Typed::Key(key, mods));
                true
            }
            Event::Char { ch } => {
                self.typed.push(Typed::Char(ch));
                true
            }
            Event::Ime(ref i) => {
                self.typed.push(Typed::Ime(i.clone()));
                true
            }
            _ => self.input.feed(e),
        }
    }

    /// Where the input method may write, as of the last frame: the caret of the focused text field in dip, or
    /// the start of its composition. None when no text field has the keyboard (a number field neither).
    pub fn ime_spot(&self) -> Option<Rect> {
        self.spot
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
        self.cursor = Cursor::Arrow;
        self.tip_candidate = None;
        self.list = DrawList::default();
        self.size = size;
        self.menu_node = None;
        self.menu_begin_frame();
    }

    /// The application's time in seconds. Tooltips are timed by it, so tests set it by hand.
    pub fn set_clock(&mut self, seconds: f64) {
        self.clock = seconds;
    }

    /// The pointer's shape for the frame just built: what the widgets under it asked for, else the arrow.
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// For the application's canvases (the map, the cut), which know their own shape.
    pub fn want_cursor(&mut self, c: Cursor) {
        self.cursor = c;
    }

    /// The page's `title` attribute for the widget `id` built this frame; empty text means none. The tip
    /// shows when the pointer has rested on the widget for `TIP_DELAY`; a disabled widget has one too.
    pub fn title(&mut self, id: Id, text: &str) {
        if text.is_empty() {
            return;
        }
        // Judged on the last frame's visible rect, as the pointer is everywhere else.
        if let (Some(r), Some(p)) = (self.hits.get(&id), self.input.pos) {
            if r.contains(p) {
                self.tip_candidate = Some((id, text.to_string()));
            }
        }
    }

    /// Seconds until a tip is due, while one waits; None when none waits or it shows already.
    pub fn wake_in(&self) -> Option<f32> {
        if self.tip_candidate.is_none() || self.tip_hidden {
            return None;
        }
        let left = TIP_DELAY - (self.clock - self.tip_since);
        (left > 0.0).then_some(left as f32)
    }

    /// A container called `key` under the open one; `children` builds what is inside.
    pub fn node(&mut self, key: &str, style: Style, children: impl FnOnce(&mut Ui)) -> Id {
        let id = self.key(key);
        self.open(key, id, style, Paint::None);
        children(self);
        self.close();
        id
    }

    /// A box of the given style that the application draws itself: draw gets the pointer's answer for it
    /// and returns commands in the box's own coordinates (0,0 is its top-left corner). The commands are not
    /// clipped to the box unless they push a clip themselves, so ticks may hang outside it. draw runs
    /// before the box is laid out, so it judges the pointer against the last frame's rect (Resp::rect).
    pub fn canvas(&mut self, key: &str, style: Style, draw: impl FnOnce(&mut Ui, &Resp) -> Vec<Cmd>) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, false, Cursor::Arrow);
        let cmds = draw(self, &resp);
        self.open(key, id, style, Paint::Cmds(cmds));
        self.close();
        resp
    }

    /// A container that tells its body where the pointer is across it: the pointer's x as a share of the
    /// container's width, judged against the LAST frame's rect (what the person saw), before the children are
    /// built. None when the pointer is outside the visible part of the rect, when another widget holds the
    /// press, or on the container's first frame. It never becomes active and never takes a click: the
    /// widgets inside keep theirs.
    pub fn hover_area(&mut self, key: &str, style: Style, body: impl FnOnce(&mut Ui, Option<f32>)) -> Id {
        let id = self.key(key);
        let share = match (self.rects.get(&id), self.hits.get(&id), self.input.pos) {
            (Some(r), Some(seen), Some(p)) if r.w > 0.0 && seen.contains(p) && self.active.is_none() => Some((p.x - r.x) / r.w),
            _ => None,
        };
        self.open(key, id, style, Paint::None);
        body(self, share);
        self.close();
        id
    }

    /// A container painted `lit` while the pointer is over its visible rect of the last frame: the launcher's
    /// `.profile:hover` (a row that lights and says nothing can be clicked). Unlike `hover_area` it stays lit
    /// while a press is held, and like it takes no click: the widgets inside keep theirs.
    pub fn hover_box(&mut self, key: &str, style: Style, lit: Color, body: impl FnOnce(&mut Ui)) -> Id {
        let id = self.key(key);
        let over = !self.under_menu() && matches!((self.hits.get(&id), self.input.pos), (Some(seen), Some(p)) if seen.contains(p));
        let paint = if over { Paint::Box { fill: Some(lit), frame: None } } else { Paint::None };
        self.open(key, id, style, paint);
        body(self);
        self.close();
        id
    }

    /// The application's time in seconds, as last given to `set_clock`.
    pub fn clock(&self) -> f64 {
        self.clock
    }

    /// Where the pointer is, in window coordinates; None until it has been seen.
    pub fn pointer(&self) -> Option<Vec2> {
        self.input.pos
    }

    /// The wheel notches of this frame (positive away from the user) and where the pointer was, when it was
    /// over the visible part of widget id in the last frame. Takes them: a second call, or a scroll area
    /// closing later, gets nothing, as with nested scroll areas.
    pub fn take_wheel(&mut self, id: Id) -> Option<(f32, Vec2)> {
        let over = self.hits.get(&id).copied();
        match (self.input.wheel_pos, over) {
            (Some(p), Some(r)) if self.input.wheel != 0.0 && r.contains(p) => {
                let n = self.input.wheel;
                self.input.wheel = 0.0;
                Some((n, p))
            }
            _ => None,
        }
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
        self.spot = None;
        let mut painter = Painter {
            tree: &self.tree,
            paints: &self.paints,
            list: &mut self.list,
            abs: &mut abs,
            vis: &mut vis,
            scrolls: &mut self.scrolls,
            spot: &mut self.spot,
        };
        painter.node(self.root, Vec2::new(0.0, 0.0), window);
        self.paint_menu(&abs);
        self.paint_tip();
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

    /// The tip, last of all so it lies over everything: a framed box under the pointer.
    fn paint_tip(&mut self) {
        let now = self.tip_candidate.as_ref().map(|c| c.0);
        if now != self.tip_id {
            self.tip_id = now;
            self.tip_since = self.clock;
            self.tip_hidden = false;
        }
        if self.input.pressed {
            self.tip_hidden = true;
        }
        let (Some((_, text)), Some(at)) = (self.tip_candidate.clone(), self.input.pos) else { return };
        if self.tip_hidden || self.clock - self.tip_since < TIP_DELAY {
            return;
        }
        // OUR LOOK in the page's tokens: the browser's tip is the system's.
        let t = self.theme;
        let look = TextLook::small(Tone::Text);
        let lines = wrap_lines(&text, look, TIP_WRAP);
        let w = lines.iter().map(|l| look.width(l)).fold(0.0, f32::max) + 14.0;
        let h = lines.len() as f32 * look.line + 8.0;
        let x = at.x.min(self.size.x - w).max(0.0);
        let below = at.y + 20.0;
        let y = if below + h <= self.size.y { below } else { (at.y - h - 4.0).max(0.0) };
        let r = Rect::new(x, y, w, h);
        self.list.fill(r, t.ground);
        self.list.frame(r, 1.0, t.edge);
        for (i, line) in lines.iter().enumerate() {
            draw_line(&mut self.list, line, look, t.text, x + 7.0, y + 4.0 + i as f32 * look.line + half_leading(look));
        }
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
        self.add_text_with(text, look, color, underline, false, false, style)
    }

    /// A text leaf whose word may be struck through (`text-decoration: line-through`).
    pub(crate) fn add_text_struck(&mut self, text: &str, look: TextLook, color: Color, strike: bool, style: Style) -> NodeId {
        self.add_text_with(text, look, color, false, strike, false, style)
    }

    /// A text leaf that may shrink below its text (`min-width: 0`) and is cut with an ellipsis when painted.
    pub(crate) fn add_ellipsis(&mut self, text: &str, look: TextLook, color: Color, underline: bool) -> NodeId {
        let style = Style { min_w: Dim::Px(0.0), ..Style::default() };
        self.add_text_with(text, look, color, underline, false, true, style)
    }

    fn add_text_with(&mut self, text: &str, look: TextLook, color: Color, underline: bool, strike: bool, ellipsis: bool, style: Style) -> NodeId {
        let owned = text.to_string();
        let width = look.width(text);
        let n = self.add("text", style, Paint::Text { text: owned, look, color, underline, strike, ellipsis, wrap: false });
        let measured = Measured { size: Vec2::new(width, look.line), baseline: baseline(look) };
        self.tree.set_leaf(n, Box::new(move |_: Avail| measured));
        n
    }

    /// A text leaf that breaks into lines at its box's width. Its measure answers the three questions the
    /// layout asks: min-content (`Some(0)`: the widest word, one word to a line), a given width, and none
    /// (everything on one line). Nothing to show measures 0 x 0.
    pub(crate) fn add_paragraph(&mut self, text: &str, look: TextLook, color: Color, style: Style) -> NodeId {
        let owned = text.to_string();
        let n = self.add("paragraph", style, Paint::Text { text: owned.clone(), look, color, underline: false, strike: false, ellipsis: false, wrap: true });
        let base = baseline(look);
        self.tree.set_leaf(
            n,
            Box::new(move |a: Avail| {
                let words: Vec<&str> = owned.split_whitespace().collect();
                if words.is_empty() {
                    return Measured::default();
                }
                let size = match a.w {
                    Some(w) if w == 0.0 => {
                        let widest = words.iter().map(|x| look.width(x)).fold(0.0, f32::max);
                        Vec2::new(widest, words.len() as f32 * look.line)
                    }
                    Some(w) => {
                        let lines = wrap_lines(&owned, look, w);
                        let widest = lines.iter().map(|l| look.width(l)).fold(0.0, f32::max);
                        Vec2::new(widest, lines.len() as f32 * look.line)
                    }
                    None => Vec2::new(look.width(&words.join(" ")), look.line),
                };
                Measured { size, baseline: base }
            }),
        );
        n
    }

    /// A label and a text that flows on after it, wrapping back to the left edge: `.saved .path`. The leaf's
    /// measure answers as a paragraph's does; the label always takes the first line.
    pub(crate) fn add_flow(&mut self, lead: &str, lead_color: Color, gap: f32, text: &str, look: TextLook, color: Color, style: Style) -> NodeId {
        let paint = Paint::Flow { lead: lead.to_string(), lead_color, gap, text: text.to_string(), look, color };
        let n = self.add("flow", style, paint);
        let (lead, text) = (lead.to_string(), text.to_string());
        let base = baseline(look);
        self.tree.set_leaf(
            n,
            Box::new(move |a: Avail| {
                let lead_w = look.width(&lead) + gap;
                let size = match a.w {
                    Some(w) if w == 0.0 => {
                        let widest = text.split_whitespace().map(|x| look.width(x)).fold(lead_w, f32::max);
                        Vec2::new(widest, look.line)
                    }
                    Some(w) => {
                        let lines = wrap_lines_after(&text, look, w, lead_w);
                        let widest = lines.iter().enumerate().map(|(i, l)| look.width(l) + if i == 0 { lead_w } else { 0.0 }).fold(lead_w, f32::max);
                        Vec2::new(widest, lines.len().max(1) as f32 * look.line)
                    }
                    None => Vec2::new(lead_w + look.width(&text.split_whitespace().collect::<Vec<_>>().join(" ")), look.line),
                };
                Measured { size, baseline: base }
            }),
        );
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
    pub(crate) fn interact(&mut self, id: Id, disabled: bool, cursor: Cursor) -> Resp {
        let rect = self.rects.get(&id).copied();
        let reach = self.hits.get(&id).copied();
        let over = !disabled && !self.under_menu() && matches!((reach, self.input.pos), (Some(r), Some(p)) if r.contains(p));
        let pressed = over && self.input.pressed && self.active.is_none();
        if pressed {
            self.active = Some(id);
        }
        let is_active = self.active == Some(id);
        let clicked = is_active && self.input.released && over;
        let held = is_active && self.input.down;
        let hovered = over && (!self.input.down || is_active);
        if clicked {
            observe!(Debug, "click {:x}", id.0);
        }
        // The last widget built wins, which is the innermost. A held one keeps its shape wherever the pointer
        // goes; a disabled one asks for none (`button:disabled { cursor: default }`).
        if cursor != Cursor::Arrow && !disabled && (hovered || held) {
            self.cursor = cursor;
        }
        Resp { id, rect, hovered, held, clicked, pressed }
    }
}

/// One line of text at `x`, `y` (the top of its content box).
pub(crate) fn draw_line(list: &mut DrawList, s: &str, look: TextLook, color: Color, x: f32, y: f32) {
    if look.tracking == 0.0 {
        list.text_weighted(Vec2::new(x, y), s, look.size, color, look.weight);
    } else {
        // Plex Mono is monospaced and has no kerning, so one glyph per step equals a spaced string.
        let mut buf = [0u8; 4];
        for (i, ch) in s.chars().enumerate() {
            list.text_weighted(Vec2::new(x + i as f32 * look.step(), y), &*ch.encode_utf8(&mut buf), look.size, color, look.weight);
        }
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
    /// Set by the focused text field painted.
    spot: &'a mut Option<Rect>,
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
            Paint::Running { frame, bar, share } => {
                self.list.frame(r, 1.0, *frame);
                self.list.fill(Rect::new(r.x + 1.0, r.y + 1.0, share.clamp(0.0, 1.0) * (r.w - 2.0), r.h - 2.0), *bar);
            }
            Paint::Field(f) => {
                if let Some(s) = paint_field(self.list, f, r, self.tree.style(n).pad) {
                    *self.spot = Some(s);
                }
            }
            Paint::Cmds(cmds) => {
                let d = Vec2::new(r.x, r.y);
                self.list.cmds.extend(cmds.iter().map(|c| c.translated(d)));
            }
            Paint::Chevron { color } => {
                // [from, to) of each 1 px row, relative to the leaf's top-left.
                const ROWS: [&[(f32, f32)]; 5] = [&[(0.0, 2.0), (6.0, 8.0)], &[(0.0, 3.0), (5.0, 8.0)], &[(1.0, 7.0)], &[(2.0, 6.0)], &[(3.0, 5.0)]];
                for (i, row) in ROWS.iter().enumerate() {
                    for &(a, b) in *row {
                        self.list.fill(Rect::new(r.x + a, r.y + i as f32, b - a, 1.0), *color);
                    }
                }
            }
            Paint::Flow { lead, lead_color, gap, text, look, color } => {
                let lead_w = look.width(lead) + gap;
                let top = r.y + half_leading(*look);
                draw_line(self.list, lead, *look, *lead_color, r.x, top);
                for (i, line) in wrap_lines_after(text, *look, r.w, lead_w).iter().enumerate() {
                    let x = if i == 0 { r.x + lead_w } else { r.x };
                    draw_line(self.list, line, *look, *color, x, top + i as f32 * look.line);
                }
            }
            Paint::Text { text, look, color, underline, strike, ellipsis, wrap } => {
                if *wrap {
                    for (i, line) in wrap_lines(text, *look, r.w).iter().enumerate() {
                        let y = r.y + i as f32 * look.line + half_leading(*look);
                        draw_line(self.list, line, *look, *color, r.x, y);
                    }
                } else {
                    let y = r.y + half_leading(*look);
                    let drawn = if *ellipsis { ellipsize(text, *look, r.w) } else { text.clone() };
                    // A box narrower than the ellipsis cell still shows one: cut it to the box.
                    let cut = *ellipsis && drawn == "\u{2026}" && r.w < look.step();
                    if cut {
                        self.list.push_clip(r);
                    }
                    draw_line(self.list, &drawn, *look, *color, r.x, y);
                    if cut {
                        self.list.pop_clip();
                    }
                    if *underline {
                        // text-underline-offset 3px from the baseline (line 152); the 1 px thickness is UNVERIFIED
                        // against the browser.
                        let w = look.width(&drawn).min(r.w);
                        self.list.fill(Rect::new(r.x, r.y + baseline(*look) + 3.0, w, 1.0), *color);
                    }
                    if *strike {
                        // A line-through sits about a third of the font size above the baseline; the exact
                        // place and thickness are UNVERIFIED against Chrome.
                        let w = look.width(&drawn).min(r.w);
                        let y = r.y + baseline(*look) - (0.3 * look.size).round();
                        self.list.fill(Rect::new(r.x, y, w, 1.0), *color);
                    }
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

            // The ground is opaque behind the list, so laying it over in 1 px bands of rising opacity equals
            // the page's `mask-image` gradient, quantised per band.
            for i in 0..s.fade as usize {
                let band = Rect::new(r.x, r.y + r.h - s.fade + i as f32, r.w, 1.0);
                self.list.fill(band, s.ground.with_alpha((i as f32 + 0.5) / s.fade));
            }
            // The bar comes after the mask: it lives in the gutter, beside the content, so the fade has no
            // business with it, and the bottom arrow must stay plain.
            if overflow {
                let hue = |hot| if s.hot == hot { s.text } else { s.muted };
                self.list.fill(track_rect(r), s.line);
                self.list.fill(thumb_rect(r, content_h, max, off), hue(BarHot::Thumb));
                let (up, down) = arrow_rects(r);
                let (show_up, show_down) = arrows_shown(max, off);
                if show_up {
                    self.list.polyline(chevron(up, ArrowDir::Up), 1.0, hue(BarHot::Up));
                }
                if show_down {
                    self.list.polyline(chevron(down, ArrowDir::Down), 1.0, hue(BarHot::Down));
                }
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
