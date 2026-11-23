//! The strip tab that opens a list ("More" in the bar): opening, a disabled row, picking, and the list being as
//! wide as its longest row when the tab is narrower.

use mere_base::{Button, Cmd, Event, Rect, Vec2};
use mere_layout::{Dim, Style};
use mere_ui::tokens::GAP_TIGHT;
use mere_ui::{LIST_PAD, LIST_ROW, TabMenuResp, TabMenuRow, TabLook, TabOpts, Theme, Ui};

const SIZE: Vec2 = Vec2::new(400.0, 300.0);

struct Page {
    ui: Ui,
    rows: Vec<TabMenuRow>,
    last: Option<TabMenuResp>,
    picked: Vec<usize>,
    list: mere_base::DrawList,
}

impl Page {
    fn new() -> Page {
        let rows = vec![
            TabMenuRow { text: "Flat".into(), disabled: true },
            TabMenuRow { text: "Temperature".into(), disabled: false },
            TabMenuRow { text: "Structures".into(), disabled: false },
        ];
        let mut p = Page { ui: Ui::new(Theme::dark()), rows, last: None, picked: Vec::new(), list: Default::default() };
        p.frame();
        p
    }

    fn frame(&mut self) {
        self.ui.begin_frame(SIZE);
        let rows = self.rows.clone();
        let (last, picked) = (&mut self.last, &mut self.picked);
        self.ui.node("row", Style { w: Dim::Px(300.0), ..Style::default() }, |ui| {
            ui.tab("first", "Biomes", TabLook::Strip, TabOpts::default());
            let r = ui.tab_menu("more", "More", &rows);
            picked.extend(r.picked);
            *last = Some(r);
        });
        self.list = self.ui.end_frame().clone();
    }

    fn click(&mut self, p: Vec2) {
        self.ui.event(&Event::PointerMoved { pos: p });
        self.ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
        self.frame();
        self.ui.event(&Event::PointerUp { pos: p, button: Button::Left });
        self.frame();
    }

    fn tab(&self) -> Rect {
        self.ui.rect(self.last.unwrap().resp.id).unwrap()
    }

    fn open(&mut self) {
        let b = self.tab();
        self.click(Vec2::new(b.x + 10.0, b.y + 10.0));
    }

    fn row(&self, i: usize) -> Vec2 {
        let b = self.tab();
        Vec2::new(b.x + 20.0, b.y + b.h + GAP_TIGHT + LIST_PAD + (i as f32 + 0.5) * LIST_ROW)
    }

    /// The list's frame: the rectangle painted in the edge colour under the tab.
    fn list_rect(&self) -> Rect {
        let t = Theme::dark();
        self.list.cmds.iter().rev().find_map(|c| if let Cmd::Fill { rect, color } = c { (*color == t.ground && rect.h > 30.0).then_some(*rect) } else { None }).expect("the list is painted")
    }
}

#[test]
fn a_click_opens_the_list_and_a_second_click_closes_it() {
    let mut p = Page::new();
    p.open();
    assert!(p.last.unwrap().open);
    p.open();
    assert!(!p.last.unwrap().open);
}

#[test]
fn the_list_is_as_wide_as_its_longest_row_and_starts_at_the_tab() {
    let mut p = Page::new();
    p.open();
    let (b, l) = (p.tab(), p.list_rect());
    assert!(b.w < 11.0 * 7.2, "the tab is narrower than the longest row ({})", b.w);
    assert!(l.w >= 11.0 * 7.2, "{l:?}");
    // An overlay lies on whole pixels: the tab, after words of fractional widths, may not.
    assert_eq!(l.x, b.x.round());
    assert_eq!(l.y, (b.y + b.h + GAP_TIGHT).round());
}

#[test]
fn a_press_on_a_disabled_row_does_nothing_and_the_list_stays_open() {
    let mut p = Page::new();
    p.open();
    let at = p.row(0);
    p.click(at);
    assert!(p.last.unwrap().open, "the list stays open");
    assert!(p.picked.is_empty());
}

#[test]
fn a_press_on_an_enabled_row_picks_it_once_and_closes() {
    let mut p = Page::new();
    p.open();
    let at = p.row(2);
    p.click(at);
    assert_eq!(p.picked, vec![2]);
    assert!(!p.last.unwrap().open);
}
