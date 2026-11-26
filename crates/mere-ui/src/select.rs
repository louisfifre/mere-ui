//! The two short lists of the kit that apply at once, built on the menu (menu.rs) so that the kit has one system of
//! open lists: the select (`select_menu`, the pick of one short text, opened from a line) and the list of a strip tab
//! (`tab_menu`, the bar's "More"). Each is a menu holding one group of immediate-selection rows (row.rs) that closes on
//! its pick; it keeps its own open state (`Ui::opened`), since the caller only wants the pick. So both take the
//! keyboard (Space, Up, Down, Escape) and the focus as the menu does, and lie in an overlay over what scrolls beneath.

use crate::menu::{MenuOpts, Trigger};
use crate::row::ImmediateRow;
use crate::tokens::CONTROL;
use crate::ui::{Resp, Ui};

#[derive(Copy, Clone, Debug, Default)]
pub struct SelectOpts {
    /// Disabled: faded and no list.
    pub disabled: bool,
    /// The line's height when it is not a control's (`control-compact` in a dense row).
    pub height: Option<f32>,
    /// The list opens above the line instead of under it.
    pub up: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct SelectResp {
    pub resp: Resp,
    /// A row was chosen this frame (the value may be the one it was).
    pub changed: bool,
    pub open: bool,
}

/// One row of the list opened from a strip tab (`Ui::tab_menu`): a disabled row is shown faded and takes no press,
/// the list staying open.
#[derive(Clone, Debug, PartialEq)]
pub struct TabMenuRow {
    pub text: String,
    pub disabled: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct TabMenuResp {
    pub resp: Resp,
    /// The row chosen this frame, as an index into the rows given.
    pub picked: Option<usize>,
    pub open: bool,
}

impl Ui {
    /// Whether the list keyed `key` in the open container was open, as the menu built with it last left it.
    fn was_open(&self, key: &str) -> bool {
        self.opened.contains(&self.peek_key(key))
    }

    fn keep_open(&mut self, key: &str, open: bool) {
        let id = self.peek_key(key);
        if open {
            self.opened.insert(id);
        } else {
            self.opened.remove(&id);
        }
    }

    /// A select: a line showing the chosen option that opens the options under it, as wide as it, the chosen one
    /// marked and focused when it opens. `chosen` is the index into `options`; a press on a row (or Space on the focused one) sets it and closes
    /// the list.
    pub fn select_menu(&mut self, key: &str, options: &[String], chosen: &mut usize, opts: SelectOpts) -> SelectResp {
        let mut open = self.was_open(key);
        let mut pick = None;
        let shown = options.get(*chosen).map_or("", String::as_str);
        let menu = MenuOpts { up: opts.up, disabled: opts.disabled, height: Some(opts.height.unwrap_or(CONTROL)), start: Some(*chosen), over: None };
        let current = *chosen;
        let r = self.menu(key, Trigger::Line(shown), &mut open, menu, |ui| {
            ui.group("options", |ui| {
                for (i, o) in options.iter().enumerate() {
                    let row = ImmediateRow { label: o, selected: i == current, ..ImmediateRow::default() };
                    if ui.immediate_row(&i.to_string(), row).clicked {
                        pick = Some(i);
                    }
                }
            });
        });
        if let Some(i) = pick {
            *chosen = i;
            open = false;
        }
        self.keep_open(key, open);
        SelectResp { resp: r.line, changed: pick.is_some(), open }
    }

    /// A tab of the strip that opens a list of rows under it (the bar's "More"), as wide as its rows want and at
    /// least the tab. The tab is drawn pressed while the list is open. `picked` is the row chosen this frame, which
    /// closes the list; the app does what that row stands for.
    pub fn tab_menu(&mut self, key: &str, label: &str, rows: &[TabMenuRow]) -> TabMenuResp {
        let mut open = self.was_open(key);
        let mut picked = None;
        let r = self.menu(key, Trigger::Tab(label), &mut open, MenuOpts::default(), |ui| {
            ui.group("rows", |ui| {
                for (i, row) in rows.iter().enumerate() {
                    let row = ImmediateRow { label: &row.text, disabled: row.disabled, ..ImmediateRow::default() };
                    if ui.immediate_row(&i.to_string(), row).clicked {
                        picked = Some(i);
                    }
                }
            });
        });
        if picked.is_some() {
            open = false;
        }
        self.keep_open(key, open);
        TabMenuResp { resp: r.line, picked, open }
    }
}
