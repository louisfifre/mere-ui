//! The menu: the container of an open list. A trigger (`Trigger`: a line with a chevron, or a strip tab) opens, above
//! it or under it, a framed box on the ground colour that holds groups of rows (`group`, and `selection_row`,
//! `action_row`, `immediate_row` in row.rs): the menu brings the frame and the fill, its groups and rows have none,
//! and the separators between groups come from the groups themselves.
//!
//! The list opens above or under its trigger (`MenuOpts::up`): from a line exactly as wide as it, from a tab as wide
//! as its rows want and at least the tab, kept inside the window. It holds real widgets, built by the caller inside it
//! (`Ui::overlay`), and stays open while rows are acted on; the caller may close it (a single choice or an
//! immediate-selection row closes on its pick, an action that opens a dialog closes it). Escape or a press outside it
//! closes it (the press reaches nothing else) and gives the focus back to the trigger. It never scrolls: a list too
//! long for the window is the caller's to shorten.
//!
//! `select_menu` and `tab_menu` (select.rs) are this menu holding a group of immediate-selection rows: the kit has one
//! system of open lists.
//!
//! Keyboard: the rows and the trigger take the focus as a field does (a press, or Tab and Shift+Tab in build order);
//! Space acts on the one that has it (asks for a row, opens or closes the list, runs an action). Down on the closed
//! trigger opens the list on its first row, Up on its last; in the open list Up and Down move the focus between rows,
//! stopping at either end. The backend sends Space as the character ' '. Enter, while the focus is on the trigger or
//! in its list, is only reported (`MenuResp::enter`): what it does is the caller's.
//!
//! ```ignore
//! let r = ui.menu("size", Trigger::Line("Size: Medium"), &mut open, MenuOpts::default(), |ui| {
//!     ui.group("sizes", |ui| {
//!         for (i, s) in sizes.iter().enumerate() {
//!             let row = SelectionRow { label: s, selector: Selector::Single, selected: i == chosen, ..SelectionRow::default() };
//!             if ui.selection_row(&format!("s{i}"), row).clicked { chosen = i; close = true; }
//!         }
//!     });
//!     ui.group("more", |ui| if ui.action_row("custom", "Custom size\u{2026}").clicked { custom = true; });
//! });
//! if close { open = false; }
//! ```

use mere_base::{Cursor, Key, Rect};
use mere_layout::{Dim, Edges, Insets, Margin, Position, Side, Style};

use crate::field::{Focus, Typed};
use crate::id::Id;
use crate::line::LineLook;
use crate::tab::{TabLook, TabOpts};
use crate::tokens::{CONTROL_COMPACT, FRAME, GAP, GAP_TIGHT, INSET_TIGHT};
use crate::ui::{Paint, Resp, Ui};

/// What opens a menu.
#[derive(Copy, Clone, Debug)]
pub enum Trigger<'a> {
    /// A framed line that shows its text cut at its end and a chevron at its right (`MenuOpts::height` tall); the list
    /// is exactly as wide.
    Line(&'a str),
    /// A strip tab with a chevron after its word, drawn pressed while the list is open (the bar's "More"); the list is
    /// as wide as its rows want, at least the tab.
    Tab(&'a str),
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MenuOpts {
    /// The list opens above its trigger (a control at the bottom of a panel) instead of under it.
    pub up: bool,
    /// The trigger is faded and opens nothing; an open list closes.
    pub disabled: bool,
    /// A line's height, its frame included: `control-compact` when None.
    pub height: Option<f32>,
    /// The row, in build order, that takes the focus when a press or Space opens the list: the first when None. A
    /// list of one choice gives its chosen row, as a list box does.
    pub start: Option<usize>,
}

/// What the person did to a menu this frame.
#[derive(Copy, Clone, Debug)]
pub struct MenuResp {
    /// The trigger the list hangs from.
    pub line: Resp,
    /// The list's box while it is open.
    pub list: Option<Id>,
    pub open: bool,
    /// It opened or closed in this frame (a press on the trigger, Space, Down, Escape, a press outside, the caller).
    pub opened: bool,
    pub closed: bool,
    /// Enter was pressed while the focus was on the trigger or in the list.
    pub enter: bool,
}

/// The open list as last built: its list's box and its trigger, whose rects as the last frame laid them out are
/// what a press is judged against at the start of the next frame. Ids and not rects: on the frame a list opens, its
/// box has no rect yet when it is built.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Hung {
    pub id: Id,
    pub list: Id,
    pub line: Id,
}

/// The keys a focused row or trigger took this frame.
#[derive(Copy, Clone, Default)]
pub(crate) struct Keys {
    pub space: bool,
    /// Up (-1) or Down (+1), the last one typed.
    pub step: i32,
}

impl Ui {
    pub(crate) fn has_focus(&self, id: Id) -> bool {
        self.focus.as_ref().is_some_and(|f| f.id == id)
    }

    /// Takes the keyboard from whoever has it; a field that had it is told when it is built (`Ui::lost`).
    pub(crate) fn give_focus(&mut self, id: Id) {
        if !self.has_focus(id) {
            self.lost = self.focus.replace(Focus { id, state: None, select_all: false });
        }
    }

    /// The focus of what is not a field: in Tab's walk unless disabled, taken by a press on it, lost to a press
    /// elsewhere. Inside an open menu the one `MenuOpts::start` names (the first by default) takes it on the frame the list
    /// opens. True when it has the
    /// focus after this.
    pub(crate) fn focusable(&mut self, resp: &Resp, disabled: bool) -> bool {
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
        // After the press: the press that opened the list lies on its trigger, elsewhere for this row.
        if let Some(rows) = &mut self.menu_rows {
            rows.push(id);
            if self.menu_first == Some(rows.len() - 1) {
                self.menu_first = None;
                self.give_focus(id);
            }
        }
        self.has_focus(id)
    }

    /// The keys of a focused non-field: Tab moves the focus on (after painting, as from a field). Space, Up and Down
    /// are taken, so that the row a key on the trigger just focused does not act on the same key.
    pub(crate) fn focus_keys(&mut self, id: Id) -> Keys {
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
    pub(crate) fn row_keys(&mut self, id: Id, focused: bool) -> bool {
        let keys = if focused { self.focus_keys(id) } else { Keys::default() };
        if keys.step != 0 {
            self.menu_step = Some((id, keys.step));
        }
        keys.space
    }

    /// A menu: `trigger` opens a list above it or under it, and `rows` builds what the list holds (groups of rows)
    /// while it is open; it is not built otherwise. `open` is the caller's: a press on the trigger or Space on it flips
    /// it, Down or Up opens it, and Escape or a press outside the trigger and the list sets it false. When the list
    /// opens, a row takes the focus (`MenuOpts::start` after a press or Space, the first after Down, the last after
    /// Up); when it closes, the trigger does, unless the caller or a press outside gave the keyboard to something
    /// else. The focused line is framed in the text colour.
    pub fn menu(&mut self, key: &str, trigger: Trigger, open: &mut bool, opts: MenuOpts, rows: impl FnOnce(&mut Ui)) -> MenuResp {
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
        let tab = matches!(trigger, Trigger::Tab(_));
        // A tab keeps the strip's overlap of its neighbour's frame; a line takes its parent's width.
        let wrap = if tab {
            Style { position: Position::Relative, margin: Margin { left: Side::Px(-FRAME), ..Margin::default() }, ..Style::default() }
        } else {
            Style { column: true, w: Dim::Pct(100.0), min_w: Dim::Px(0.0), position: Position::Relative, ..Style::default() }
        };
        self.open(key, id, wrap, Paint::None);
        let lid = self.key("line");
        let resp = self.interact(lid, opts.disabled, Cursor::Hand);
        let focused = self.focusable(&resp, opts.disabled);
        let keys = if focused { self.focus_keys(lid) } else { Keys::default() };
        // Up or Down on the trigger of an open list goes into it, to its first row or its last.
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
        // The keyboard goes back to the trigger from the list, not from what the caller gave it as it closed the list (a
        // field that a pick asks for) nor from what a press outside took.
        let in_list = self.focus.as_ref().is_none_or(|f| self.menu_last_rows.contains(&f.id));
        if closed && !opts.disabled && in_list {
            self.give_focus(lid);
        }
        let has = self.has_focus(lid);
        match trigger {
            Trigger::Line(text) => {
                let look = LineLook { height: opts.height.unwrap_or(CONTROL_COMPACT), disabled: opts.disabled, focused: has };
                self.menu_line("line", lid, text, look);
            }
            Trigger::Tab(word) => {
                let lit = resp.hovered || resp.held;
                let (_, color) = self.tab_box("line", lid, lit, word, TabLook::Strip, TabOpts { pressed: *open, disabled: opts.disabled }, Margin::default());
                self.add_chevron(Side::Px(GAP), color);
                self.close();
            }
        }

        let mut list = None;
        let mut mine = Vec::new();
        if *open {
            let zero = Dim::Px(0.0);
            let mut inset = Insets { left: zero, right: zero, ..Insets::default() };
            if tab {
                // Placed once its rows have said how wide they want to be, below.
                inset = Insets { left: zero, right: Dim::Auto, ..Insets::default() };
            }
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
            // The row `start` names (Down: the first) takes the focus as it is built, unless Up asked for the last,
            // given once all are built.
            self.menu_first = match into {
                Some(1) => Some(0),
                Some(_) => None,
                None => opened.then_some(opts.start.unwrap_or(0)),
            };
            self.menu_step = None;
            self.menu_want = 0.0;
            let list_id = self.overlay("list", style, rows);
            if tab {
                // As wide as the rows want and at least the tab, moved left by what the window lacks on its right.
                // The room an absolute box gets is its parent's, the tab's: the width is set here, not left to the
                // layout, which would cut the rows to the tab.
                let line = resp.rect.unwrap_or_default();
                let width = (self.menu_want + 2.0 * FRAME).max(line.w).ceil();
                let left = (self.size().x - line.x - width).min(0.0).max(-line.x);
                let node = *self.lifts.last().expect("the list was just built");
                let s = self.tree.style_mut(node);
                s.w = Dim::Px(width);
                s.inset.left = Dim::Px(left);
            }
            mine = self.menu_rows.take().unwrap_or_default();
            self.menu_last_rows.clone_from(&mine);
            self.menu_first = None;
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
            self.menu_hung = Some(Hung { id, list: list_id, line: lid });
            self.menu_built = true;
            list = Some(list_id);
        }
        self.close();
        let ours = self.focus.as_ref().is_some_and(|f| f.id == lid || mine.contains(&f.id));
        let enter = ours && self.enter_typed();
        MenuResp { line: resp, list, open: *open, opened, closed, enter }
    }

    /// At the start of a frame, before any widget reads the pointer: a press outside the open list and its trigger
    /// closes the list and is taken, so that it reaches nothing under it.
    pub(crate) fn menu_begin_frame(&mut self) {
        self.menu_built = false;
        let Some(h) = self.menu_hung else { return };
        let Some(p) = self.input.press_pos.filter(|_| self.input.pressed) else { return };
        let inside = |id| self.rect(id).is_some_and(|r: Rect| r.contains(p));
        if inside(h.list) || inside(h.line) {
            return;
        }
        self.menu_close = Some(h.id);
        self.input.pressed = false;
        self.input.press_pos = None;
    }
}
