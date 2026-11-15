//! The core without a GPU: pointer rules, colours by state, and the sizes of the page's widgets.

#[allow(unused_imports)]
use nui_ui::tokens::*;
use std::sync::Mutex;

use nui_base::{Button, Cmd, Color, DrawList, Event, Vec2, Weight, log};
use nui_layout::{Align, Dim, Display, Edges, Style, Track, Tracks};
use nui_ui::{ButtonLook, ButtonOpts, Resp, TextLook, Theme, Tone, Ui};

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
    assert_eq!(texts(&list)[0].1, "off");
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    down(&mut ui, c);
    up(&mut ui, c);
    let (r, list) = build(&mut ui, &mut on);
    assert!(r.clicked && on);
    assert_eq!(texts(&list)[0].1, "on");
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

fn setting_row(ui: &mut Ui, on: &mut bool) -> (nui_ui::Id, nui_ui::Id) {
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
    near(tg.w, 57.6);
    near(tg.h, 24.0);
    let off = texts(&list).into_iter().find(|t| t.1 == "off").unwrap();
    near(off.0.x, row.x + 149.4);
}

#[test]
fn colours_follow_state() {
    let t = Theme::dark();
    // Primary.
    let mut ui = dark();
    let (r, normal) = primary(&mut ui, ButtonOpts::default());
    assert_eq!(fills(&normal)[0], t.text);
    let c = centre(&ui, &r);
    mv(&mut ui, c);
    let (_, hover) = primary(&mut ui, ButtonOpts::default());
    assert_eq!(fills(&hover)[0], t.faded(t.text, FADE_LIT));
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
    assert_eq!(frames(&tog(&mut ui).1)[0], t.text);
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


