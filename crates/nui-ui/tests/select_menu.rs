//! The select's menu: opening, picking, closing, and lying over a clip.

use nui_base::{Button, Cmd, DrawList, Event, Key, Mods, Rect, Vec2};
use nui_layout::{Dim, Style};
use nui_ui::{ButtonLook, ButtonOpts, MENU_ROW, ScrollOpts, SelectOpts, SelectResp, Theme, Ui};

const SIZE: Vec2 = Vec2::new(400.0, 300.0);

struct Page {
    ui: Ui,
    chosen: usize,
    clicks: usize,
    scroll_y: f32,
    in_scroll: bool,
    disabled: bool,
    last: Option<SelectResp>,
    under: Option<nui_ui::Id>,
    list: DrawList,
}

fn options() -> Vec<String> {
    ["the game menu", "world: New World", "a server"].map(String::from).to_vec()
}

impl Page {
    fn new() -> Page {
        let mut p = Page { ui: Ui::new(Theme::dark()), chosen: 1, clicks: 0, scroll_y: 0.0, in_scroll: false, disabled: false, last: None, under: None, list: DrawList::default() };
        p.frame();
        p
    }

    fn frame(&mut self) {
        self.ui.begin_frame(SIZE);
        let opts = options();
        let (chosen, disabled, in_scroll) = (&mut self.chosen, self.disabled, self.in_scroll);
        let (clicks, last, under) = (&mut self.clicks, &mut self.last, &mut self.under);
        let mut body = |ui: &mut Ui| {
            // A button above the select: a press that closes the menu must not click it.
            if !in_scroll {
                let r = ui.button_with("above", "Above", ButtonLook::Plain, ButtonOpts::default());
                *under = Some(r.id);
                if r.clicked {
                    *clicks += 1;
                }
            }
            let sel = ui.select_menu("sel", &opts, chosen, SelectOpts { disabled, ..SelectOpts::default() });
            *last = Some(sel);
        };
        let root = Style { w: Dim::Px(200.0), column: true, ..Style::default() };
        if self.in_scroll {
            let y = &mut self.scroll_y;
            self.ui.node("col", root, |ui| {
                let style = Style { h: Dim::Px(40.0), grow: 0.0, basis: Dim::Auto, ..nui_ui::scroll_style() };
                ui.scroll("sc", y, style, ScrollOpts { fade: 0.0 }, &mut body);
            });
        } else {
            self.ui.node("col", root, &mut body);
        }
        self.list = self.ui.end_frame().clone();
    }

    fn box_rect(&self) -> Rect {
        self.ui.rect(self.last.unwrap().resp.id).unwrap()
    }

    fn at(&mut self, p: Vec2) {
        self.ui.event(&Event::PointerMoved { pos: p });
        self.frame();
    }

    fn click(&mut self, p: Vec2) {
        self.ui.event(&Event::PointerMoved { pos: p });
        self.ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
        self.frame();
        self.ui.event(&Event::PointerUp { pos: p, button: Button::Left });
        self.frame();
    }

    fn open(&mut self) {
        let b = self.box_rect();
        self.click(Vec2::new(b.x + 20.0, b.y + 10.0));
    }

    /// The point on row `i` of the open menu.
    fn row(&self, i: usize) -> Vec2 {
        let b = self.box_rect();
        Vec2::new(b.x + 30.0, b.y + b.h + 1.0 + i as f32 * MENU_ROW + 10.0)
    }
}

#[test]
fn a_click_on_the_box_opens_and_a_second_closes() {
    let mut p = Page::new();
    assert!(!p.last.unwrap().open);
    p.open();
    assert!(p.last.unwrap().open);
    let b = p.box_rect();
    assert_eq!(b.h, 25.0);
    p.click(Vec2::new(b.x + 20.0, b.y + 10.0));
    assert!(!p.last.unwrap().open);
}

#[test]
fn the_menu_opens_with_the_chosen_row_hot_and_paints_every_option() {
    let mut p = Page::new();
    p.open();
    let texts: Vec<String> = p.list.cmds.iter().filter_map(|c| if let Cmd::Text { text, .. } = c { Some(text.clone()) } else { None }).collect();
    for o in options() {
        assert!(texts.iter().filter(|t| **t == o).count() >= 1, "{o} not drawn");
    }
    let t = Theme::dark();
    let hot: Vec<Rect> = p.list.cmds.iter().filter_map(|c| if let Cmd::Fill { rect, color } = c { (*color == t.select).then_some(*rect) } else { None }).collect();
    assert_eq!(hot.len(), 1);
    let b = p.box_rect();
    assert_eq!(hot[0].y, b.y + b.h + 1.0 + MENU_ROW, "row 1 (the chosen one) is hot");
}

#[test]
fn the_pointer_over_a_row_makes_it_hot_and_a_press_picks_and_closes() {
    let mut p = Page::new();
    p.open();
    let at = p.row(2);
    p.at(at);
    let t = Theme::dark();
    let hot_y = |p: &Page| p.list.cmds.iter().find_map(|c| if let Cmd::Fill { rect, color } = c { (*color == t.select).then_some(rect.y) } else { None }).unwrap();
    let b = p.box_rect();
    assert_eq!(hot_y(&p), b.y + b.h + 1.0 + 2.0 * MENU_ROW);
    p.click(at);
    assert!(!p.last.unwrap().open);
    assert_eq!(p.chosen, 2);
}

#[test]
fn a_press_outside_closes_and_the_button_under_it_is_not_clicked() {
    let mut p = Page::new();
    p.open();
    let btn = p.ui.rect(p.under.unwrap()).unwrap();
    p.click(Vec2::new(btn.x + 4.0, btn.y + 4.0));
    assert!(!p.last.unwrap().open);
    assert_eq!(p.clicks, 0);
}

#[test]
fn a_press_over_the_menu_does_not_click_the_button_beneath_it() {
    let mut p = Page::new();
    p.open();
    let at = p.row(0);
    p.click(at);
    assert_eq!(p.chosen, 0);
    assert_eq!(p.clicks, 0, "the press on a row is the menu's");
}

#[test]
fn escape_closes() {
    let mut p = Page::new();
    p.open();
    assert!(p.last.unwrap().open);
    p.ui.event(&Event::Key { key: Key::Escape, mods: Mods::default() });
    p.frame();
    assert!(!p.last.unwrap().open);
    assert_eq!(p.chosen, 1);
}

#[test]
fn the_menu_is_drawn_outside_a_scroll_areas_clip() {
    let mut p = Page::new();
    p.in_scroll = true;
    p.frame();
    p.frame();
    p.open();
    assert!(p.last.unwrap().open);
    // The scroll area is 40 px tall, the menu is 62: its ground fill must come at clip depth 0.
    let t = Theme::dark();
    let mut depth = 0i32;
    let mut found = None;
    for c in &p.list.cmds {
        match c {
            Cmd::PushClip { .. } => depth += 1,
            Cmd::PopClip => depth -= 1,
            Cmd::Fill { rect, color } if *color == t.ground && rect.h > 50.0 => found = Some((depth, *rect)),
            _ => {}
        }
    }
    let (depth, rect) = found.expect("the menu's ground");
    assert_eq!(depth, 0);
    assert!(rect.y + rect.h > 40.0, "it reaches beyond the 40 px area");
}

#[test]
fn a_disabled_select_does_not_open() {
    let mut p = Page::new();
    p.disabled = true;
    p.frame();
    p.open();
    assert!(!p.last.unwrap().open);
}
