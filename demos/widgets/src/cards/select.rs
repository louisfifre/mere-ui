//! The Select card: the box at control height and compact, at rest, disabled and open, and a long option
//! shortened in a narrow box (components/Select/preview.html).

use std::cell::Cell as Memo;

use mere_base::{Button, Event, Rect, Vec2};
use mere_layout::{Dim, Style};
use mere_ui::{Id, MENU_ROW, SelectOpts, Theme, Ui};

use super::{Block, Card, Cell, State, render};

/// The preview's column width.
const COL: f32 = 180.0;
/// The long option's box.
const NARROW: f32 = 140.0;
/// Compact height of the preview (`height: 23`).
const COMPACT: f32 = 23.0;

/// Gets a widget clicked in a still picture. A cell is built three times in a fresh `Ui` and a widget can only
/// be hit where it stood in the last frame, so the click is made on the second build; the `Ui` does the rest.
pub(super) struct Probe {
    id: Memo<Option<Id>>,
    done: Memo<bool>,
}

impl Probe {
    pub(super) fn new() -> Probe {
        Probe { id: Memo::new(None), done: Memo::new(false) }
    }

    pub(super) fn set(&self, id: Id) {
        self.id.set(Some(id));
    }

    /// The widget's last rect, once per fresh `Ui`: None on its first build (which also rearms the probe, the
    /// same cell being built in several `Ui`s), then Some on the next, then None again.
    pub(super) fn due(&self, ui: &Ui) -> Option<Rect> {
        let Some(r) = self.id.get().and_then(|id| ui.rect(id)) else {
            self.done.set(false);
            return None;
        };
        if self.done.replace(true) { None } else { Some(r) }
    }
}

/// A click of the left button at `p`.
pub(super) fn click(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerMoved { pos: p });
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
}

pub(super) fn centre(r: Rect) -> Vec2 {
    Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn options(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// `N.select(options, 1, o)`. `open` clicks the box so that its list shows, the pointer left on the third row
/// (`hot: 2`). `width` puts the box in a narrower one.
fn select(list: &[&str], opts: SelectOpts, open: bool, width: Option<f32>) -> Cell {
    let list = options(list);
    let probe = Probe::new();
    Cell::new(State::Rest, move |ui: &mut Ui, _| {
        let due = if open { probe.due(ui) } else { None };
        if let Some(r) = due {
            click(ui, centre(r));
        }
        let mut chosen = 1;
        let mut id = None;
        let style = Style { w: width.map_or(Dim::Auto, Dim::Px), ..Style::default() };
        let mut one = |ui: &mut Ui| {
            let r = ui.select_menu("sel", &list, &mut chosen, opts);
            probe.set(r.resp.id);
            id = Some(r.resp.id);
        };
        if width.is_some() {
            ui.node("narrow", style, |ui| one(ui));
        } else {
            one(ui);
        }
        if let Some(r) = due {
            // The list opened under the box; the pointer goes to its third row, where the list's hot row
            // follows it on the next build.
            ui.event(&Event::PointerMoved { pos: Vec2::new(r.x + 12.0, r.y + r.h + 1.0 + 2.5 * MENU_ROW) });
        }
        id.unwrap()
    })
}

pub fn blocks() -> Vec<Block> {
    let three = ["Option one", "Option two", "Option three"];
    let long = ["Option one", "An option with a very long name", "Option three"];
    let compact = SelectOpts { height: Some(COMPACT), ..SelectOpts::default() };
    let off = SelectOpts { disabled: true, ..SelectOpts::default() };
    let compact_off = SelectOpts { height: Some(COMPACT), disabled: true };
    vec![
        Block::sparse(vec![
            vec![
                Some(select(&three, SelectOpts::default(), false, None)),
                Some(select(&three, off, false, None)),
                Some(select(&three, SelectOpts::default(), true, None)),
            ],
            vec![Some(select(&three, compact, false, None)), Some(select(&three, compact_off, false, None)), None],
            vec![Some(select(&long, SelectOpts::default(), false, Some(NARROW))), None, None],
        ])
        .col_width(COL),
    ]
}

pub fn card(theme: Theme) -> Card {
    render(theme, blocks())
}
