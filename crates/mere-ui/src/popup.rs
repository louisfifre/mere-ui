//! The popup list of the select box (`select_menu`) and of a strip tab (`tab_menu`): the box shows the chosen option and a click
//! opens a list under it. The list is painted by `end_frame` after the whole tree, outside every clip, so it
//! lies over whatever scrolls beneath; while it is open it takes the presses (a press elsewhere closes it and
//! reaches nothing else) and the widgets under it are not hovered.
//!
//! Subset: the pointer and Escape only. The browser's popup also answers Up, Down, Enter and typing; the
//! launcher's selects are short, so those are left out.

use mere_base::{Cursor, DrawList, Rect, Vec2};
use mere_layout::{Align, Dim, Edges, Margin, Side, Style};

use crate::id::Id;
use crate::tab::{TabLook, TabOpts};
use crate::text::{TextLook, Tone, half_leading};
use crate::tokens::{CONTROL, FADE_DISABLED, FRAME, GAP, INSET, INSET_HAIR, INSET_TIGHT};
use crate::ui::{Paint, Resp, Ui, draw_line};

/// One row of the popup list, its line centred in it.
pub const MENU_ROW: f32 = crate::tokens::MENU_ROW;
/// The box's height: a control's.
const BOX_H: f32 = CONTROL;

#[derive(Copy, Clone, Debug, Default)]
pub struct SelectOpts {
    /// Disabled: faded and no list.
    pub disabled: bool,
    /// The box's height when it is not a control's: a compact one, with `inset-hair` above and below.
    pub height: Option<f32>,
}

#[derive(Copy, Clone, Debug)]
pub struct SelectResp {
    pub resp: Resp,
    /// A row was chosen this frame (the value may be the one it was).
    pub changed: bool,
    pub open: bool,
}

/// One row of the list opened from a strip tab (`Ui::tab_menu`): a disabled row is shown faded and a press on it does
/// nothing, the list staying open.
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

/// The one open popup list.
pub(crate) struct Popup {
    pub id: Id,
    pub options: Vec<String>,
    /// Which rows are disabled (empty: none), and the width the list needs for its longest row (0: the box's).
    pub disabled: Vec<bool>,
    pub wide: f32,
    pub hot: usize,
    /// Where it was painted in the last frame, and the box it hangs from: what the pointer is judged against.
    pub rect: Rect,
    pub box_rect: Rect,
}

impl Popup {
    fn row_at(&self, p: Vec2) -> usize {
        let n = self.options.len().max(1);
        (((p.y - (self.rect.y + 1.0)) / MENU_ROW).floor().max(0.0) as usize).min(n - 1)
    }
}

impl Ui {
    /// A select box with its popup list. `chosen` is the index into `options`; a press on a row sets it.
    pub fn select_menu(&mut self, key: &str, options: &[String], chosen: &mut usize, opts: SelectOpts) -> SelectResp {
        let id = self.key(key);
        let resp = self.interact(id, opts.disabled, Cursor::Hand);
        let mut changed = false;
        if let Some((pid, row)) = self.popup_pick {
            if pid == id {
                self.popup_pick = None;
                *chosen = row.min(options.len().saturating_sub(1));
                changed = true;
            }
        }
        let is_open = self.popup.as_ref().is_some_and(|m| m.id == id);
        if opts.disabled {
            if is_open {
                self.popup = None;
            }
        } else if resp.clicked {
            self.popup = if is_open {
                None
            } else {
                let rect = resp.rect.unwrap_or_default();
                Some(Popup { id, options: options.to_vec(), disabled: Vec::new(), wide: 0.0, hot: *chosen, rect: Rect::default(), box_rect: rect })
            };
        }
        let open = self.popup.as_ref().is_some_and(|m| m.id == id);
        let t = *self.theme();
        let fade = |c| if opts.disabled { t.faded(c, FADE_DISABLED) } else { c };
        let style = Style {
            w: Dim::Pct(100.0),
            h: Dim::Px(opts.height.unwrap_or(BOX_H)),
            pad: {
                let v = FRAME + if opts.height.is_some() { INSET_HAIR } else { INSET_TIGHT };
                Edges::new(v, FRAME + INSET_TIGHT, v, FRAME + INSET)
            },
            align_items: Align::Center,
            ..Style::default()
        };
        let node = self.open(key, id, style, Paint::Box { fill: None, frame: Some(fade(t.edge)) });
        let shown = options.get(*chosen).map_or("", String::as_str);
        // The text shrinks and ends in an ellipsis before the arrow, as the browser's does.
        self.add_ellipsis(shown, TextLook::body(Tone::Text), fade(t.text), false);
        let chevron = Style {
            w: Dim::Px(8.0),
            h: Dim::Px(5.0),
            shrink: 0.0,
            margin: Margin { left: Side::Auto, ..Margin::default() },
            ..Style::default()
        };
        self.add("chevron", chevron, Paint::Chevron { color: fade(t.text) });
        self.close();
        if open {
            self.popup_node = Some(node);
            if let Some(m) = &mut self.popup {
                m.options = options.to_vec();
            }
        }
        SelectResp { resp, changed, open }
    }

    /// A tab of the strip that opens a list of rows under it (the bar's "More"): the same popup list as `select_menu`,
    /// drawn by the same code, hung from a strip tab instead of a select box. The tab is drawn pressed while the
    /// list is open. `picked` is the row chosen this frame; the app does what that row stands for.
    pub fn tab_menu(&mut self, key: &str, label: &str, rows: &[TabMenuRow]) -> TabMenuResp {
        let id = self.key(key);
        let mut picked = None;
        if let Some((pid, row)) = self.popup_pick {
            if pid == id {
                self.popup_pick = None;
                picked = Some(row.min(rows.len().saturating_sub(1)));
            }
        }
        let was_open = self.popup.as_ref().is_some_and(|m| m.id == id);
        let (resp, node, word) = self.tab_open(key, label, TabLook::Strip, TabOpts { pressed: was_open, disabled: false });
        let chevron = Style { w: Dim::Px(8.0), h: Dim::Px(5.0), shrink: 0.0, margin: Margin { left: Side::Px(GAP), ..Margin::default() }, ..Style::default() };
        self.add("chevron", chevron, Paint::Chevron { color: word });
        self.close();
        if resp.clicked {
            self.popup = if was_open {
                None
            } else {
                let rect = resp.rect.unwrap_or_default();
                Some(Popup { id, options: Vec::new(), disabled: Vec::new(), wide: 0.0, hot: 0, rect: Rect::default(), box_rect: rect })
            };
        }
        let open = self.popup.as_ref().is_some_and(|m| m.id == id);
        if open {
            self.popup_node = Some(node);
            let look = TextLook::body(Tone::Text);
            if let Some(m) = &mut self.popup {
                m.options = rows.iter().map(|r| r.text.clone()).collect();
                m.disabled = rows.iter().map(|r| r.disabled).collect();
                // The text keeps `inset` on both sides, inside the frame.
                m.wide = rows.iter().map(|r| look.width(&r.text)).fold(0.0, f32::max) + 2.0 * (INSET + FRAME);
            }
        }
        TabMenuResp { resp, picked, open }
    }

    /// At the start of a frame, before any widget reads the pointer: moves the hot row, and takes the press
    /// the popup list owns (a row picks and closes; anywhere but the box closes). Taken presses are removed from the
    /// input so no widget acts on them.
    pub(crate) fn popup_begin_frame(&mut self) {
        let Some(m) = &mut self.popup else { return };
        if let Some(p) = self.input.pos {
            if m.rect.contains(p) {
                m.hot = m.row_at(p);
            }
        }
        let Some(p) = self.input.press_pos.filter(|_| self.input.pressed) else { return };
        if m.rect.contains(p) {
            let row = m.row_at(p);
            // A disabled row takes the press and does nothing: the list stays open.
            if !m.disabled.get(row).copied().unwrap_or(false) {
                self.popup_pick = Some((m.id, row));
                self.popup = None;
            }
        } else if !m.box_rect.contains(p) {
            self.popup = None;
        } else {
            return;
        }
        self.input.pressed = false;
        self.input.press_pos = None;
    }

    /// True when the pointer is over the open popup list as it was last painted.
    pub(crate) fn under_popup(&self) -> bool {
        matches!((&self.popup, self.input.pos), (Some(m), Some(p)) if m.rect.contains(p))
    }

    /// Paints the popup list over the finished frame: under its box, or above it when the window ends first.
    pub(crate) fn paint_popup(&mut self, abs: &[Rect]) {
        let Some(node) = self.popup_node.take() else {
            // The box was not built this frame (the page changed): its list goes with it.
            self.popup = None;
            return;
        };
        let t = *self.theme();
        let Some(m) = &mut self.popup else { return };
        let b = abs[node.0];
        let h = m.options.len() as f32 * MENU_ROW + 2.0;
        let below = b.y + b.h;
        let y = if below + h <= self.size.y || b.y - h < 0.0 { below } else { b.y - h };
        // A list that needs more than its box's width (a tab's) takes it, and slides back in if the window ends first.
        let w = b.w.max(m.wide);
        let x = if m.wide > 0.0 { b.x.min(self.size.x - w).max(0.0) } else { b.x };
        let r = Rect::new(x, y, w, h);
        m.rect = r;
        m.box_rect = b;
        let look = TextLook::body(Tone::Text);
        let list: &mut DrawList = &mut self.list;
        list.fill(r, t.ground);
        list.frame(r, 1.0, t.edge);
        for (i, text) in m.options.iter().enumerate() {
            let top = r.y + 1.0 + i as f32 * MENU_ROW;
            let off = m.disabled.get(i).copied().unwrap_or(false);
            let hot = i == m.hot && !off;
            if hot {
                list.fill(Rect::new(r.x + 1.0, top, r.w - 2.0, MENU_ROW), t.select);
            }
            let color = if hot { t.select_text } else if off { t.faded(t.text, FADE_DISABLED) } else { t.text };
            draw_line(list, text, look, color, r.x + FRAME + INSET, top + (MENU_ROW - look.line) / 2.0 + half_leading(look));
        }
    }
}
