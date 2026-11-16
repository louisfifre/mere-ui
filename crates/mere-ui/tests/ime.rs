//! The input method in the text field: the composition shown inline and underlined, the value untouched until
//! the commit, cancel, the number field refusing it, and the spot the backend reads. No system: the messages
//! of the input method are `Event::Ime` values.
//!
//! The field sits at (20, 20), 214 wide, as in `field.rs`: content box x 29, y 25; the glyph box (15.6 tall)
//! starts at y 26.2; a cell is 7.2 wide; the underline row is the content top + 15 = 40.

use mere_base::{Button, Cmd, Color, DrawList, Event, Ime, Key, Mods, Rect, Vec2};
use mere_layout::{Dim, Insets, Position, Style};
use mere_ui::{FieldOpts, FieldResp, NumberSpec, Theme, Ui};

const SIZE: Vec2 = Vec2::new(860.0, 440.0);
const NONE: Mods = Mods { ctrl: false, shift: false };
const CTRL: Mods = Mods { ctrl: true, shift: false };
const SHIFT: Mods = Mods { ctrl: false, shift: true };
const X0: f32 = 29.0;
const INNER_Y: f32 = 25.0;
const MID_Y: f32 = 37.0;

fn frame(ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> DrawList {
    ui.begin_frame(SIZE);
    build(ui);
    ui.end_frame().clone()
}

fn at(x: f32, y: f32) -> Style {
    Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(x), top: Dim::Px(y), right: Dim::Auto, bottom: Dim::Auto },
        ..Style::default()
    }
}

fn one(ui: &mut Ui, s: &mut String, max_chars: usize) -> (DrawList, FieldResp) {
    let mut r = None;
    let list = frame(ui, |ui| {
        ui.node("cell", at(20.0, 20.0), |ui| {
            r = Some(ui.text_field("f", s, FieldOpts { width: Dim::Px(214.0), max_chars, ..FieldOpts::default() }));
        });
    });
    (list, r.unwrap())
}

fn plain(ui: &mut Ui, s: &mut String) -> (DrawList, FieldResp) {
    one(ui, s, 0)
}

fn click(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerMoved { pos: p });
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
}

fn ime(ui: &mut Ui, i: Ime) {
    ui.event(&Event::Ime(i));
}
fn update(ui: &mut Ui, text: &str, cursor: usize) {
    ime(ui, Ime::Update { text: text.to_string(), cursor });
}
fn commit(ui: &mut Ui, text: &str) {
    ime(ui, Ime::Commit { text: text.to_string() });
}

/// A field holding `text`, clicked far right so the caret is at the end.
fn focused_end(text: &str, max_chars: usize) -> (Ui, String) {
    let mut ui = Ui::new(Theme::dark());
    let mut s = text.to_string();
    one(&mut ui, &mut s, max_chars);
    click(&mut ui, Vec2::new(X0 + 200.0, MID_Y));
    let (_, r) = one(&mut ui, &mut s, max_chars);
    assert!(r.focused);
    (ui, s)
}

fn fills(l: &DrawList) -> Vec<(Rect, Color)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Fill { rect, color } = c { Some((*rect, *color)) } else { None }).collect()
}
fn texts(l: &DrawList) -> Vec<(Vec2, String)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Text { at, text, .. } = c { Some((*at, text.clone())) } else { None }).collect()
}
/// The 1 px tall fill, if any: the underline.
fn underline(l: &DrawList) -> Option<Rect> {
    fills(l).into_iter().find(|f| f.0.h == 1.0 && f.0.w > 1.0).map(|f| f.0)
}
fn caret(l: &DrawList, t: &Theme) -> Option<Rect> {
    fills(l).into_iter().find(|f| f.1 == t.text && f.0.w == 1.0 && f.0.h > 10.0).map(|f| f.0)
}
fn selection(l: &DrawList, t: &Theme) -> Option<Rect> {
    fills(l).into_iter().find(|f| f.1 == t.select).map(|f| f.0)
}
fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-3, "{a} != {b}");
}

#[test]
fn start_and_update_show_the_preedit_without_touching_the_value() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_end("ab", 0);
    ime(&mut ui, Ime::Start);
    update(&mut ui, "にほ", 2);
    let (l, r) = plain(&mut ui, &mut s);
    assert_eq!(s, "ab");
    assert!(!r.changed);
    assert!(r.composing);
    // Shown: "ab" at cell 0, に at cell 2, ほ at cell 4, each centred by 1.2.
    let tx = texts(&l);
    assert_eq!(tx.iter().map(|x| x.1.as_str()).collect::<Vec<_>>(), ["ab", "に", "ほ"]);
    near(tx[1].0.x, X0 + 14.4 + 1.2);
    near(tx[2].0.x, X0 + 28.8 + 1.2);
    let u = underline(&l).expect("underline");
    near(u.x, X0 + 14.4);
    near(u.w, 28.8);
    near(u.y, INNER_Y + 15.0);
    // The caret is at the end of the composition: cell 6.
    near(caret(&l, &t).unwrap().x, (X0 + 43.2).round());
}

#[test]
fn update_cursor_inside() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_end("ab", 0);
    ime(&mut ui, Ime::Start);
    update(&mut ui, "日本語", 1);
    let (l, _) = plain(&mut ui, &mut s);
    // Cell 2 where the composition starts, plus the 2 cells of 日.
    near(caret(&l, &t).unwrap().x, (X0 + 14.4 + 14.4).round());
}

#[test]
fn commit_inserts_once_and_undoes_as_one_step() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_end("ab", 0);
    ime(&mut ui, Ime::Start);
    update(&mut ui, "日本語", 3);
    commit(&mut ui, "日本語");
    ime(&mut ui, Ime::End);
    let (l, r) = plain(&mut ui, &mut s);
    assert_eq!(s, "ab日本語");
    assert!(r.changed);
    assert!(!r.composing);
    assert!(underline(&l).is_none());
    near(caret(&l, &t).unwrap().x, (X0 + 57.6).round());
    ui.event(&Event::Key { key: Key::Z, mods: CTRL });
    let (_, r) = plain(&mut ui, &mut s);
    assert_eq!(s, "ab");
    assert!(r.changed);
}

#[test]
fn cancel_leaves_text_and_selection() {
    let t = Theme::dark();
    let (mut ui, mut s) = focused_end("abcd", 0);
    // Home, Right x1, Shift+Right x2: "bc" selected.
    ui.event(&Event::Key { key: Key::Home, mods: NONE });
    ui.event(&Event::Key { key: Key::Right, mods: NONE });
    ui.event(&Event::Key { key: Key::Right, mods: SHIFT });
    ui.event(&Event::Key { key: Key::Right, mods: SHIFT });
    let (l, _) = plain(&mut ui, &mut s);
    assert!(selection(&l, &t).is_some());
    ime(&mut ui, Ime::Start);
    update(&mut ui, "か", 1);
    let (l, r) = plain(&mut ui, &mut s);
    assert!(!r.changed);
    assert!(selection(&l, &t).is_none(), "no selection paint while composing");
    assert_eq!(texts(&l).iter().map(|x| x.1.as_str()).collect::<Vec<_>>(), ["a", "か", "d"]);
    update(&mut ui, "", 0);
    ime(&mut ui, Ime::End);
    let (l, r) = plain(&mut ui, &mut s);
    assert!(!r.changed);
    assert!(!r.composing);
    assert_eq!(s, "abcd");
    let sel = selection(&l, &t).expect("the selection is back");
    near(sel.x, X0 + 7.2);
    near(sel.w, 14.4);
}

#[test]
fn commit_replaces_the_selection() {
    let (mut ui, mut s) = focused_end("abcd", 0);
    ui.event(&Event::Key { key: Key::Home, mods: NONE });
    ui.event(&Event::Key { key: Key::Right, mods: NONE });
    ui.event(&Event::Key { key: Key::Right, mods: SHIFT });
    ui.event(&Event::Key { key: Key::Right, mods: SHIFT });
    plain(&mut ui, &mut s);
    commit(&mut ui, "か");
    plain(&mut ui, &mut s);
    assert_eq!(s, "aかd");
}

#[test]
fn max_chars_cuts_the_commit() {
    let (mut ui, mut s) = focused_end("abc", 4);
    commit(&mut ui, "日本");
    one(&mut ui, &mut s, 4);
    assert_eq!(s, "abc日");
}

#[test]
fn number_field_ignores_the_input_method() {
    let spec = NumberSpec { min: 0.5, max: 3.0, step: 0.025 };
    let mut v = 1.0f64;
    let mut ui = Ui::new(Theme::dark());
    let num = |ui: &mut Ui, v: &mut f64| {
        let mut r = None;
        let l = frame(ui, |ui| {
            ui.node("cell", at(20.0, 20.0), |ui| r = Some(ui.number("n", v, spec)));
        });
        (l, r.unwrap())
    };
    num(&mut ui, &mut v);
    click(&mut ui, Vec2::new(30.0, 32.0));
    let (_, r) = num(&mut ui, &mut v);
    assert!(r.focused);
    ime(&mut ui, Ime::Start);
    update(&mut ui, "に", 1);
    commit(&mut ui, "に");
    let (l, r) = num(&mut ui, &mut v);
    assert_eq!(v, 1.0);
    assert!(!r.changed && !r.composing);
    assert_eq!(ui.ime_spot(), None);
    assert!(underline(&l).is_none());
}

#[test]
fn spot_follows_the_caret_and_the_composition() {
    let mut ui = Ui::new(Theme::dark());
    let mut s = "ab".to_string();
    plain(&mut ui, &mut s);
    assert_eq!(ui.ime_spot(), None);
    click(&mut ui, Vec2::new(X0 + 200.0, MID_Y));
    plain(&mut ui, &mut s);
    let r = ui.ime_spot().expect("focused");
    near(r.x, X0 + 14.4);
    near(r.y, INNER_Y + 1.2);
    near(r.w, 1.0);
    near(r.h, 15.6);
    // While composing the spot stays at the composition's start, wherever its cursor goes.
    ime(&mut ui, Ime::Start);
    update(&mut ui, "日本語", 3);
    plain(&mut ui, &mut s);
    near(ui.ime_spot().unwrap().x, X0 + 14.4);
    update(&mut ui, "日本語", 1);
    plain(&mut ui, &mut s);
    near(ui.ime_spot().unwrap().x, X0 + 14.4);
    // A press elsewhere drops the field's state with the composition.
    click(&mut ui, Vec2::new(500.0, 300.0));
    plain(&mut ui, &mut s);
    assert_eq!(ui.ime_spot(), None);
    click(&mut ui, Vec2::new(X0 + 200.0, MID_Y));
    let (l, r) = plain(&mut ui, &mut s);
    assert!(r.focused && !r.composing);
    assert!(underline(&l).is_none());
    assert_eq!(s, "ab");
}

#[test]
fn keys_are_dropped_while_composing() {
    let (mut ui, mut s) = focused_end("ab", 0);
    ime(&mut ui, Ime::Start);
    update(&mut ui, "に", 1);
    plain(&mut ui, &mut s);
    ui.event(&Event::Key { key: Key::Backspace, mods: NONE });
    ui.event(&Event::Char { ch: 'x' });
    let (_, r) = plain(&mut ui, &mut s);
    assert_eq!(s, "ab");
    assert!(!r.changed);
    assert!(r.composing);
    assert!(r.focused);
}
