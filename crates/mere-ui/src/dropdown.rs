//! Selection controls: the checkbox and the radio button alone with their labels (`checkbox`, `radio`), and the
//! dropdown (`dropdown`): a line in the compact select's look that opens a list of option rows (`option`) above or
//! under it. One row type serves both kinds of choice: several with check boxes, one with radio buttons; either may
//! carry a second muted line under its label.
//!
//! Use `select_menu` to pick one value among short texts, the list closing on the pick; use `dropdown` when the rows
//! carry marks, a second line, separators or an action, or when several can be chosen.
//!
//! The list opens above or under its line with exactly the line's width, it holds real widgets (built by the caller
//! inside it, `Ui::overlay`), and it stays open while rows are acted on; the caller may close it (a single choice
//! closes on its pick). Escape or a press outside it closes it (the press reaches nothing else, as with a menu) and
//! gives the focus back to the line. It never scrolls: a list too long for the window is the caller's to shorten.
//!
//! A row asks and the caller decides: `Resp.clicked` on a row means a press or Space asked for it to change, and the
//! caller sets what is selected (one row for a single choice, a flip for several).
//!
//! Keyboard: an option row, the action row and the line take the focus as a field does (a press, or Tab and
//! Shift+Tab in build order); Space acts on the one that has it (asks for a row, opens or closes the list, runs the
//! action). Down on the closed line opens the list on its first row, Up on its last; in the open list Up and Down
//! move the focus between rows, stopping at either end. The backend sends Space as the character ' '. Enter, while
//! the focus is on the line or in its list, is only reported (`DropdownResp::enter`): what it does is the caller's.
//!
//! ```ignore
//! let r = ui.dropdown("size", "Size: Medium", &mut open, DropdownOpts::default(), |ui| {
//!     for (i, s) in sizes.iter().enumerate() {
//!         let row = OptionRow { label: s, mark: Mark::Radio, selected: i == chosen, ..OptionRow::default() };
//!         if ui.option(&format!("s{i}"), row).clicked { chosen = i; close = true; }
//!     }
//!     ui.dropdown_separator();
//!     if ui.dropdown_action("custom", "Custom size\u{2026}").clicked { custom = true; }
//! });
//! if close { open = false; }
//! ```

use mere_base::{Cmd, Color, Cursor, Key, Rect, Vec2};
use mere_layout::{Align, Dim, Display, Edges, Insets, Margin, Position, Side, Style, Track, Tracks};

use crate::field::{Focus, Typed};
use crate::id::Id;
use crate::text::{TextLook, Tone};
use crate::tokens::{CONTROL_COMPACT, FADE_DISABLED, FRAME, GAP, GAP_TIGHT, INSET, INSET_HAIR, INSET_TIGHT, SPACE_4, SWATCH};
use crate::ui::{Paint, Resp, Ui};

/// The side of a check box and of a radio button: a swatch's, so that an option row's columns line up with a list
/// row's.
pub const MARK_SIZE: f32 = SWATCH;
/// How far below the top of its first line the mark of an option row sits: the boards' `margin-top: 4px`.
const MARK_DROP: f32 = SPACE_4;
/// The check: 1 px squares inside the frame, the boards' 10 px glyph moved in by the frame.
const CHECK: [(f32, f32); 7] = [(9.0, 3.0), (8.0, 4.0), (7.0, 5.0), (6.0, 6.0), (5.0, 7.0), (4.0, 6.0), (3.0, 5.0)];
/// The radio button's ring as blocks of whole pixels `(x, y, w, h)`, a circle of `MARK_SIZE`: no antialiasing, so
/// that it stays as sharp as the check box's frame beside it.
const RING: [(f32, f32, f32, f32); 12] = [
    (4.0, 0.0, 4.0, 1.0),
    (2.0, 1.0, 2.0, 1.0),
    (8.0, 1.0, 2.0, 1.0),
    (1.0, 2.0, 1.0, 2.0),
    (10.0, 2.0, 1.0, 2.0),
    (0.0, 4.0, 1.0, 4.0),
    (11.0, 4.0, 1.0, 4.0),
    (1.0, 8.0, 1.0, 2.0),
    (10.0, 8.0, 1.0, 2.0),
    (2.0, 10.0, 2.0, 1.0),
    (8.0, 10.0, 2.0, 1.0),
    (4.0, 11.0, 4.0, 1.0),
];
/// The chosen radio button's dot, 6 px across and centred in the ring, in the same blocks.
const DOT: [(f32, f32, f32, f32); 5] = [(5.0, 3.0, 2.0, 1.0), (4.0, 4.0, 4.0, 1.0), (3.0, 5.0, 6.0, 2.0), (4.0, 7.0, 4.0, 1.0), (5.0, 8.0, 2.0, 1.0)];

/// A standalone checkbox.
#[derive(Copy, Clone, Debug, Default)]
pub struct CheckboxOpts {
    /// Faded to `FADE_DISABLED`, and no press.
    pub disabled: bool,
}

/// A standalone radio button.
#[derive(Copy, Clone, Debug, Default)]
pub struct RadioOpts {
    /// Faded to `FADE_DISABLED`, and no press.
    pub disabled: bool,
}

/// What an option row shows at its head.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mark {
    /// A check box: one of several choices.
    #[default]
    Check,
    /// A radio button: the one choice.
    Radio,
}

/// What an option row says. The second line is `detail` and `detail_tail` joined by " · ", muted; it is left out when
/// both are empty. The label is cut at its end, `detail` is kept whole, and `detail_tail` is cut at its start, so
/// that the end of a long path or address stays readable.
#[derive(Copy, Clone, Debug, Default)]
pub struct OptionRow<'a> {
    pub label: &'a str,
    pub detail: &'a str,
    pub detail_tail: &'a str,
    pub mark: Mark,
    /// Shown as chosen: the caller's value, which a click asks to change.
    pub selected: bool,
    /// Faded, no press and out of the focus's walk.
    pub disabled: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct DropdownOpts {
    /// The list opens above its line (a control at the bottom of a panel) instead of under it.
    pub up: bool,
    /// The line is faded and opens nothing; an open list closes.
    pub disabled: bool,
}

/// What the person did to a dropdown this frame.
#[derive(Copy, Clone, Debug)]
pub struct DropdownResp {
    /// The line the list hangs from.
    pub line: Resp,
    /// The list's box while it is open.
    pub list: Option<Id>,
    pub open: bool,
    /// It opened or closed in this frame (a press on the line, Space, Down, Escape, a press outside, the caller).
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

/// The keys a focused row or line took this frame.
#[derive(Copy, Clone, Default)]
struct Keys {
    space: bool,
    /// Up (-1) or Down (+1), the last one typed.
    step: i32,
}

fn blocks(blocks: &[(f32, f32, f32, f32)], color: Color) -> impl Iterator<Item = Cmd> + '_ {
    blocks.iter().map(move |&(x, y, w, h)| Cmd::Fill { rect: Rect::new(x, y, w, h), color })
}

/// The check alone, in a `MARK_SIZE` square's coordinates.
fn check_cmds(color: Color) -> impl Iterator<Item = Cmd> {
    CHECK.into_iter().map(move |(x, y)| Cmd::Fill { rect: Rect::new(x, y, 1.0, 1.0), color })
}

/// A mark's commands in its own coordinates: `line` is the frame or the ring, `inner` the check or the dot when
/// selected.
fn mark_cmds(mark: Mark, line: Color, inner: Option<Color>) -> Vec<Cmd> {
    let mut cmds = Vec::new();
    match mark {
        Mark::Check => {
            cmds.push(Cmd::Frame { rect: Rect::new(0.0, 0.0, MARK_SIZE, MARK_SIZE), width_px: FRAME, color: line });
            if let Some(c) = inner {
                cmds.extend(check_cmds(c));
            }
        }
        Mark::Radio => {
            cmds.extend(blocks(&RING, line));
            if let Some(c) = inner {
                cmds.extend(blocks(&DOT, c));
            }
        }
    }
    cmds
}

/// A row of the list: `inset-tight` above and below, `inset` at the sides; the mark's column, then the text's.
fn row_style() -> Style {
    let cols = [Track::Px(MARK_SIZE), Track::MinMaxFr0(1.0)];
    Style { display: Display::Grid, cols: Tracks::new(&cols), gap: Vec2::new(GAP, 0.0), align_items: Align::Start, pad: Edges::vh(INSET_TIGHT, INSET), ..Style::default() }
}

fn mark_style() -> Style {
    Style { w: Dim::Px(MARK_SIZE), h: Dim::Px(MARK_SIZE), margin: Margin { top: Side::Px(MARK_DROP), ..Margin::default() }, ..Style::default() }
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
    /// elsewhere. Inside an open dropdown the first one built takes it on the frame the list opens. True when it
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
        if let Some(rows) = &mut self.dropdown_rows {
            rows.push(id);
            if std::mem::take(&mut self.dropdown_first) {
                self.give_focus(id);
            }
        }
        self.has_focus(id)
    }

    /// The keys of a focused non-field: Tab moves the focus on (after painting, as from a field). Space, Up and Down
    /// are taken, so that the row a key on the line just focused does not act on the same key.
    fn focus_keys(&mut self, id: Id) -> Keys {
        let mut keys = Keys::default();
        for t in &self.typed {
            match t {
                Typed::Key(Key::Tab, m) => self.tab = Some((id, m.shift)),
                Typed::Key(Key::Up, _) => keys.step = -1,
                Typed::Key(Key::Down, _) => keys.step = 1,
                Typed::Char(' ') => keys.space = true,
                _ => {}
            }
        }
        self.typed.retain(|t| !matches!(t, Typed::Char(' ') | Typed::Key(Key::Up | Key::Down, _)));
        keys
    }

    fn enter_typed(&self) -> bool {
        self.typed.iter().any(|t| matches!(t, Typed::Key(Key::Enter, _)))
    }

    /// A mark's line and inner colours: the line in the text colour when selected or under the pointer, else the
    /// edge, or muted on the selection fill of a focused row (the edge does not show on it).
    fn mark_colors(&self, selected: bool, lit: bool, on_select: bool, disabled: bool) -> (Color, Option<Color>) {
        let t = *self.theme();
        let line = if selected || lit {
            t.text
        } else if on_select {
            t.muted
        } else {
            t.edge
        };
        let fade = |c| if disabled { t.faded(c, FADE_DISABLED) } else { c };
        (fade(line), selected.then(|| fade(t.text)))
    }

    /// A mark with its label beside it, `control-compact` tall; a press on either is `Resp.clicked`. `flip` paints
    /// the value a press gives (the checkbox, which changes its own value); otherwise the caller applies the press
    /// after this frame was built, and one more frame shows it.
    fn labelled_mark(&mut self, key: &str, label: &str, mark: Mark, selected: bool, flip: bool, disabled: bool) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, disabled, Cursor::Hand);
        let shown = if flip && resp.clicked { !selected } else { selected };
        if resp.clicked && !flip {
            self.redraw = true;
        }
        let t = *self.theme();
        let (line, inner) = self.mark_colors(shown, resp.hovered || resp.held, false, disabled);
        let style = Style { h: Dim::Px(CONTROL_COMPACT), gap: Vec2::new(GAP, 0.0), align_items: Align::Center, justify_self: Some(Align::Start), ..Style::default() };
        self.open(key, id, style, Paint::None);
        let sq = Style { w: Dim::Px(MARK_SIZE), h: Dim::Px(MARK_SIZE), shrink: 0.0, ..Style::default() };
        self.add("mark", sq, Paint::Cmds(mark_cmds(mark, line, inner)));
        if !label.is_empty() {
            let word = if disabled { t.faded(t.text, FADE_DISABLED) } else { t.text };
            self.add_text(label, TextLook::body(Tone::Text), word, false, Style::default());
        }
        self.close();
        resp
    }

    /// A checkbox with its label beside it, `control-compact` tall: a press on the box or the label checks or
    /// unchecks it. `Resp.clicked` means the value changed. Like the toggle it takes no keyboard focus; an option
    /// row does.
    pub fn checkbox(&mut self, key: &str, label: &str, checked: &mut bool, opts: CheckboxOpts) -> Resp {
        let resp = self.labelled_mark(key, label, Mark::Check, *checked, true, opts.disabled);
        if resp.clicked {
            *checked = !*checked;
        }
        resp
    }

    /// A radio button with its label beside it, `control-compact` tall. `Resp.clicked` means the person asked for
    /// this one (a press on the ring or the label); the caller selects it and unselects the others of its group, and
    /// the next frame, which the `Ui` asks for (`want_redraw`), shows it. Like the checkbox it takes no keyboard focus.
    pub fn radio(&mut self, key: &str, label: &str, selected: bool, opts: RadioOpts) -> Resp {
        self.labelled_mark(key, label, Mark::Radio, selected, false, opts.disabled)
    }

    /// One row of a dropdown (or of any list): the mark at its head (a check box, a radio button) facing the label on
    /// the first line, and, muted on a second, `detail` and
    /// `detail_tail`. The whole row takes the press; the focused row is filled with the selection colour. A press, or
    /// Space once focused, is `Resp.clicked`: the person asked for the row to change, which the caller applies; the
    /// `Ui` then asks for one more frame (`want_redraw`) to show it.
    pub fn option(&mut self, key: &str, row: OptionRow) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, row.disabled, Cursor::Hand);
        let focused = self.focusable(&resp, row.disabled);
        let keys = if focused { self.focus_keys(id) } else { Keys::default() };
        resp.clicked |= keys.space;
        if resp.clicked {
            // The caller applies the change after this row was painted: one more frame shows it.
            self.redraw = true;
        }
        if keys.step != 0 {
            self.dropdown_step = Some((id, keys.step));
        }
        let t = *self.theme();
        let fade = |c| if row.disabled { t.faded(c, FADE_DISABLED) } else { c };
        let (line, inner) = self.mark_colors(row.selected, resp.hovered || resp.held, focused, row.disabled);
        self.open(key, id, row_style(), Paint::Box { fill: focused.then_some(t.select), frame: None });
        self.add("mark", mark_style(), Paint::Cmds(mark_cmds(row.mark, line, inner)));
        let label_color = fade(if focused { t.select_text } else { t.text });
        let muted = fade(t.muted);
        self.node("lines", Style { column: true, min_w: Dim::Px(0.0), ..Style::default() }, |ui| {
            ui.add_ellipsis(row.label, TextLook::body(Tone::Text), label_color, false);
            if !(row.detail.is_empty() && row.detail_tail.is_empty()) {
                // The status look (11 px on 16) keeps a two-line row on whole pixels, which the small look's 16.5 does not.
                let look = TextLook::status(Tone::Muted);
                ui.node("detail", Style { min_w: Dim::Px(0.0), ..Style::default() }, |ui| {
                    let lead = match (row.detail.is_empty(), row.detail_tail.is_empty()) {
                        (false, false) => format!("{} \u{b7} ", row.detail),
                        (false, true) => row.detail.to_string(),
                        _ => String::new(),
                    };
                    if !lead.is_empty() {
                        ui.add_text(&lead, look, muted, false, Style { shrink: 0.0, ..Style::default() });
                    }
                    if !row.detail_tail.is_empty() {
                        ui.add_ellipsis_start(row.detail_tail, look, muted);
                    }
                });
            }
        });
        self.close();
        resp
    }

    /// The unmarked row at the end of a dropdown (an action such as "Add an item…"): its text in the rows' text column,
    /// after an empty marks' column so that it lines up with the labels above, underlined under the pointer, filled when focused. A
    /// press or Space on it is `Resp.clicked`; the list stays open, the caller closes it if the action wants (setting
    /// the list's `open` false), and the line then has the focus back.
    pub fn dropdown_action(&mut self, key: &str, text: &str) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, false, Cursor::Hand);
        let focused = self.focusable(&resp, false);
        let keys = if focused { self.focus_keys(id) } else { Keys::default() };
        resp.clicked |= keys.space;
        if keys.step != 0 {
            self.dropdown_step = Some((id, keys.step));
        }
        let t = *self.theme();
        self.open(key, id, row_style(), Paint::Box { fill: focused.then_some(t.select), frame: None });
        self.add("pad", Style { w: Dim::Px(MARK_SIZE), ..Style::default() }, Paint::None);
        let color = if focused { t.select_text } else { t.text };
        self.add_ellipsis(text, TextLook::body(Tone::Text), color, resp.hovered);
        self.close();
        resp
    }

    /// A thin line between groups of rows in a dropdown, `inset` in from its sides.
    pub fn dropdown_separator(&mut self) {
        let t = *self.theme();
        let (v, h) = (Side::Px(INSET_TIGHT), Side::Px(INSET));
        let style = Style { h: Dim::Px(FRAME), shrink: 0.0, margin: Margin { left: h, top: v, right: h, bottom: v }, ..Style::default() };
        self.fill(style, t.line);
    }

    /// A line in the compact select's look (its text cut at its end, the arrow at the right) that opens a list of
    /// option rows above it or under it, exactly as wide. `open` is the caller's: a press on the line or Space on it
    /// flips it, Down or Up opens it, and Escape or a press outside the line and the list sets it false. `rows` builds
    /// what the list holds (option rows, separators, an action row) while it is open; it is not built otherwise.
    /// When the list opens, its first row takes the focus (its last when Up opened it); when it closes, the line
    /// does. The focused line is framed in the text colour.
    pub fn dropdown(&mut self, key: &str, line: &str, open: &mut bool, opts: DropdownOpts, rows: impl FnOnce(&mut Ui)) -> DropdownResp {
        let id = self.key(key);
        let (mut opened, mut closed) = (false, false);
        // Escape or a press outside, taken before this frame was built.
        if self.dropdown_close == Some(id) {
            self.dropdown_close = None;
            if *open {
                *open = false;
                closed = true;
            }
        }
        // The caller closed the list it held open in the last frame (a pick, or an action row that opens a dialog):
        // closed as Escape would close it, so that the keyboard is not left on a row that is gone.
        if !*open && !closed && self.dropdown_hung.is_some_and(|h| h.id == id) {
            closed = true;
        }
        let wrap = Style { column: true, min_w: Dim::Px(0.0), position: Position::Relative, ..Style::default() };
        self.open(key, id, wrap, Paint::None);
        let lid = self.key("line");
        let resp = self.interact(lid, opts.disabled, Cursor::Hand);
        let focused = self.focusable(&resp, opts.disabled);
        let keys = if focused { self.focus_keys(lid) } else { Keys::default() };
        // Up or Down on the line of an open list goes into it, to its first row or its last.
        let mut into = None;
        if opts.disabled {
            closed |= std::mem::replace(open, false);
        } else if resp.clicked || keys.space {
            *open = !*open;
            if *open {
                opened = true;
            } else {
                closed = true;
            }
        } else if keys.step != 0 {
            if !*open {
                *open = true;
                opened = true;
            }
            into = Some(keys.step);
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
            self.dropdown_rows = Some(Vec::new());
            // The first row takes the focus as it is built, unless Up asked for the last, given once all are built.
            self.dropdown_first = (opened && into != Some(-1)) || into == Some(1);
            self.dropdown_step = None;
            let list_id = self.overlay("list", style, rows);
            mine = self.dropdown_rows.take().unwrap_or_default();
            self.dropdown_first = false;
            let to = match (into, self.dropdown_step.take()) {
                (Some(-1), _) => mine.last().copied(),
                (_, Some((from, step))) => mine.iter().position(|&r| r == from).and_then(|at| mine.get((at as i64 + step as i64).max(0) as usize)).copied(),
                _ => None,
            };
            if let Some(to) = to.filter(|&to| !self.has_focus(to)) {
                self.give_focus(to);
                // The row was painted without the focus: show it in one more frame, as after Tab.
                self.redraw = true;
            }
            self.dropdown_hung = Some(Hung { id, list: self.rect(list_id).unwrap_or_default(), line: resp.rect.unwrap_or_default() });
            self.dropdown_built = true;
            list = Some(list_id);
        }
        self.close();
        let ours = self.focus.as_ref().is_some_and(|f| f.id == lid || mine.contains(&f.id));
        let enter = ours && self.enter_typed();
        DropdownResp { line: resp, list, open: *open, opened, closed, enter }
    }

    /// At the start of a frame, before any widget reads the pointer: a press outside the open list and its line
    /// closes the list and is taken, so that it reaches nothing under it.
    pub(crate) fn dropdown_begin_frame(&mut self) {
        self.dropdown_built = false;
        let Some(h) = self.dropdown_hung else { return };
        let Some(p) = self.input.press_pos.filter(|_| self.input.pressed) else { return };
        if h.list.contains(p) || h.line.contains(p) {
            return;
        }
        self.dropdown_close = Some(h.id);
        self.input.pressed = false;
        self.input.press_pos = None;
    }
}
