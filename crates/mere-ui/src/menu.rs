//! The menu (`menu`): a line in the compact select's look that opens, above or under it, an assembly of rows. A row
//! is a selection (`menu_select`: a selector facing a label, with an optional muted detail line under it) or an
//! action (`menu_action`: a text that runs something); thin separators (`menu_separator`) part groups of rows. Its
//! selection rows serve either kind of choice: several with multiple selectors, one with single selectors.
//!
//! Use `select_menu` to pick one value among short texts, the list closing on the pick; use `menu` when the rows
//! carry selectors, a second line, separators or an action, or when several can be chosen.
//!
//! The list opens above or under its line with exactly the line's width, it holds real widgets (built by the caller
//! inside it, `Ui::overlay`), and it stays open while rows are acted on; the caller may close it (a single choice
//! closes on its pick). Escape or a press outside it closes it (the press reaches nothing else, as with a popup list)
//! and gives the focus back to the line. It never scrolls: a list too long for the window is the caller's to shorten.
//!
//! A row asks and the caller decides: `Resp.clicked` on a selection row means a press or Space asked for it to
//! change, and the caller sets what is selected (one row for a single choice, a flip for several).
//!
//! Keyboard: a selection row, an action row and the line take the focus as a field does (a press, or Tab and
//! Shift+Tab in build order); Space acts on the one that has it (asks for a row, opens or closes the list, runs the
//! action). Down on the closed line opens the list on its first row, Up on its last; in the open list Up and Down
//! move the focus between rows, stopping at either end. The backend sends Space as the character ' '. Enter, while
//! the focus is on the line or in its list, is only reported (`MenuResp::enter`): what it does is the caller's.
//!
//! ```ignore
//! let r = ui.menu("size", "Size: Medium", &mut open, MenuOpts::default(), |ui| {
//!     for (i, s) in sizes.iter().enumerate() {
//!         let row = SelectRow { label: s, selector: Selector::Single, selected: i == chosen, ..SelectRow::default() };
//!         if ui.menu_select(&format!("s{i}"), row).clicked { chosen = i; close = true; }
//!     }
//!     ui.menu_separator();
//!     if ui.menu_action("custom", "Custom size\u{2026}").clicked { custom = true; }
//! });
//! if close { open = false; }
//! ```

use mere_base::{Cursor, Key, Rect, Vec2};
use mere_layout::{Align, Dim, Display, Edges, Insets, Margin, Position, Side, Style, Track, Tracks};

use crate::field::{Focus, Typed};
use crate::id::Id;
use crate::line::LineLook;
use crate::selector::{MARK_SIZE, MarkOpts, Selector};
use crate::text::{TextLook, Tone};
use crate::tokens::{CONTROL_COMPACT, FRAME, GAP, GAP_TIGHT, INSET, INSET_HAIR, INSET_TIGHT, SPACE_4};
use crate::ui::{Paint, Resp, Ui};

/// How far below the top of its first line the selector of a selection row sits: the boards' `margin-top: 4px`.
const MARK_DROP: f32 = SPACE_4;

/// What a selection row says and shows. The second line is `detail` and `detail_tail` joined by " · ", muted; it is
/// left out when both are empty. The label is cut at its end, `detail` is kept whole, and `detail_tail` is cut at its
/// start, so that the end of a long path or address stays readable.
#[derive(Copy, Clone, Debug, Default)]
pub struct SelectRow<'a> {
    pub label: &'a str,
    pub detail: &'a str,
    pub detail_tail: &'a str,
    pub selector: Selector,
    /// Shown as selected: the caller's value, which a click asks to change.
    pub selected: bool,
    /// Faded, no press and out of the focus's walk.
    pub disabled: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MenuOpts {
    /// The list opens above its line (a control at the bottom of a panel) instead of under it.
    pub up: bool,
    /// The line is faded and opens nothing; an open list closes.
    pub disabled: bool,
}

/// What the person did to a menu this frame.
#[derive(Copy, Clone, Debug)]
pub struct MenuResp {
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

/// A row of the list: `inset-tight` above and below, `inset` at the sides; the selectors' column, then the text's.
fn row_style() -> Style {
    let cols = [Track::Px(MARK_SIZE), Track::MinMaxFr0(1.0)];
    Style { display: Display::Grid, cols: Tracks::new(&cols), gap: Vec2::new(GAP, 0.0), align_items: Align::Start, pad: Edges::vh(INSET_TIGHT, INSET), ..Style::default() }
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
    /// elsewhere. Inside an open menu the first one built takes it on the frame the list opens. True when it has the
    /// focus after this.
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
        if let Some(rows) = &mut self.menu_rows {
            rows.push(id);
            if std::mem::take(&mut self.menu_first) {
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

    /// The keys of a row: Space, once it is focused, is a press; Up or Down is kept for the menu, which moves the
    /// focus once all its rows are built.
    fn row_keys(&mut self, id: Id, focused: bool) -> bool {
        let keys = if focused { self.focus_keys(id) } else { Keys::default() };
        if keys.step != 0 {
            self.menu_step = Some((id, keys.step));
        }
        keys.space
    }

    /// A selection row of a menu (or of any list): the selector at its head facing the label on the first line, and,
    /// muted on a second, `detail` and `detail_tail`. The whole row takes the press; the focused row is filled with the
    /// selection colour. A press, or Space once focused, is `Resp.clicked`: the person asked for the row to change,
    /// which the caller applies; the `Ui` then asks for one more frame (`want_redraw`) to show it.
    pub fn menu_select(&mut self, key: &str, row: SelectRow) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, row.disabled, Cursor::Hand);
        let focused = self.focusable(&resp, row.disabled);
        resp.clicked |= self.row_keys(id, focused);
        if resp.clicked {
            // The caller applies the change after this row was painted: one more frame shows it.
            self.redraw = true;
        }
        let t = *self.theme();
        let fade = |c| t.disabled_if(c, row.disabled);
        self.open(key, id, row_style(), Paint::Box { fill: focused.then_some(t.select), frame: None });
        let mark = MarkOpts { disabled: row.disabled, lit: resp.hovered || resp.held, on_selection: focused };
        let drop = Style { pad: Edges::new(MARK_DROP, 0.0, 0.0, 0.0), ..Style::default() };
        self.node("mark", drop, |ui| ui.selector_mark(row.selector, row.selected, mark));
        let label_color = fade(if focused { t.select_text } else { t.text });
        let muted = fade(t.muted);
        self.node("lines", Style { column: true, min_w: Dim::Px(0.0), ..Style::default() }, |ui| {
            ui.add_ellipsis(row.label, TextLook::body(Tone::Text), label_color, false);
            ui.detail_line(row.detail, row.detail_tail, muted);
        });
        self.close();
        resp
    }

    /// An action row of a menu (such as "Add an item…"): its text in the rows' text column, after an empty selectors'
    /// column so that it lines up with the labels above, underlined under the pointer, filled when focused. A press or
    /// Space on it is `Resp.clicked`; the list stays open, the caller closes it if the action wants (setting the
    /// menu's `open` false), and the line then has the focus back.
    pub fn menu_action(&mut self, key: &str, text: &str) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, false, Cursor::Hand);
        let focused = self.focusable(&resp, false);
        resp.clicked |= self.row_keys(id, focused);
        let t = *self.theme();
        self.open(key, id, row_style(), Paint::Box { fill: focused.then_some(t.select), frame: None });
        self.add("pad", Style { w: Dim::Px(MARK_SIZE), ..Style::default() }, Paint::None);
        let color = if focused { t.select_text } else { t.text };
        self.add_ellipsis(text, TextLook::body(Tone::Text), color, resp.hovered);
        self.close();
        resp
    }

    /// A thin line between groups of rows in a menu, `inset` in from its sides.
    pub fn menu_separator(&mut self) {
        self.rule(crate::rule::Rule::Separator, Edges::vh(INSET_TIGHT, INSET));
    }

    /// A line in the compact select's look (its text cut at its end, the arrow at the right) that opens a list of rows
    /// above it or under it, exactly as wide. `open` is the caller's: a press on the line or Space on it flips it, Down
    /// or Up opens it, and Escape or a press outside the line and the list sets it false. `rows` builds what the list
    /// holds (selection rows, separators, action rows) while it is open; it is not built otherwise. When the list
    /// opens, its first row takes the focus (its last when Up opened it); when it closes, the line does. The focused
    /// line is framed in the text colour.
    pub fn menu(&mut self, key: &str, line: &str, open: &mut bool, opts: MenuOpts, rows: impl FnOnce(&mut Ui)) -> MenuResp {
        let id = self.key(key);
        let (mut opened, mut closed) = (false, false);
        // Escape or a press outside, taken before this frame was built.
        if self.menu_close == Some(id) {
            self.menu_close = None;
            if *open {
                *open = false;
                closed = true;
            }
        }
        // The caller closed the list it held open in the last frame (a pick, or an action row that opens a dialog):
        // closed as Escape would close it, so that the keyboard is not left on a row that is gone.
        if !*open && !closed && self.menu_hung.is_some_and(|h| h.id == id) {
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
        let look = LineLook { height: CONTROL_COMPACT, pad_v: INSET_HAIR, disabled: opts.disabled, focused: self.has_focus(lid) };
        self.menu_line("line", lid, line, look);

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
            self.menu_rows = Some(Vec::new());
            // The first row takes the focus as it is built, unless Up asked for the last, given once all are built.
            self.menu_first = (opened && into != Some(-1)) || into == Some(1);
            self.menu_step = None;
            let list_id = self.overlay("list", style, rows);
            mine = self.menu_rows.take().unwrap_or_default();
            self.menu_first = false;
            let to = match (into, self.menu_step.take()) {
                (Some(-1), _) => mine.last().copied(),
                (_, Some((from, step))) => mine.iter().position(|&r| r == from).and_then(|at| mine.get((at as i64 + step as i64).max(0) as usize)).copied(),
                _ => None,
            };
            if let Some(to) = to.filter(|&to| !self.has_focus(to)) {
                self.give_focus(to);
                // The row was painted without the focus: show it in one more frame, as after Tab.
                self.redraw = true;
            }
            self.menu_hung = Some(Hung { id, list: self.rect(list_id).unwrap_or_default(), line: resp.rect.unwrap_or_default() });
            self.menu_built = true;
            list = Some(list_id);
        }
        self.close();
        let ours = self.focus.as_ref().is_some_and(|f| f.id == lid || mine.contains(&f.id));
        let enter = ours && self.enter_typed();
        MenuResp { line: resp, list, open: *open, opened, closed, enter }
    }

    /// At the start of a frame, before any widget reads the pointer: a press outside the open list and its line
    /// closes the list and is taken, so that it reaches nothing under it.
    pub(crate) fn menu_begin_frame(&mut self) {
        self.menu_built = false;
        let Some(h) = self.menu_hung else { return };
        let Some(p) = self.input.press_pos.filter(|_| self.input.pressed) else { return };
        if h.list.contains(p) || h.line.contains(p) {
            return;
        }
        self.menu_close = Some(h.id);
        self.input.pressed = false;
        self.input.press_pos = None;
    }
}
