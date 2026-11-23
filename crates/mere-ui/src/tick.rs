//! Tick boxes and the tick list: a square box that a press ticks and unticks, alone with its label (`tick_box`) or as
//! the head of a row that names a place on one line and says where it is on a second (`tick_row`), and a list of
//! such rows hung from a line that looks like the compact select (`tick_list`).
//!
//! The list is the select's menu turned for choosing several things: it opens above or under its line with exactly
//! the line's width, it holds real widgets (built by the caller inside it, `Ui::overlay`), and it stays open while
//! rows are ticked. Escape or a press outside it closes it (the press reaches nothing else, as with a menu) and gives
//! the focus back to the line. It never scrolls: a list too long for the window is the caller's to shorten.
//!
//! Keyboard: a tick row, the list's action row and the line take the focus as a field does (a press, or Tab and
//! Shift+Tab in build order); Space acts on the one that has it (ticks a row, opens or closes the list, runs the
//! action). The backend sends Space as the character ' '. Enter, while the focus is on the line or in its list, is
//! only reported (`TickListResp::enter`): what it does is the caller's.
//!
//! ```ignore
//! let r = ui.tick_list("place", "To: Exploration", &mut open, TickListOpts { up: true, ..Default::default() }, |ui| {
//!     for (i, p) in places.iter_mut().enumerate() {
//!         ui.tick_row(&format!("p{i}"), TickRow { name: &p.name, how: &p.how, path: &p.path, disabled: false }, &mut p.ticked);
//!     }
//!     ui.tick_rule();
//!     if ui.tick_action("choose", "Choose a folder…").clicked { choose = true; }
//! });
//! if r.enter { apply = true; }
//! ```

use mere_base::{Cmd, Color, Cursor, Key, Rect, Vec2};
use mere_layout::{Align, Dim, Display, Edges, Insets, Margin, Position, Side, Style, Track, Tracks};

use crate::field::{Focus, Typed};
use crate::id::Id;
use crate::text::{TextLook, Tone};
use crate::tokens::{CONTROL_COMPACT, FADE_DISABLED, FRAME, GAP, GAP_TIGHT, INSET, INSET_HAIR, INSET_TIGHT, SPACE_4, SWATCH};
use crate::ui::{Paint, Resp, Ui};

/// The box's side: a swatch's, so that a tick row's columns line up with a list row's.
pub const TICK_BOX: f32 = SWATCH;
/// How far below the top of its first line the box of a tick row sits: the boards' `margin-top: 4px`.
const BOX_DROP: f32 = SPACE_4;
/// The tick: a check of 1 px squares inside the frame, the boards' 10 px glyph moved in by the frame.
const CHECK: [(f32, f32); 7] = [(9.0, 3.0), (8.0, 4.0), (7.0, 5.0), (6.0, 6.0), (5.0, 7.0), (4.0, 6.0), (3.0, 5.0)];

/// A standalone tick box.
#[derive(Copy, Clone, Debug, Default)]
pub struct TickOpts {
    /// Faded to `FADE_DISABLED`, and no press.
    pub disabled: bool,
}

/// What a tick row says. The second line is `how` and `path` joined by " · ", muted; it is left out when both are
/// empty. The name is cut at its end, the path at its start, so that the folder's own name stays readable.
#[derive(Copy, Clone, Debug, Default)]
pub struct TickRow<'a> {
    pub name: &'a str,
    pub how: &'a str,
    pub path: &'a str,
    /// Faded, no press and out of Tab's walk.
    pub disabled: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TickListOpts {
    /// The list opens above its line (the corner over a panel's button) instead of under it.
    pub up: bool,
    /// The line is faded and opens nothing; an open list closes.
    pub disabled: bool,
}

/// What the person did to a tick list this frame.
#[derive(Copy, Clone, Debug)]
pub struct TickListResp {
    /// The line the list hangs from.
    pub line: Resp,
    /// The list's box while it is open.
    pub list: Option<Id>,
    pub open: bool,
    /// It opened or closed in this frame (a press on the line, Space, Escape, a press outside).
    pub opened: bool,
    pub closed: bool,
    /// Enter was pressed while the focus was on the line or in the list.
    pub enter: bool,
}

/// The open list as last built: what a press is judged against at the start of the next frame.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Hung {
    pub id: Id,
    pub list: Rect,
    pub line: Rect,
}

/// The box: its frame, and the tick when ticked, in the box's own coordinates.
fn box_cmds(frame: Color, tick: Option<Color>) -> Vec<Cmd> {
    let mut cmds = vec![Cmd::Frame { rect: Rect::new(0.0, 0.0, TICK_BOX, TICK_BOX), width_px: FRAME, color: frame }];
    if let Some(c) = tick {
        cmds.extend(CHECK.iter().map(|&(x, y)| Cmd::Fill { rect: Rect::new(x, y, 1.0, 1.0), color: c }));
    }
    cmds
}

/// A row of the list: the box column and the text, `inset-tight` above and below, `inset` at the sides.
fn row_style() -> Style {
    Style {
        display: Display::Grid,
        cols: Tracks::new(&[Track::Px(TICK_BOX), Track::MinMaxFr0(1.0)]),
        gap: Vec2::new(GAP, 0.0),
        align_items: Align::Start,
        pad: Edges::vh(INSET_TIGHT, INSET),
        ..Style::default()
    }
}

impl Ui {
    fn has_focus(&self, id: Id) -> bool {
        self.focus.as_ref().is_some_and(|f| f.id == id)
    }

    /// Takes the keyboard from whoever has it; a field that had it is told when it is built (`Ui::lost`).
    fn give_focus(&mut self, id: Id) {
        if !self.has_focus(id) {
            self.lost = self.focus.replace(Focus { id, state: None, select_all: false });
        }
    }

    /// The focus of what is not a field: in Tab's walk unless disabled, taken by a press on it, lost to a press
    /// elsewhere. Inside an open tick list the first one built takes it on the frame the list opens. True when it
    /// has the focus after this.
    fn focusable(&mut self, resp: &Resp, disabled: bool) -> bool {
        let id = resp.id;
        if disabled {
            if self.has_focus(id) {
                self.focus = None;
            }
            return false;
        }
        self.order.push(id);
        if resp.pressed {
            self.give_focus(id);
        } else if self.input.pressed && self.has_focus(id) {
            self.focus = None;
        }
        // After the press: the press that opened the list lies on its line, elsewhere for this row.
        if let Some(rows) = &mut self.tick_rows {
            rows.push(id);
            if std::mem::take(&mut self.tick_first) {
                self.give_focus(id);
            }
        }
        self.has_focus(id)
    }

    /// The keys of a focused non-field: Tab moves the focus on (after painting, as from a field); true when Space came.
    /// Space is taken, so that the row a Space on the line just focused does not tick with it.
    fn focus_keys(&mut self, id: Id) -> bool {
        for t in &self.typed {
            if let Typed::Key(Key::Tab, m) = t {
                self.tab = Some((id, m.shift));
            }
        }
        let n = self.typed.len();
        self.typed.retain(|t| !matches!(t, Typed::Char(' ')));
        self.typed.len() != n
    }

    fn enter_typed(&self) -> bool {
        self.typed.iter().any(|t| matches!(t, Typed::Key(Key::Enter, _)))
    }

    /// The box's frame and tick colours: the frame in the text colour when ticked or under the pointer, else the
    /// edge, or muted on the selection fill of a focused row (the edge does not show on it).
    fn tick_colors(&self, ticked: bool, lit: bool, on_select: bool, disabled: bool) -> (Color, Option<Color>) {
        let t = *self.theme();
        let frame = if ticked || lit {
            t.text
        } else if on_select {
            t.muted
        } else {
            t.edge
        };
        let fade = |c| if disabled { t.faded(c, FADE_DISABLED) } else { c };
        (fade(frame), ticked.then(|| fade(t.text)))
    }

    /// A tick box with its label beside it, `control-compact` tall: a press on the box or the label ticks or unticks
    /// it. `Resp.clicked` means the value changed. Like the toggle it takes no keyboard focus; a tick row does.
    pub fn tick_box(&mut self, key: &str, label: &str, ticked: &mut bool, opts: TickOpts) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, opts.disabled, Cursor::Hand);
        if resp.clicked {
            *ticked = !*ticked;
        }
        let t = *self.theme();
        let (frame, tick) = self.tick_colors(*ticked, resp.hovered || resp.held, false, opts.disabled);
        let style = Style {
            h: Dim::Px(CONTROL_COMPACT),
            gap: Vec2::new(GAP, 0.0),
            align_items: Align::Center,
            justify_self: Some(Align::Start),
            ..Style::default()
        };
        self.open(key, id, style, Paint::None);
        let sq = Style { w: Dim::Px(TICK_BOX), h: Dim::Px(TICK_BOX), shrink: 0.0, ..Style::default() };
        self.add("box", sq, Paint::Cmds(box_cmds(frame, tick)));
        if !label.is_empty() {
            let word = if opts.disabled { t.faded(t.text, FADE_DISABLED) } else { t.text };
            self.add_text(label, TextLook::body(Tone::Text), word, false, Style::default());
        }
        self.close();
        resp
    }

    /// A row with a tick box: the name on the first line, and, muted on a second, how the place is reached and its
    /// folder. The whole row takes the press; the focused row is filled with the selection colour and Space ticks
    /// it. `Resp.clicked` means the value changed (by the pointer or by Space).
    pub fn tick_row(&mut self, key: &str, row: TickRow, ticked: &mut bool) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, row.disabled, Cursor::Hand);
        let focused = self.focusable(&resp, row.disabled);
        let flip = resp.clicked || (focused && self.focus_keys(id));
        if flip {
            *ticked = !*ticked;
        }
        resp.clicked = flip;
        let t = *self.theme();
        let fade = |c| if row.disabled { t.faded(c, FADE_DISABLED) } else { c };
        let (frame, tick) = self.tick_colors(*ticked, resp.hovered || resp.held, focused, row.disabled);
        let fill = focused.then_some(t.select);
        self.open(key, id, row_style(), Paint::Box { fill, frame: None });
        let sq = Style { w: Dim::Px(TICK_BOX), h: Dim::Px(TICK_BOX), margin: Margin { top: Side::Px(BOX_DROP), ..Margin::default() }, ..Style::default() };
        self.add("box", sq, Paint::Cmds(box_cmds(frame, tick)));
        let name_color = fade(if focused { t.select_text } else { t.text });
        let muted = fade(t.muted);
        self.node("lines", Style { column: true, min_w: Dim::Px(0.0), ..Style::default() }, |ui| {
            ui.add_ellipsis(row.name, TextLook::body(Tone::Text), name_color, false);
            if !(row.how.is_empty() && row.path.is_empty()) {
                // The status look (11 px on 16) keeps a two-line row on whole pixels, which the small look's 16.5 does not.
                let look = TextLook::status(Tone::Muted);
                ui.node("where", Style { min_w: Dim::Px(0.0), ..Style::default() }, |ui| {
                    let lead = match (row.how.is_empty(), row.path.is_empty()) {
                        (false, false) => format!("{} \u{b7} ", row.how),
                        (false, true) => row.how.to_string(),
                        _ => String::new(),
                    };
                    if !lead.is_empty() {
                        ui.add_text(&lead, look, muted, false, Style { shrink: 0.0, ..Style::default() });
                    }
                    if !row.path.is_empty() {
                        ui.add_ellipsis_start(row.path, look, muted);
                    }
                });
            }
        });
        self.close();
        resp
    }

    /// The plain row at the end of a tick list (an action such as "Choose a folder…"): its text in the rows' text
    /// column, underlined under the pointer, filled when focused. A press or Space on it is `Resp.clicked`; the list
    /// stays open, the caller closes it if the action wants.
    pub fn tick_action(&mut self, key: &str, text: &str) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, false, Cursor::Hand);
        let focused = self.focusable(&resp, false);
        resp.clicked |= focused && self.focus_keys(id);
        let t = *self.theme();
        self.open(key, id, row_style(), Paint::Box { fill: focused.then_some(t.select), frame: None });
        self.add("pad", Style { w: Dim::Px(TICK_BOX), ..Style::default() }, Paint::None);
        let color = if focused { t.select_text } else { t.text };
        self.add_ellipsis(text, TextLook::body(Tone::Text), color, resp.hovered);
        self.close();
        resp
    }

    /// A thin rule between groups of rows in a tick list, `inset` in from its sides.
    pub fn tick_rule(&mut self) {
        let t = *self.theme();
        let (v, h) = (Side::Px(INSET_TIGHT), Side::Px(INSET));
        let style = Style { h: Dim::Px(FRAME), shrink: 0.0, margin: Margin { left: h, top: v, right: h, bottom: v }, ..Style::default() };
        self.fill(style, t.line);
    }

    /// A line in the compact select's look (its text cut at its end, the arrow at the right) that opens a list of tick
    /// rows above it or under it, exactly as wide. `open` is the caller's: a press on the line or Space on it flips it,
    /// and Escape or a press outside the line and the list sets it false. `rows` builds what the list holds (tick
    /// rows, rules, an action row) while it is open; it is not built otherwise. When the list opens, its first row
    /// takes the focus; when it closes, the line does. The focused line is framed in the text colour.
    pub fn tick_list(&mut self, key: &str, line: &str, open: &mut bool, opts: TickListOpts, rows: impl FnOnce(&mut Ui)) -> TickListResp {
        let id = self.key(key);
        let (mut opened, mut closed) = (false, false);
        // Escape or a press outside, taken before this frame was built.
        if self.tick_close == Some(id) {
            self.tick_close = None;
            if *open {
                *open = false;
                closed = true;
            }
        }
        let wrap = Style { column: true, min_w: Dim::Px(0.0), position: Position::Relative, ..Style::default() };
        self.open(key, id, wrap, Paint::None);
        let lid = self.key("line");
        let resp = self.interact(lid, opts.disabled, Cursor::Hand);
        let focused = self.focusable(&resp, opts.disabled);
        let space = focused && self.focus_keys(lid);
        if opts.disabled {
            closed |= std::mem::replace(open, false);
        } else if resp.clicked || space {
            *open = !*open;
            if *open {
                opened = true;
            } else {
                closed = true;
            }
        }
        if closed && !opts.disabled {
            self.give_focus(lid);
        }
        let t = *self.theme();
        let fade = |c| if opts.disabled { t.faded(c, FADE_DISABLED) } else { c };
        let frame = fade(if self.has_focus(lid) { t.text } else { t.edge });
        let style = Style {
            w: Dim::Pct(100.0),
            h: Dim::Px(CONTROL_COMPACT),
            pad: Edges::new(FRAME + INSET_HAIR, FRAME + INSET_TIGHT, FRAME + INSET_HAIR, FRAME + INSET),
            align_items: Align::Center,
            ..Style::default()
        };
        self.open("line", lid, style, Paint::Box { fill: None, frame: Some(frame) });
        self.add_ellipsis(line, TextLook::body(Tone::Text), fade(t.text), false);
        let chevron = Style { w: Dim::Px(8.0), h: Dim::Px(5.0), shrink: 0.0, margin: Margin { left: Side::Auto, ..Margin::default() }, ..Style::default() };
        self.add("chevron", chevron, Paint::Chevron { color: fade(t.text) });
        self.close();

        let mut list = None;
        let mut mine = Vec::new();
        if *open {
            let zero = Dim::Px(0.0);
            let mut inset = Insets { left: zero, right: zero, ..Insets::default() };
            let mut margin = Margin::default();
            if opts.up {
                inset.bottom = Dim::Pct(100.0);
                margin.bottom = Side::Px(GAP_TIGHT);
            } else {
                inset.top = Dim::Pct(100.0);
                margin.top = Side::Px(GAP_TIGHT);
            }
            let style = Style { position: Position::Absolute, inset, margin, column: true, pad: Edges::vh(FRAME + INSET_TIGHT, FRAME), ..Style::default() };
            self.tick_rows = Some(Vec::new());
            self.tick_first = opened;
            let list_id = self.overlay("list", style, rows);
            mine = self.tick_rows.take().unwrap_or_default();
            self.tick_first = false;
            self.tick_hung = Some(Hung { id, list: self.rect(list_id).unwrap_or_default(), line: resp.rect.unwrap_or_default() });
            self.tick_built = true;
            list = Some(list_id);
        }
        self.close();
        let ours = self.focus.as_ref().is_some_and(|f| f.id == lid || mine.contains(&f.id));
        let enter = ours && self.enter_typed();
        TickListResp { line: resp, list, open: *open, opened, closed, enter }
    }

    /// At the start of a frame, before any widget reads the pointer: a press outside the open list and its line
    /// closes the list and is taken, so that it reaches nothing under it.
    pub(crate) fn tick_begin_frame(&mut self) {
        self.tick_built = false;
        let Some(h) = self.tick_hung else { return };
        let Some(p) = self.input.press_pos.filter(|_| self.input.pressed) else { return };
        if h.list.contains(p) || h.line.contains(p) {
            return;
        }
        self.tick_close = Some(h.id);
        self.input.pressed = false;
        self.input.press_pos = None;
    }
}
