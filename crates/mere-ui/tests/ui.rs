//! The core without a GPU: pointer rules, colours by state, and the sizes of the page's widgets.

#[allow(unused_imports)]
use mere_ui::tokens::*;
use std::sync::Mutex;

use mere_base::{Button, Cmd, Color, DrawList, Event, Vec2, Weight, log};
use mere_layout::{Align, Dim, Display, Edges, Style, Track, Tracks};
use mere_base::Rect;
use mere_ui::{ButtonLook, ButtonOpts, Resp, SWITCH_W, SWITCH_H, TextLook, Theme, ToggleOpts, Tone, Ui};

const SIZE: Vec2 = Vec2::new(600.0, 300.0);

fn frame(ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> DrawList {
    ui.begin_frame(SIZE);
    build(ui);
    ui.end_frame().clone()
}

fn dark() -> Ui {
    Ui::new(Theme::dark())
}

fn row180() -> Style {
    Style { w: Dim::Px(180.0), ..Style::default() }
}

/// One Primary button in a 180 px row.
fn primary(ui: &mut Ui, opts: ButtonOpts) -> (Resp, DrawList) {
    let mut r = None;
    let list = frame(ui, |ui| {
        ui.node("row", row180(), |ui| r = Some(ui.button_with("go", "Generate preview", ButtonLook::Primary, opts)));
    });
    (r.unwrap(), list)
}

fn centre(ui: &Ui, r: &Resp) -> Vec2 {
    let r = ui.rect(r.id).expect("the widget was drawn once");
    Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn mv(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerMoved { pos: p });
}
fn down(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
}
fn up(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
}

fn px(c: Color) -> [u32; 3] {
    [c.r, c.g, c.b].map(|v| (v * 255.0).round() as u32)
}

fn fills(l: &DrawList) -> Vec<Color> {
    l.cmds.iter().filter_map(|c| if let Cmd::Fill { color, .. } = c { Some(*color) } else { None }).collect()
}
fn frames(l: &DrawList) -> Vec<Color> {
    l.cmds.iter().filter_map(|c| if let Cmd::Frame { color, .. } = c { Some(*color) } else { None }).collect()
}
fn texts(l: &DrawList) -> Vec<(Vec2, String, Color, Weight)> {
    l.cmds
        .iter()
        .filter_map(|c| if let Cmd::Text { at, text, color, weight, .. } = c { Some((*at, text.clone(), *color, *weight)) } else { None })
        .collect()
}

fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-3, "{a} != {b}");
}

#[test]
fn tracking_adds_after_every_char() {
    near(TextLook::heading().width("WORLD"), 39.6);
    let mut ui = dark();
    let list = frame(&mut ui, |ui| {
        ui.text("WORLD", TextLook::heading());
    });
    let t = texts(&list);
    assert_eq!(t.len(), 5);
    for (i, c) in t.iter().enumerate() {
        near(c.0.x, i as f32 * 7.92);
        assert_eq!(c.3, Weight::SemiBold);
    }
}

#[test]
fn theme_dark_and_light_are_the_page_variables() {
    let d = Theme::dark();
    assert_eq!(d.ground, Color::hex(0x000000));
    assert_eq!(d.line, Color::hex(0x303030));
    assert_eq!(d.text, Color::hex(0xebebeb));
    assert_eq!(d.muted, Color::hex(0x959595));
    assert_eq!(d.warn, Color::hex(0xe2a356));
    assert_eq!(d.grid, Color::hex(0xffffff).with_alpha(0.16));
    assert_eq!(d.track, Color::hex(0x606060));
    assert_eq!(d.edge, Color::hex(0x484848));
    assert_eq!((d.body, d.small), (12.0, 11.0));
    let l = Theme::light();
    assert_eq!(l.ground, Color::hex(0xf5f5f5));
    assert_eq!(l.line, Color::hex(0xcecece));
    assert_eq!(l.text, Color::hex(0x0f0f0f));
    assert_eq!(l.muted, Color::hex(0x606060));
    assert_eq!(l.warn, Color::hex(0x865401));
    assert_eq!(l.grid, Color::hex(0x000000).with_alpha(0.12));
    assert_eq!(l.track, Color::hex(0x959595));
    assert_eq!(l.edge, Color::hex(0xb1b1b1));
    assert_eq!((l.body, l.small), (12.0, 11.0));
}

#[test]
fn button_clicks_on_release_inside() {
    let mut ui = dark();
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    assert!(!r.clicked);
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    down(&mut ui, c);
    up(&mut ui, c);
    assert!(primary(&mut ui, ButtonOpts::default()).0.clicked);
    assert!(!primary(&mut ui, ButtonOpts::default()).0.clicked);
}

#[test]
fn press_inside_release_outside_is_no_click() {
    let mut ui = dark();
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    down(&mut ui, c);
    assert!(primary(&mut ui, ButtonOpts::default()).0.held);
    let out = Vec2::new(500.0, 250.0);
    mv(&mut ui, out);
    up(&mut ui, out);
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    assert!(!r.clicked && !r.held);
}

#[test]
fn press_outside_then_enter_while_down_neither_hovers_nor_clicks() {
    let mut ui = dark();
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    let c = centre(&ui, &r);
    let out = Vec2::new(500.0, 250.0);
    mv(&mut ui, out);
    down(&mut ui, out);
    primary(&mut ui, ButtonOpts::default());
    mv(&mut ui, c);
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    assert!(!r.hovered && !r.held && !r.clicked);
    up(&mut ui, c);
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    assert!(!r.clicked);
}

#[test]
fn hover_follows_the_pointer() {
    let mut ui = dark();
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    assert!(primary(&mut ui, ButtonOpts::default()).0.hovered);
    mv(&mut ui, Vec2::new(500.0, 250.0));
    assert!(!primary(&mut ui, ButtonOpts::default()).0.hovered);
}

#[test]
fn disabled_ignores_the_pointer() {
    let t = Theme::dark();
    let off = ButtonOpts { disabled: true, grow: 0.0 };
    let mut ui = dark();
    let (r, _) = primary(&mut ui, off);
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    down(&mut ui, c);
    up(&mut ui, c);
    let (r, list) = primary(&mut ui, off);
    assert!(!r.hovered && !r.held && !r.clicked);
    assert!(fills(&list).is_empty(), "a disabled primary button has no fill");
    assert_eq!(frames(&list), vec![t.edge]);
    assert_eq!(texts(&list)[0].2, t.muted);
}

#[test]
fn toggle_flips_on_click_and_shows_the_new_word() {
    let mut ui = dark();
    let mut on = false;
    let build = |ui: &mut Ui, on: &mut bool| {
        let mut r = None;
        let list = frame(ui, |ui| {
            ui.node("row", row180(), |ui| r = Some(ui.toggle("islands", on, "on", "off")));
        });
        (r.unwrap(), list)
    };
    let (r, list) = build(&mut ui, &mut on);
    assert_eq!(texts(&list)[0].1, "OFF");
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    down(&mut ui, c);
    up(&mut ui, c);
    let (r, list) = build(&mut ui, &mut on);
    assert!(r.clicked && on);
    assert_eq!(texts(&list)[0].1, "ON");
}

#[test]
fn first_frame_has_no_interaction() {
    for p in [Vec2::new(5.0, 5.0), Vec2::new(90.0, 17.0)] {
        let mut ui = dark();
        mv(&mut ui, p);
        down(&mut ui, p);
        up(&mut ui, p);
        let (r, _) = primary(&mut ui, ButtonOpts::default());
        assert!(r.rect.is_none() && !r.hovered && !r.held && !r.clicked);
    }
}
#[test]
fn active_is_dropped_when_its_widget_goes() {
    let mut ui = dark();
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    down(&mut ui, c);
    assert!(primary(&mut ui, ButtonOpts::default()).0.held);
    frame(&mut ui, |_| {});
    up(&mut ui, c);
    let (r, _) = primary(&mut ui, ButtonOpts::default());
    assert!(!r.held && !r.clicked);
}

fn setting_row(ui: &mut Ui, on: &mut bool) -> (mere_ui::Id, mere_ui::Id) {
    let style = Style {
        display: Display::Grid,
        cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Px(57.6)]),
        gap: Vec2::new(10.0, 0.0),
        align_items: Align::Center,
        pad: Edges::vh(2.0, 0.0),
        w: Dim::Px(180.0),
        ..Style::default()
    };
    let mut toggle = None;
    let row = ui.node("setting", style, |ui| {
        ui.label("Islands", Tone::Text);
        ui.text("", TextLook::body(Tone::Muted));
        toggle = Some(ui.toggle("islands", on, "on", "off").id);
    });
    (row, toggle.unwrap())
}

#[test]
fn sizes_match_the_page() {
    let mut ui = dark();
    let mut ids = None;
    let list = frame(&mut ui, |ui| {
        let (mut a, mut b, mut c) = (None, None, None);
        ui.node("row", row180(), |ui| a = Some(ui.button_with("go", "Generate preview", ButtonLook::Primary, ButtonOpts { grow: 1.0, ..Default::default() }).id));
        ui.node("plain-row", Style { align_items: Align::Start, ..Style::default() }, |ui| {
            b = Some(ui.button("detail", "16 blocks", ButtonLook::Plain).id);
            c = Some(ui.button("export", "Export", ButtonLook::Link).id);
        });
        ids = Some((a.unwrap(), b.unwrap(), c.unwrap()));
    });
    let (go, plain, link) = ids.unwrap();
    let g = ui.rect(go).unwrap();
    near(g.w, 180.0);
    near(g.h, CONTROL_LARGE);
    let t = texts(&list);
    near(t[0].0.x, g.x + 13.0);
    near(t[0].0.y, g.y + 10.2);
    assert_eq!(t[0].3, Weight::SemiBold);
    let p = ui.rect(plain).unwrap();
    near(p.w, 80.8);
    near(p.h, 22.0);
    near(t[1].0.y, p.y + 3.2);
    let l = ui.rect(link).unwrap();
    near(l.w, 43.2);
    near(l.h, 22.0);
    let underline = list.cmds.iter().rev().find_map(|c| if let Cmd::Fill { rect, .. } = c { Some(*rect) } else { None }).unwrap();
    near(underline.x, l.x);
    near(underline.y, l.y + 18.5);
    near(underline.w, 43.2);
    near(underline.h, 1.0);

    // A toggle in a `.setting` grid row.
    let mut ui = dark();
    let mut on = false;
    let mut ids = None;
    let list = frame(&mut ui, |ui| ids = Some(setting_row(ui, &mut on)));
    let (row, toggle) = ids.unwrap();
    let (row, tg) = (ui.rect(row).unwrap(), ui.rect(toggle).unwrap());
    near(tg.x - row.x, 122.4);
    // The switch, 8 px, then the word, in the cell's own width and not stretched to the cell.
    near(tg.w, SWITCH_W + 8.0 + 3.0 * 7.2);
    near(tg.h, 24.0);
    let off = texts(&list).into_iter().find(|t| t.1 == "OFF").unwrap();
    near(off.0.x, tg.x + SWITCH_W + 8.0);
}

#[test]
fn colours_follow_state() {
    let t = Theme::dark();
    // Primary.
    let mut ui = dark();
    let (r, normal) = primary(&mut ui, ButtonOpts::default());
    assert_eq!(fills(&normal)[0], t.fill);
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    let (_, hover) = primary(&mut ui, ButtonOpts::default());
    assert_eq!(fills(&hover)[0], t.faded(t.fill, FADE_LIT));
    down(&mut ui, c);
    let (r, held) = primary(&mut ui, ButtonOpts::default());
    assert!(r.held);
    assert_eq!(fills(&held)[0], fills(&hover)[0]);
    // Plain.
    let plain = |ui: &mut Ui| {
        let mut r = None;
        let l = frame(ui, |ui| {
            ui.node("row", Style { align_items: Align::Start, ..Style::default() }, |ui| r = Some(ui.button("d", "16 blocks", ButtonLook::Plain)));
        });
        (r.unwrap(), l)
    };
    let mut ui = dark();
    let (r, l) = plain(&mut ui);
    assert_eq!(texts(&l)[0].2, t.muted);
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    assert_eq!(texts(&plain(&mut ui).1)[0].2, t.text);
    // Toggle.
    let tog = |ui: &mut Ui| {
        let mut on = false;
        let mut r = None;
        let l = frame(ui, |ui| {
            ui.node("row", row180(), |ui| r = Some(ui.toggle("t", &mut on, "on", "off")));
        });
        (r.unwrap(), l)
    };
    let mut ui = dark();
    let (r, l) = tog(&mut ui);
    assert_eq!(frames(&l)[0], t.edge);
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    let hover = tog(&mut ui).1;
    assert_eq!(frames(&hover), vec![t.text, t.text], "frame and knob outline");
    assert_eq!(frames(&l), vec![t.edge, t.muted], "at rest the frame is the edge and the knob muted");
    assert_eq!(px(t.text), [0xeb, 0xeb, 0xeb]);
}

static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[test]
fn a_key_used_twice_is_reported() {
    log::add_sink(|rec| {
        let m = rec.message.to_string();
        if m.contains("used twice") {
            SEEN.lock().unwrap().push(m);
        }
    });
    let mut ui = dark();
    frame(&mut ui, |ui| {
        ui.node("n", Style::default(), |ui| {
            ui.button("dup-probe", "A", ButtonLook::Plain);
            ui.button("dup-probe", "B", ButtonLook::Plain);
        });
    });
    assert!(SEEN.lock().unwrap().iter().any(|m| m.contains("dup-probe")));
}



/// One toggle in a 180 px row, with its words as the catalog gives them (lower case: the widget capitalises).
fn switch(ui: &mut Ui, on: &mut bool, opts: ToggleOpts) -> (Resp, DrawList) {
    let mut r = None;
    let list = frame(ui, |ui| {
        ui.node("row", row180(), |ui| r = Some(ui.toggle_with("t", on, "on", "off", opts)));
    });
    (r.unwrap(), list)
}

fn rects_of(l: &DrawList) -> Vec<(Rect, Color, bool)> {
    l.cmds
        .iter()
        .filter_map(|c| match c {
            Cmd::Fill { rect, color } => Some((*rect, *color, true)),
            Cmd::Frame { rect, color, .. } => Some((*rect, *color, false)),
            _ => None,
        })
        .collect()
}

#[test]
fn an_off_switch_is_an_outline_with_an_outlined_knob_on_the_left() {
    let t = Theme::dark();
    let (mut ui, mut on) = (dark(), false);
    let (r, l) = switch(&mut ui, &mut on, ToggleOpts::default());
    let at = ui.rect(r.id).unwrap();
    let parts = rects_of(&l);
    assert_eq!(parts.len(), 2, "a frame and a knob, nothing filled: {parts:?}");
    let ((frame, fc, f_fill), (knob, kc, k_fill)) = (parts[0], parts[1]);
    assert!(!f_fill && !k_fill, "outlines only");
    assert_eq!((frame.w, frame.h), (SWITCH_W, SWITCH_H));
    assert_eq!((fc, kc), (t.edge, t.muted));
    assert!(knob.w == knob.h && knob.w < frame.h, "a square knob inside the frame");
    assert!(knob.x < frame.x + frame.w / 2.0, "the knob is on the left: {knob:?}");
    assert!(knob.x > frame.x && knob.y > frame.y && knob.y + knob.h < frame.y + frame.h);
    near(frame.x, at.x);
    assert_eq!(texts(&l)[0].1, "OFF");
    assert_eq!(texts(&l)[0].2, t.muted);
}

#[test]
fn an_on_switch_is_filled_with_the_knob_on_the_right() {
    let t = Theme::dark();
    let (mut ui, mut on) = (dark(), true);
    let (_, l) = switch(&mut ui, &mut on, ToggleOpts::default());
    let parts = rects_of(&l);
    assert_eq!(parts.len(), 2);
    let ((frame, fc, f_fill), (knob, kc, k_fill)) = (parts[0], parts[1]);
    assert!(f_fill && k_fill, "both filled");
    assert_eq!((fc, kc), (t.fill, t.on_fill));
    assert!(knob.x > frame.x + frame.w / 2.0, "the knob is on the right: {knob:?}");
    // The knob keeps the gap to the frame that the off knob keeps on the other side.
    let off_gap = {
        let (mut ui, mut off) = (dark(), false);
        let (_, l) = switch(&mut ui, &mut off, ToggleOpts::default());
        let p = rects_of(&l);
        p[1].0.x - p[0].0.x
    };
    near((frame.x + frame.w) - (knob.x + knob.w), off_gap);    assert_eq!(texts(&l)[0].1, "ON");
    assert_eq!(texts(&l)[0].2, t.text);
}

#[test]
fn a_click_on_the_word_flips_the_switch_and_a_click_beside_the_box_does_not() {
    let (mut ui, mut on) = (dark(), false);
    let (r, _) = switch(&mut ui, &mut on, ToggleOpts::default());
    let at = ui.rect(r.id).unwrap();
    // The word is the right end of the box.
    let word = Vec2::new(at.x + at.w - 3.0, at.y + at.h / 2.0);
    mv(&mut ui, word);
    down(&mut ui, word);
    switch(&mut ui, &mut on, ToggleOpts::default());
    up(&mut ui, word);
    let (r, _) = switch(&mut ui, &mut on, ToggleOpts::default());
    assert!(r.clicked && on, "the word reaches the switch");
    // In the row's free space, to the right of the box.
    let beside = Vec2::new(at.x + at.w + 20.0, at.y + at.h / 2.0);
    mv(&mut ui, beside);
    down(&mut ui, beside);
    switch(&mut ui, &mut on, ToggleOpts::default());
    up(&mut ui, beside);
    let (r, _) = switch(&mut ui, &mut on, ToggleOpts::default());
    assert!(!r.clicked && on);
}

#[test]
fn a_hovered_on_switch_is_lit_and_a_disabled_one_is_faded() {
    let t = Theme::dark();
    let (mut ui, mut on) = (dark(), true);
    let (r, _) = switch(&mut ui, &mut on, ToggleOpts::default());
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    let (_, l) = switch(&mut ui, &mut on, ToggleOpts::default());
    assert_eq!(rects_of(&l)[0].1, t.faded(t.fill, FADE_LIT), "the primary button's hover");
    let (mut ui, mut off) = (dark(), false);
    let (r, _) = switch(&mut ui, &mut off, ToggleOpts { disabled: true, ..ToggleOpts::default() });
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    let (_, l) = switch(&mut ui, &mut off, ToggleOpts { disabled: true, ..ToggleOpts::default() });
    let parts = rects_of(&l);
    assert_eq!(parts[0].1, t.faded(t.edge, FADE_DISABLED), "no hover when disabled, and faded");
    assert_eq!(texts(&l)[0].2, t.faded(t.muted, FADE_DISABLED));
}