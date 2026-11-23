//! The select's menu: opening, picking, closing, and lying over a clip.

use mere_base::{Button, Cmd, DrawList, Event, Key, Mods, Rect, Vec2};
use mere_layout::{Dim, Style};
use mere_ui::tokens::{CONTROL, GAP, GAP_TIGHT};
use mere_ui::{ActionLook, ActionOpts, LIST_PAD, LIST_ROW, ScrollOpts, SelectOpts, SelectResp, Theme, Ui};

const SIZE: Vec2 = Vec2::new(400.0, 300.0);

struct Page {
    ui: Ui,
    chosen: usize,
    clicks: usize,
    scroll_y: f32,
    in_scroll: bool,
    disabled: bool,
    last: Option<SelectResp>,
    under: Option<mere_ui::Id>,
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
                let r = ui.action_with("above", "Above", ActionLook::Plain, ActionOpts::default());
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
                let style = Style { h: Dim::Px(40.0), grow: 0.0, basis: Dim::Auto, ..mere_ui::scroll_style() };
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

    /// The top of row `i` of the open menu.
    fn row_top(&self, i: usize) -> f32 {
        let b = self.box_rect();
        b.y + b.h + GAP_TIGHT + LIST_PAD + i as f32 * LIST_ROW
    }

    /// The point on row `i` of the open menu.
    fn row(&self, i: usize) -> Vec2 {
        Vec2::new(self.box_rect().x + 30.0, self.row_top(i) + LIST_ROW / 2.0)
    }
}

#[test]
fn a_click_on_the_box_opens_and_a_second_closes() {
    let mut p = Page::new();
    assert!(!p.last.unwrap().open);
    p.open();
    assert!(p.last.unwrap().open);
    let b = p.box_rect();
    assert_eq!(b.h, CONTROL);
    p.click(Vec2::new(b.x + 20.0, b.y + 10.0));
    assert!(!p.last.unwrap().open);
}

#[test]
fn the_menu_opens_with_the_chosen_row_focused_and_paints_every_option() {
    let mut p = Page::new();
    p.open();
    let texts: Vec<String> = p.list.cmds.iter().filter_map(|c| if let Cmd::Text { text, .. } = c { Some(text.clone()) } else { None }).collect();
    for o in options() {
        assert!(texts.iter().filter(|t| **t == o).count() >= 1, "{o} not drawn");
    }
    let t = Theme::dark();
    let hot: Vec<Rect> = p.list.cmds.iter().filter_map(|c| if let Cmd::Fill { rect, color } = c { (*color == t.select).then_some(*rect) } else { None }).collect();
    assert_eq!(hot.len(), 1);
    assert_eq!(hot[0].y, p.row_top(1), "row 1 (the chosen one) has the focus");
}

#[test]
fn the_pointer_over_a_row_lights_it_and_a_press_picks_and_closes() {
    let mut p = Page::new();
    p.open();
    let at = p.row(2);
    p.at(at);
    let t = Theme::dark();
    let lit_y = |p: &Page| p.list.cmds.iter().find_map(|c| if let Cmd::Fill { rect, color } = c { (*color == t.tint()).then_some(rect.y) } else { None }).unwrap();
    assert_eq!(lit_y(&p), p.row_top(2));
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
    // The application can tell that Escape is the menu's.
    assert!(p.ui.menu_open());
    p.ui.event(&Event::Key { key: Key::Escape, mods: Mods::default() });
    assert!(!p.ui.menu_open());
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
    // The scroll area is 40 px tall, the menu is 88: its ground fill must come at clip depth 0.
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


#[test]
fn the_keyboard_opens_moves_and_picks_and_the_line_gets_the_focus_back() {
    let mut p = Page::new();
    // A press gives the line the focus and opens the list on the chosen row; Escape closes it.
    p.open();
    p.ui.event(&Event::Key { key: Key::Escape, mods: Mods::default() });
    p.frame();
    assert!(!p.last.unwrap().open);
    let line = p.last.unwrap().resp.id;
    assert_eq!(p.ui.focused(), Some(line), "the closed list gives the focus back to its line");
    // Space opens it again, Down moves to the next row, Space picks it and closes.
    p.ui.event(&Event::Char { ch: ' ' });
    p.frame();
    assert!(p.last.unwrap().open, "Space on the line opens the list");
    p.ui.event(&Event::Key { key: Key::Down, mods: Mods::default() });
    p.frame();
    p.ui.event(&Event::Char { ch: ' ' });
    p.frame();
    assert_eq!(p.chosen, 2, "Down then Space picks the row after the chosen one");
    p.frame();
    assert!(!p.last.unwrap().open, "and closes the list");
    assert_eq!(p.ui.focused(), Some(line));
}

/// A text cut short in a narrow select stops `gap` before the chevron: the chevron's one rule, whatever the trigger.
#[test]
fn a_cut_text_keeps_its_gap_before_the_chevron() {
    let mut ui = Ui::new(Theme::dark());
    let opts = vec!["An option with a name far too long for its line".to_string()];
    let mut chosen = 0;
    let mut list = DrawList::default();
    for _ in 0..2 {
        ui.begin_frame(SIZE);
        ui.node("col", Style { w: Dim::Px(140.0), column: true, ..Style::default() }, |ui| {
            ui.select_menu("sel", &opts, &mut chosen, SelectOpts::default());
        });
        list = ui.end_frame().clone();
    }
    let (end, size) = list
        .cmds
        .iter()
        .find_map(|c| if let Cmd::Text { at, text, size, .. } = c { text.contains('\u{2026}').then(|| (at.x + mere_ui::text_width(text, *size), *size)) } else { None })
        .expect("the text is cut");
    assert!(size > 0.0);
    let chevron = list.cmds.iter().filter_map(|c| if let Cmd::Fill { rect, .. } = c { (rect.h == 1.0 && rect.w <= 8.0).then_some(rect.x) } else { None }).fold(f32::MAX, f32::min);
    assert!(chevron - end >= GAP - 0.5, "the text ends at {end}, the chevron starts at {chevron}");
}
