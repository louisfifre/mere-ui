//! The text field and the number field: focus, caret, selection, scrolling, the keys, the clipboard, Tab.
//! No GPU: the draw list is read as data. Every number is derived in a comment from the geometry below.
//!
//! The field sits at (20, 20), border-box 214 wide and 26 tall (Text look: padding 3 + border 1 on top and
//! bottom, 6 + 1 left and right). So the content box is x 27, y 24, w 200, h 18. A char is 7.2 wide (0.6 em
//! of 12 px), and the glyph box (ascent + descent, 1.3 em = 15.6) sits 1.2 below the content top, so the
//! caret and the selection are 15.6 tall at y 25.2.

use std::cell::RefCell;
use std::rc::Rc;

use nui_base::{Button, Clipboard, Cmd, Color, DrawList, Event, Key, Mods, Rect, Vec2};
use nui_layout::{Dim, Insets, Position, Style};
use nui_ui::{FieldLook, FieldOpts, FieldResp, Id, MemoryClipboard, NumberSpec, Theme, Ui};

const SIZE: Vec2 = Vec2::new(860.0, 440.0);
const NONE: Mods = Mods { ctrl: false, shift: false };
const CTRL: Mods = Mods { ctrl: true, shift: false };
const SHIFT: Mods = Mods { ctrl: false, shift: true };

fn frame(ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> DrawList {
    ui.begin_frame(SIZE);
    build(ui);
    ui.end_frame().clone()
}

fn at(x: f32, y: f32, style: Style) -> Style {
    Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(x), top: Dim::Px(y), right: Dim::Auto, bottom: Dim::Auto },
        ..style
    }
}

fn mv(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerMoved { pos: p });
}
fn down(ui: &mut Ui, p: Vec2, clicks: u8) {
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks });
}
fn up(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
}
fn key(ui: &mut Ui, k: Key, m: Mods) {
    ui.event(&Event::Key { key: k, mods: m });
}
fn typ(ui: &mut Ui, s: &str) {
    for ch in s.chars() {
        ui.event(&Event::Char { ch });
    }
}
fn keys(ui: &mut Ui, k: Key, m: Mods, n: usize) {
    for _ in 0..n {
        key(ui, k, m);
    }
}

fn fills(l: &DrawList) -> Vec<(Rect, Color)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Fill { rect, color } = c { Some((*rect, *color)) } else { None }).collect()
}
fn texts(l: &DrawList) -> Vec<(Vec2, String, Color)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Text { at, text, color, .. } = c { Some((*at, text.clone(), *color)) } else { None }).collect()
}
fn frames_of(l: &DrawList) -> Vec<Color> {
    l.cmds.iter().filter_map(|c| if let Cmd::Frame { color, .. } = c { Some(*color) } else { None }).collect()
}
fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-3, "{a} != {b}");
}
fn rect_is(r: Rect, x: f32, y: f32, w: f32, h: f32) {
    near(r.x, x);
    near(r.y, y);
    near(r.w, w);
    near(r.h, h);
}

/// The pointer's x for a caret at char column `col` of a field at x 20: 27 + 7.2 col.
fn col_x(col: f32) -> f32 {
    27.0 + 7.2 * col
}
const MID_Y: f32 = 37.0;

/// One text field at (20, 20), 214 wide.
fn one(ui: &mut Ui, s: &mut String, opts: FieldOpts) -> (DrawList, FieldResp) {
    let mut r = None;
    let list = frame(ui, |ui| {
        ui.node("cell", at(20.0, 20.0, Style::default()), |ui| {
            r = Some(ui.text_field("f", s, FieldOpts { width: Dim::Px(214.0), ..opts }));
        });
    });
    (list, r.unwrap())
}

fn plain(ui: &mut Ui, s: &mut String) -> (DrawList, FieldResp) {
    one(ui, s, FieldOpts::default())
}

/// A field holding `text`, clicked at char column `col`, after one frame to know where it is.
fn focused_at(text: &str, col: f32) -> (Ui, String) {
    let mut ui = Ui::new(Theme::dark());
    let mut s = text.to_string();
    plain(&mut ui, &mut s);
    let p = Vec2::new(col_x(col), MID_Y);
    mv(&mut ui, p);
    down(&mut ui, p, 1);
    up(&mut ui, p);
    let (_, r) = plain(&mut ui, &mut s);
    assert!(r.focused);
    (ui, s)
}

fn focused_end(text: &str) -> (Ui, String) {
    focused_at(text, text.chars().count() as f32)
}

/// The select-coloured fill, if any.
fn selection(l: &DrawList, t: &Theme) -> Option<Rect> {
    fills(l).into_iter().find(|f| f.1 == t.select).map(|f| f.0)
}
fn caret(l: &DrawList, t: &Theme) -> Option<Rect> {
    fills(l).into_iter().find(|f| f.1 == t.text).map(|f| f.0)
}

#[derive(Clone, Default)]
struct Shared(Rc<RefCell<Option<String>>>);
impl Clipboard for Shared {
    fn get(&mut self) -> Option<String> {
        self.0.borrow().clone()
    }
    fn set(&mut self, text: &str) {
        *self.0.borrow_mut() = Some(text.to_string());
    }
}

// ---- 1-5: focus, typing, caret, selection ----

#[test]
fn a_click_focuses_and_shows_a_caret() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let mut s = "abcdefghij".to_string();
    let (list, r) = plain(&mut ui, &mut s);
    assert!(!r.focused);
    assert_eq!(frames_of(&list), vec![t.edge]);
    rect_is(ui.rect(r.resp.id).unwrap(), 20.0, 20.0, 214.0, 26.0);
    // Column 3: x = 27 + 21.6 = 48.6, which the caret rounds to 49.
    let p = Vec2::new(col_x(3.0), MID_Y);
    mv(&mut ui, p);
    down(&mut ui, p, 1);
    up(&mut ui, p);
    let (list, r) = plain(&mut ui, &mut s);
    assert!(r.focused);
    assert_eq!(ui.focused(), Some(r.resp.id));
    assert_eq!(frames_of(&list), vec![t.text]);
    let f = fills(&list);
    assert_eq!(f.len(), 1);
    rect_is(f[0].0, 49.0, 25.2, 1.0, 15.6);
    assert_eq!(f[0].1, t.text);
}

#[test]
fn typing_edits_the_string_and_moves_the_caret() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_at("", 0.0);
    typ(&mut ui, "1234567890");
    let (list, r) = plain(&mut ui, &mut s);
    assert_eq!(s, "1234567890");
    assert!(r.changed);
    // 10 chars: 27 + 72 = 99, and the text is 72 wide, inside the 200.
    rect_is(caret(&list, &t).unwrap(), 99.0, 25.2, 1.0, 15.6);
    let (_, r) = plain(&mut ui, &mut s);
    assert!(!r.changed, "a quiet frame changes nothing");
}

#[test]
fn home_and_right_move_the_caret() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_end("1234567890");
    key(&mut ui, Key::Home, NONE);
    keys(&mut ui, Key::Right, NONE, 4);
    let (list, _) = plain(&mut ui, &mut s);
    // 27 + 4 x 7.2 = 55.8, rounded to 56.
    rect_is(caret(&list, &t).unwrap(), 56.0, 25.2, 1.0, 15.6);
}

#[test]
fn shift_right_selects_and_the_caret_hides() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_end("1234567890");
    key(&mut ui, Key::Home, NONE);
    keys(&mut ui, Key::Right, NONE, 2);
    keys(&mut ui, Key::Right, SHIFT, 4);
    let (list, _) = plain(&mut ui, &mut s);
    // Columns 2..6: from 27 + 14.4 = 41.4, 4 x 7.2 = 28.8 wide.
    rect_is(selection(&list, &t).unwrap(), 41.4, 25.2, 28.8, 15.6);
    assert_eq!(fills(&list).len(), 1, "no caret over a selection");
    // The text is drawn again in the selection's colour, cut to the selection.
    let at = list.cmds.iter().position(|c| matches!(c, Cmd::Text { color, .. } if *color == t.select_text)).expect("select text");
    match &list.cmds[at - 1] {
        Cmd::PushClip { rect } => rect_is(*rect, 41.4, 25.2, 28.8, 15.6),
        c => panic!("expected a clip before the selected text, got {c:?}"),
    }
    assert!(matches!(list.cmds[at + 1], Cmd::PopClip));
}

#[test]
fn left_and_right_collapse_a_selection_to_its_ends() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_end("1234567890");
    key(&mut ui, Key::Home, NONE);
    keys(&mut ui, Key::Right, NONE, 2);
    keys(&mut ui, Key::Right, SHIFT, 4);
    plain(&mut ui, &mut s);
    key(&mut ui, Key::Left, NONE);
    let (list, _) = plain(&mut ui, &mut s);
    // To the near end, column 2: 27 + 14.4 = 41.4 -> 41.
    rect_is(caret(&list, &t).unwrap(), 41.0, 25.2, 1.0, 15.6);
    keys(&mut ui, Key::Right, SHIFT, 4);
    plain(&mut ui, &mut s);
    key(&mut ui, Key::Right, NONE);
    let (list, _) = plain(&mut ui, &mut s);
    // To the far end, column 6: 27 + 43.2 = 70.2 -> 70.
    rect_is(caret(&list, &t).unwrap(), 70.0, 25.2, 1.0, 15.6);
}

// ---- 6-8: words, deleting, limits ----

#[test]
fn ctrl_arrows_jump_by_word() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_end("deep ocean_2 x");
    // Word starts at columns 0, 5 and 13; the end is 14. x = 27 + 7.2 col: 27, 63, 120.6 -> 121, 127.8 -> 128.
    for (k, col_px) in [(Key::Left, 121.0), (Key::Left, 63.0), (Key::Left, 27.0), (Key::Right, 63.0), (Key::Right, 121.0), (Key::Right, 128.0)] {
        key(&mut ui, k, CTRL);
        let (list, _) = plain(&mut ui, &mut s);
        near(caret(&list, &t).unwrap().x, col_px);
    }
}

#[test]
fn backspace_and_delete_with_and_without_a_selection() {
    let (mut ui, mut s) = focused_end("abcdef");
    key(&mut ui, Key::Backspace, NONE);
    plain(&mut ui, &mut s);
    assert_eq!(s, "abcde");
    key(&mut ui, Key::Home, NONE);
    key(&mut ui, Key::Delete, NONE);
    plain(&mut ui, &mut s);
    assert_eq!(s, "bcde");
    keys(&mut ui, Key::Right, SHIFT, 2);
    key(&mut ui, Key::Backspace, NONE);
    plain(&mut ui, &mut s);
    assert_eq!(s, "de");
    let (mut ui, mut s) = focused_end("the lazy dog");
    key(&mut ui, Key::Backspace, CTRL);
    let (_, r) = plain(&mut ui, &mut s);
    assert_eq!(s, "the lazy ");
    assert!(r.changed);
}

#[test]
fn max_chars_cuts_typing_and_paste() {
    let opts = FieldOpts { max_chars: 5, ..FieldOpts::default() };
    let mut ui = Ui::new(Theme::dark());
    let mut s = String::new();
    one(&mut ui, &mut s, opts);
    let p = Vec2::new(col_x(0.0), MID_Y);
    down(&mut ui, p, 1);
    up(&mut ui, p);
    one(&mut ui, &mut s, opts);
    typ(&mut ui, "1234567");
    one(&mut ui, &mut s, opts);
    assert_eq!(s, "12345");
    ui.set_clipboard(Box::new(MemoryClipboard(Some("abcdefgh".into()))));
    key(&mut ui, Key::A, CTRL);
    key(&mut ui, Key::V, CTRL);
    one(&mut ui, &mut s, opts);
    // The pasted text replaces the selection, so all 5 chars of room are free again.
    assert_eq!(s, "abcde");
}

// ---- 9-10: clipboard, undo ----

#[test]
fn copy_paste_and_cut_go_through_the_clipboard() {
    let board = Shared::default();
    let (mut ui, mut s) = focused_end("abcd");
    ui.set_clipboard(Box::new(board.clone()));
    key(&mut ui, Key::A, CTRL);
    key(&mut ui, Key::C, CTRL);
    key(&mut ui, Key::End, NONE);
    key(&mut ui, Key::V, CTRL);
    plain(&mut ui, &mut s);
    assert_eq!(s, "abcdabcd");
    assert_eq!(board.get_now().as_deref(), Some("abcd"));
    key(&mut ui, Key::Home, NONE);
    keys(&mut ui, Key::Right, SHIFT, 2);
    key(&mut ui, Key::X, CTRL);
    plain(&mut ui, &mut s);
    assert_eq!(s, "cdabcd");
    assert_eq!(board.get_now().as_deref(), Some("ab"));
}

impl Shared {
    fn get_now(&self) -> Option<String> {
        self.0.borrow().clone()
    }
}

#[test]
fn undo_steps_back_and_redo_forward() {
    let (mut ui, mut s) = focused_at("", 0.0);
    typ(&mut ui, "abc");
    key(&mut ui, Key::Z, CTRL);
    plain(&mut ui, &mut s);
    assert_eq!(s, "");
    key(&mut ui, Key::Z, CTRL);
    plain(&mut ui, &mut s);
    assert_eq!(s, "abc");
    // Typing and then deleting are two steps.
    let (mut ui, mut s) = focused_at("", 0.0);
    typ(&mut ui, "ab");
    key(&mut ui, Key::Backspace, NONE);
    plain(&mut ui, &mut s);
    assert_eq!(s, "a");
    key(&mut ui, Key::Z, CTRL);
    plain(&mut ui, &mut s);
    assert_eq!(s, "ab");
}

// ---- 11-12: scrolling, the mouse ----

#[test]
fn a_long_text_scrolls_to_keep_the_caret_in_view() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_at("", 0.0);
    typ(&mut ui, &"0123456789".repeat(3));
    let (list, _) = plain(&mut ui, &mut s);
    // 30 chars = 216 wide in 200: the caret at 216 pushes the text 16 left.
    let text = texts(&list).into_iter().find(|x| x.1 == s).unwrap();
    near(text.0.x, 27.0 - 16.0);
    // The caret: 27 + 216 - 16 = 227, the very right edge of the content box plus its one extra pixel.
    rect_is(caret(&list, &t).unwrap(), 227.0, 25.2, 1.0, 15.6);
    key(&mut ui, Key::Home, NONE);
    let (list, _) = plain(&mut ui, &mut s);
    near(texts(&list).into_iter().find(|x| x.1 == s).unwrap().0.x, 27.0);
    near(caret(&list, &t).unwrap().x, 27.0);
}

#[test]
fn double_triple_click_and_drag_select() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let mut s = "deep ocean".to_string();
    plain(&mut ui, &mut s);
    // 27 + 7.2 x 7.5 = 81: in the middle of the char at column 7, an "e" of "ocean" (columns 5..10).
    let p = Vec2::new(col_x(7.5), MID_Y);
    mv(&mut ui, p);
    for clicks in [1, 2] {
        down(&mut ui, p, clicks);
        up(&mut ui, p);
        plain(&mut ui, &mut s);
    }
    // Columns 5..10: from 27 + 36 = 63, 36 wide.
    let (list, _) = plain(&mut ui, &mut s);
    rect_is(selection(&list, &t).unwrap(), 63.0, 25.2, 36.0, 15.6);
    down(&mut ui, p, 3);
    up(&mut ui, p);
    let (list, _) = plain(&mut ui, &mut s);
    rect_is(selection(&list, &t).unwrap(), 27.0, 25.2, 72.0, 15.6);
    // A drag: press at column 1, move to column 4 with the button down, release.
    let (a, b) = (Vec2::new(col_x(1.0), MID_Y), Vec2::new(col_x(4.0), MID_Y));
    mv(&mut ui, a);
    down(&mut ui, a, 1);
    plain(&mut ui, &mut s);
    mv(&mut ui, b);
    plain(&mut ui, &mut s);
    up(&mut ui, b);
    let (list, _) = plain(&mut ui, &mut s);
    // Columns 1..4: from 27 + 7.2 = 34.2, 21.6 wide.
    rect_is(selection(&list, &t).unwrap(), 34.2, 25.2, 21.6, 15.6);
}

// ---- 13-16: leaving, Tab, pressing elsewhere, outside changes ----

#[test]
fn enter_submits_and_escape_cancels_both_leaving_the_field() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_at("", 0.0);
    typ(&mut ui, "x");
    key(&mut ui, Key::Enter, NONE);
    let (list, r) = plain(&mut ui, &mut s);
    assert!(r.submitted && r.blurred && !r.focused && !r.cancelled);
    assert_eq!(s, "x");
    assert_eq!(frames_of(&list), vec![t.edge]);
    assert_eq!(ui.focused(), None);
    let (mut ui, mut s) = focused_at("", 0.0);
    typ(&mut ui, "x");
    key(&mut ui, Key::Escape, NONE);
    let (list, r) = plain(&mut ui, &mut s);
    assert!(r.cancelled && r.blurred && !r.focused && !r.submitted);
    assert_eq!(s, "x", "a text field keeps what was typed");
    assert_eq!(frames_of(&list), vec![t.edge]);
}

/// Two fields, the first at (20, 20) and the second at (20, 80), built in either order.
fn two(ui: &mut Ui, a: &mut String, b: &mut String, second_first: bool) -> (DrawList, FieldResp, FieldResp) {
    let (mut ra, mut rb) = (None, None);
    let w = FieldOpts { width: Dim::Px(214.0), ..FieldOpts::default() };
    let list = frame(ui, |ui| {
        if second_first {
            ui.node("b", at(20.0, 80.0, Style::default()), |ui| rb = Some(ui.text_field("f", b, w)));
            ui.node("a", at(20.0, 20.0, Style::default()), |ui| ra = Some(ui.text_field("f", a, w)));
        } else {
            ui.node("a", at(20.0, 20.0, Style::default()), |ui| ra = Some(ui.text_field("f", a, w)));
            ui.node("b", at(20.0, 80.0, Style::default()), |ui| rb = Some(ui.text_field("f", b, w)));
        }
    });
    (list, ra.unwrap(), rb.unwrap())
}

fn click(ui: &mut Ui, p: Vec2) {
    mv(ui, p);
    down(ui, p, 1);
    up(ui, p);
}

#[test]
fn tab_moves_to_the_next_field_built_with_its_text_selected() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let (mut a, mut b) = ("alpha".to_string(), "beta".to_string());
    two(&mut ui, &mut a, &mut b, false);
    click(&mut ui, Vec2::new(col_x(0.0), MID_Y));
    let (_, ra, rb) = two(&mut ui, &mut a, &mut b, false);
    assert!(ra.focused && !rb.focused);
    key(&mut ui, Key::Tab, NONE);
    let (_, ra, rb) = two(&mut ui, &mut a, &mut b, false);
    assert!(ra.blurred && !ra.focused && !rb.focused);
    assert!(ui.want_redraw(), "the focus moved after the picture was drawn");
    assert_eq!(ui.focused(), Some(rb.resp.id));
    let (list, ra, rb) = two(&mut ui, &mut a, &mut b, false);
    assert!(!ra.focused && rb.focused);
    assert!(!ui.want_redraw());
    // "beta" in the second field (content y 84): all four chars, 27 .. 27 + 28.8, at y 85.2.
    rect_is(selection(&list, &t).unwrap(), 27.0, 85.2, 28.8, 15.6);
    // Shift+Tab goes back, and the first field selects all of "alpha" (36 wide).
    key(&mut ui, Key::Tab, SHIFT);
    two(&mut ui, &mut a, &mut b, false);
    assert!(ui.want_redraw());
    let (list, ra, _) = two(&mut ui, &mut a, &mut b, false);
    assert!(ra.focused);
    rect_is(selection(&list, &t).unwrap(), 27.0, 25.2, 36.0, 15.6);
    // Tab from the last wraps to the first: the second is focused again, Tab again comes back here.
    key(&mut ui, Key::Tab, NONE);
    two(&mut ui, &mut a, &mut b, false);
    two(&mut ui, &mut a, &mut b, false);
    assert_eq!(ui.focused(), Some(rb.resp.id));
    key(&mut ui, Key::Tab, NONE);
    two(&mut ui, &mut a, &mut b, false);
    assert_eq!(ui.focused(), Some(ra.resp.id), "wrapped round");
}

#[test]
fn a_press_outside_blurs_and_a_press_on_another_field_moves_the_focus_at_once() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let mut s = "abc".to_string();
    plain(&mut ui, &mut s);
    click(&mut ui, Vec2::new(col_x(1.0), MID_Y));
    let (_, r) = plain(&mut ui, &mut s);
    assert!(r.focused);
    click(&mut ui, Vec2::new(500.0, 300.0));
    let (list, r) = plain(&mut ui, &mut s);
    assert!(r.blurred && !r.focused);
    assert_eq!(frames_of(&list), vec![t.edge]);

    for second_first in [false, true] {
        let mut ui = Ui::new(t);
        let (mut a, mut b) = ("alpha".to_string(), "beta".to_string());
        two(&mut ui, &mut a, &mut b, second_first);
        click(&mut ui, Vec2::new(col_x(0.0), MID_Y));
        let (_, ra, _) = two(&mut ui, &mut a, &mut b, second_first);
        assert!(ra.focused);
        // The press lands on the second field (content y 84..102).
        click(&mut ui, Vec2::new(col_x(0.0), 93.0));
        let (list, ra, rb) = two(&mut ui, &mut a, &mut b, second_first);
        assert!(ra.blurred && !ra.focused, "first field told, second_first = {second_first}");
        assert!(rb.focused && !rb.blurred);
        assert_eq!(frames_of(&list).iter().filter(|c| **c == t.text).count(), 1);
    }
}

#[test]
fn an_outside_change_of_the_text_shows_with_the_caret_at_the_end() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_at("", 0.0);
    typ(&mut ui, "ab");
    plain(&mut ui, &mut s);
    s = "xyz".to_string();
    let (list, _) = plain(&mut ui, &mut s);
    assert!(texts(&list).iter().any(|x| x.1 == "xyz"));
    // 3 chars: 27 + 21.6 = 48.6 -> 49.
    rect_is(caret(&list, &t).unwrap(), 49.0, 25.2, 1.0, 15.6);
}

// ---- 17-19: numbers ----

/// A number field in a node at (20, 20); it is as wide as its text plus 14, 24 tall.
fn num(ui: &mut Ui, v: &mut f64, spec: NumberSpec) -> (DrawList, FieldResp) {
    let mut r = None;
    let list = frame(ui, |ui| {
        ui.node("cell", at(20.0, 20.0, Style::default()), |ui| r = Some(ui.number("n", v, spec)));
    });
    (list, r.unwrap())
}

const SPEC: NumberSpec = NumberSpec { min: 0.5, max: 3.0, step: 0.025 };

/// A number field focused by a click on its middle.
fn num_focused(v: &mut f64, spec: NumberSpec) -> Ui {
    let mut ui = Ui::new(Theme::dark());
    num(&mut ui, v, spec);
    click(&mut ui, Vec2::new(30.0, 32.0));
    let (_, r) = num(&mut ui, v, spec);
    assert!(r.focused);
    ui
}

#[test]
fn up_and_down_step_a_number_from_its_minimum() {
    let mut v = 1.0;
    let mut ui = num_focused(&mut v, SPEC);
    keys(&mut ui, Key::Up, NONE, 2);
    let (list, r) = num(&mut ui, &mut v, SPEC);
    // 1 is 20 steps above 0.5; two steps up is 0.5 + 22 x 0.025 = 1.05, rounded to the 3 digits of the step.
    near(v as f32, 1.05);
    assert!(r.changed && r.focused);
    assert!(texts(&list).iter().any(|x| x.1 == "1.05"));
    // At the minimum, Down stays.
    let mut v = 0.5;
    let mut ui = num_focused(&mut v, SPEC);
    key(&mut ui, Key::Down, NONE);
    let (_, r) = num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 0.5);
    assert!(!r.changed);
    // From off the grid, Up goes to the next step: 1.01 is 20.4 steps above, so 21 steps: 1.025.
    let mut v = 1.01;
    let mut ui = num_focused(&mut v, SPEC);
    key(&mut ui, Key::Up, NONE);
    num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 1.025);
    // At the maximum, Up stays.
    let mut v = 3.0;
    let mut ui = num_focused(&mut v, SPEC);
    key(&mut ui, Key::Up, NONE);
    num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 3.0);
}

#[test]
fn a_number_follows_typing_live_and_clamps_on_enter() {
    let mut v = 1.0;
    let mut ui = num_focused(&mut v, SPEC);
    key(&mut ui, Key::A, CTRL);
    typ(&mut ui, "9");
    let (_, r) = num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 9.0, "live and unclamped");
    assert!(r.changed);
    key(&mut ui, Key::Enter, NONE);
    let (list, r) = num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 3.0);
    assert!(r.submitted && r.blurred && !r.focused && r.changed);
    assert!(texts(&list).iter().any(|x| x.1 == "3"));
    // An unreadable buffer goes back to the value at focus, not to the last readable one.
    let mut v = 1.0;
    let mut ui = num_focused(&mut v, SPEC);
    key(&mut ui, Key::A, CTRL);
    typ(&mut ui, "5e");
    num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 5.0);
    key(&mut ui, Key::Enter, NONE);
    num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 1.0);
    // Escape too.
    let mut v = 1.0;
    let mut ui = num_focused(&mut v, SPEC);
    key(&mut ui, Key::A, CTRL);
    typ(&mut ui, "2");
    num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 2.0);
    key(&mut ui, Key::Escape, NONE);
    let (_, r) = num(&mut ui, &mut v, SPEC);
    assert_eq!(v, 1.0);
    assert!(r.cancelled && r.blurred);
}

#[test]
fn a_number_refuses_letters_but_e() {
    let mut v = 1.0;
    let mut ui = num_focused(&mut v, SPEC);
    key(&mut ui, Key::A, CTRL);
    typ(&mut ui, "a1b2e");
    let (list, _) = num(&mut ui, &mut v, SPEC);
    assert!(texts(&list).iter().any(|x| x.1 == "12e"));
    assert_eq!(v, 12.0, "the last readable value stays while \"12e\" does not parse");
}

// ---- 20-23: placeholder, focus option, blur, vanishing ----

#[test]
fn the_placeholder_shows_while_empty() {
    let t = Theme::dark();
    let opts = FieldOpts { placeholder: "random", ..FieldOpts::default() };
    let mut ui = Ui::new(t);
    let mut s = String::new();
    let (list, _) = one(&mut ui, &mut s, opts);
    let p = texts(&list).into_iter().find(|x| x.1 == "random").expect("the placeholder");
    assert_eq!(p.2, t.muted);
    near(p.0.x, 27.0);
    near(p.0.y, 25.2);
    let mut s = "x".to_string();
    let (list, _) = one(&mut ui, &mut s, opts);
    assert!(texts(&list).iter().all(|x| x.1 != "random"));
}

#[test]
fn the_focus_option_selects_all_once() {
    let t = Theme::dark();
    let opts = FieldOpts { focus: true, ..FieldOpts::default() };
    let mut ui = Ui::new(t);
    let mut s = "hello".to_string();
    let (list, r) = one(&mut ui, &mut s, opts);
    assert!(r.focused);
    rect_is(selection(&list, &t).unwrap(), 27.0, 25.2, 36.0, 15.6);
    key(&mut ui, Key::Right, NONE);
    let (list, _) = one(&mut ui, &mut s, opts);
    assert!(selection(&list, &t).is_none(), "not selected again");
    // The caret is at the end, column 5: 27 + 36 = 63.
    rect_is(caret(&list, &t).unwrap(), 63.0, 25.2, 1.0, 15.6);
}

#[test]
fn a_blurred_field_shows_the_start_of_its_text() {
    let (mut ui, mut s) = focused_at("", 0.0);
    typ(&mut ui, &"0123456789".repeat(3));
    let (list, _) = plain(&mut ui, &mut s);
    near(texts(&list).into_iter().find(|x| x.1 == s).unwrap().0.x, 11.0);
    key(&mut ui, Key::Enter, NONE);
    let (list, _) = plain(&mut ui, &mut s);
    near(texts(&list).into_iter().find(|x| x.1 == s).unwrap().0.x, 27.0);
}

#[test]
fn a_focused_field_that_is_not_built_any_more_drops_the_focus() {
    let (mut ui, mut s) = focused_at("abc", 1.0);
    assert!(ui.focused().is_some());
    key(&mut ui, Key::Left, NONE);
    frame(&mut ui, |_| {});
    assert_eq!(ui.focused(), None);
    let (_, r) = plain(&mut ui, &mut s);
    assert!(!r.focused);
}

#[test]
fn compact_look_is_22_tall_and_the_pad_is_the_look() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let mut s = String::new();
    let opts = FieldOpts { look: FieldLook::Compact, width: Dim::Px(130.0), ..FieldOpts::default() };
    let mut id: Option<Id> = None;
    frame(&mut ui, |ui| {
        ui.node("cell", at(20.0, 20.0, Style::default()), |ui| id = Some(ui.text_field("f", &mut s, opts).resp.id));
    });
    rect_is(ui.rect(id.unwrap()).unwrap(), 20.0, 20.0, 130.0, 22.0);
}
