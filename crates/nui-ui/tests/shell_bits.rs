//! What the world preview's shell needed of the kit, without a GPU: wrapped text, the filled box, the select,
//! the `small` and `label` looks, and the detail tab without its second line. Numbers come from the page's
//! CSS (preview.html lines 105, 109, 123, 127, 195-200, 580) and each is derived in its comment.

use nui_base::{Button, Cmd, Color, DrawList, Event, Rect, Vec2};
use nui_layout::{Dim, Style};
use nui_ui::tokens::{CONTROL, CONTROL_COMPACT};
use nui_ui::{Resp, TabOpts, TextLook, Theme, Tone, Ui, wrap_lines, wrap_lines_after};

#[test]
fn n1b_a_lead_is_taken_off_the_first_line_only() {
    let small = TextLook::small(Tone::Text);
    let step = nui_ui::ADVANCE_EM * small.size;
    // 10 cells wide, 4 taken by the lead: the long word fills the first line's 6 cells, then whole lines of 10.
    let lines = wrap_lines_after("abcdefghijklmnopqrstuvwxyz", small, 10.0 * step, 4.0 * step);
    assert_eq!(lines, ["abcdef", "ghijklmnop", "qrstuvwxyz"]);
    assert_eq!(wrap_lines_after("ab cd", small, 10.0 * step, 0.0), wrap_lines("ab cd", small, 10.0 * step));
}

const SIZE: Vec2 = Vec2::new(600.0, 400.0);
const HINT: &str = "Click \u{201c}pack\u{2026}\u{201d} beside a value to put it back to the pack\u{2019}s.";

fn body() -> TextLook {
    TextLook::body(Tone::Text)
}

fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 0.01, "{a} != {b}");
}

fn column(w: f32) -> Style {
    Style { column: true, w: Dim::Px(w), ..Style::default() }
}

/// One frame of a column `w` wide whose only content is `build`; returns the list and the column's id.
fn frame(ui: &mut Ui, w: f32, build: impl FnOnce(&mut Ui)) -> DrawList {
    ui.begin_frame(SIZE);
    ui.node("col", column(w), build);
    ui.end_frame().clone()
}

#[test]
fn n1_wrap_lines_of_the_reset_hint() {
    let small = TextLook::small(Tone::Muted);
    // 46 chars of 6.6 px = 303.6 fit in 309; the next word, " the", would make 50 chars = 330.
    assert_eq!(wrap_lines(HINT, small, 309.0), ["Click \u{201c}pack\u{2026}\u{201d} beside a value to put it back to", "the pack\u{2019}s."]);
    // 261 / 6.6 = 39.5: 38 chars fit, " back" would make 43.
    assert_eq!(wrap_lines(HINT, small, 261.0), ["Click \u{201c}pack\u{2026}\u{201d} beside a value to put it", "back to the pack\u{2019}s."]);
}

#[test]
fn n2_wrap_lines_edges() {
    // 30 / 7.2 = 4.17: four glyphs to a piece.
    assert_eq!(wrap_lines("abcdefghij", body(), 30.0), ["abcd", "efgh", "ij"]);
    assert_eq!(wrap_lines("  a   b ", body(), 100.0), ["a b"]);
    assert!(wrap_lines("", body(), 100.0).is_empty());
}

#[test]
fn n3_paragraph_height() {
    let small = TextLook::small(Tone::Muted);
    let mut ui = Ui::new(Theme::dark());
    for w in [309.0, 261.0] {
        let mut id = None;
        ui.begin_frame(SIZE);
        ui.node("col", column(w), |ui| id = Some(ui.node("p", Style::default(), |ui| ui.paragraph(HINT, small))));
        ui.end_frame();
        let r = ui.rect(id.unwrap()).unwrap();
        near(r.w, w);
        // Two lines of 16.5.
        near(r.h, 33.0);
    }
    let mut id = None;
    ui.begin_frame(SIZE);
    ui.node("col", column(309.0), |ui| id = Some(ui.node("p", Style::default(), |ui| ui.paragraph("", small))));
    ui.end_frame();
    near(ui.rect(id.unwrap()).unwrap().h, 0.0);
}

fn select_frame(ui: &mut Ui) -> Resp {
    let mut r = None;
    ui.begin_frame(SIZE);
    ui.node("col", column(320.0), |ui| r = Some(ui.select("area", "4,096 blocks")));
    ui.end_frame();
    r.unwrap()
}

#[test]
fn n4_select_size_and_click() {
    let mut ui = Ui::new(Theme::dark());
    let first = select_frame(&mut ui);
    let r = ui.rect(first.id).unwrap();
    // The display-only select is the compact control: 1 border + 3 padding + 18 line + 3 padding + 1 border.
    near(r.h, CONTROL_COMPACT);
    near(r.w, 320.0);
    let p = Vec2::new(r.x + 20.0, r.y + 10.0);
    ui.event(&Event::PointerMoved { pos: p });
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    select_frame(&mut ui);
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    let last = select_frame(&mut ui);
    assert!(last.clicked);
}

#[test]
fn n5_detail_tab_without_a_sub_line() {
    let mut ui = Ui::new(Theme::dark());
    let mut ids = Vec::new();
    ui.begin_frame(SIZE);
    ui.node("col", column(200.0), |ui| {
        ids.push(ui.detail_tab("a", "16 blocks", "", TabOpts::default()).id);
        ids.push(ui.detail_tab("b", "16 blocks", "1 min", TabOpts::default()).id);
    });
    ui.end_frame();
    // 18 line + 2 x 5 padding (frame included) + 0 = 28.
    near(ui.rect(ids[0]).unwrap().h, CONTROL);
    near(ui.rect(ids[1]).unwrap().h, CONTROL + 16.5);
}

#[test]
fn n6_fill_paints_its_rect() {
    let mut ui = Ui::new(Theme::dark());
    let c = Color::hex(0x336699);
    let list = frame(&mut ui, 300.0, |ui| ui.fill(Style { w: Dim::Px(40.0), h: Dim::Px(1.0), ..Style::default() }, c));
    let got: Vec<Rect> = list.cmds.iter().filter_map(|k| if let Cmd::Fill { rect, color } = k { (*color == c).then_some(*rect) } else { None }).collect();
    assert_eq!(got.len(), 1);
    let r = got[0];
    assert_eq!((r.x, r.y, r.w, r.h), (0.0, 0.0, 40.0, 1.0));
}

#[test]
fn n7_label_and_small_looks() {
    let l = TextLook::label();
    assert_eq!((l.size, l.line, l.tone, l.tracking), (11.0, 16.5, Tone::Muted, 0.88));
    let s = TextLook::small(Tone::Text);
    assert_eq!((s.size, s.line, s.tone, s.tracking), (11.0, 16.5, Tone::Text, 0.0));
}
