//! The launcher's icon buttons, hover box, disabled fields and toggle look, read from the draw list (mere-ui has no
//! renderer: the colours of the commands are the pixels' colours).

#[allow(unused_imports)]
use mere_ui::tokens::*;
use mere_base::{Button, Cmd, Color, DrawList, Event, Rect, Vec2};
use mere_layout::{Dim, Edges, Style};
use mere_ui::{ActionLook, ActionOpts, FieldOpts, Icon, IconLook, IconOpts, NumberOpts, NumberSpec, Theme, ToggleOpts, Ui, mix};

const SIZE: Vec2 = Vec2::new(400.0, 200.0);
const BOLT: &str = "M5 5h14v14H5z";

fn frame(ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> DrawList {
    ui.begin_frame(SIZE);
    build(ui);
    ui.end_frame().clone()
}

fn icon() -> Icon {
    Icon::parse(BOLT, 24.0).unwrap()
}

fn mv(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerMoved { pos: p });
}

/// (fill, frame, icon colour) of the first button drawn.
fn parts(l: &DrawList) -> (Option<Color>, Option<Color>, Color) {
    let fill = l.cmds.iter().find_map(|c| if let Cmd::Fill { color, .. } = c { Some(*color) } else { None });
    let frame = l.cmds.iter().find_map(|c| if let Cmd::Frame { color, .. } = c { Some(*color) } else { None });
    let shape = l.cmds.iter().find_map(|c| if let Cmd::Shape { color, .. } = c { Some(*color) } else { None }).expect("an icon");
    (fill, frame, shape)
}

fn button(ui: &mut Ui, look: IconLook, opts: IconOpts) -> (DrawList, mere_ui::Resp) {
    let mut r = None;
    let l = frame(ui, |ui| {
        ui.node("row", Style { w: Dim::Px(200.0), ..Style::default() }, |ui| r = Some(ui.icon_action("b", &icon(), look, opts)));
    });
    (l, r.unwrap())
}

/// Builds once to place the button, moves the pointer over it (or not), and builds again.
fn at_state(look: IconLook, opts: IconOpts, hover: bool) -> (Option<Color>, Option<Color>, Color) {
    let mut ui = Ui::new(Theme::dark());
    let (_, r) = button(&mut ui, look, opts);
    let rect = ui.rect(r.id).unwrap();
    mv(&mut ui, if hover { Vec2::new(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0) } else { Vec2::new(390.0, 190.0) });
    let (l, _) = button(&mut ui, look, opts);
    parts(&l)
}

#[test]
fn square_rest_hover_pressed_disabled() {
    let t = Theme::dark();
    let rest = IconOpts::default();
    assert_eq!(at_state(IconLook::Square, rest, false), (None, Some(t.edge), t.muted));
    assert_eq!(at_state(IconLook::Square, rest, true), (None, Some(t.text), t.text));
    let on = IconOpts { pressed: true, ..rest };
    assert_eq!(at_state(IconLook::Square, on, false), (Some(t.fill), Some(t.fill), t.on_fill));
    assert_eq!(at_state(IconLook::Square, on, true), (Some(t.fill), Some(t.fill), t.on_fill), "aria-expanded keeps its look on hover");
    let off = IconOpts { disabled: true, ..rest };
    assert_eq!(at_state(IconLook::Square, off, true), (None, Some(t.faded(t.edge, FADE_DISABLED)), t.faded(t.muted, FADE_DISABLED)));
}

#[test]
fn filled_rest_hover_disabled() {
    let t = Theme::dark();
    let rest = IconOpts::default();
    assert_eq!(at_state(IconLook::Filled, rest, false), (Some(t.fill), Some(t.fill), t.on_fill));
    let lit = mix(t.fill, FADE_LIT, t.ground);
    assert_eq!(at_state(IconLook::Filled, rest, true), (Some(lit), Some(lit), t.on_fill));
    let off = IconOpts { disabled: true, ..rest };
    assert_eq!(at_state(IconLook::Filled, off, true), (Some(t.faded(t.fill, FADE_DISABLED)), Some(t.faded(t.fill, FADE_DISABLED)), t.faded(t.on_fill, FADE_DISABLED)));
}

#[test]
fn bare_has_no_frame_and_is_24_by_32() {
    let t = Theme::dark();
    assert_eq!(at_state(IconLook::Bare, IconOpts::default(), false), (None, None, t.muted));
    assert_eq!(at_state(IconLook::Bare, IconOpts::default(), true), (None, None, t.text));
    let mut ui = Ui::new(Theme::dark());
    let (_, r) = button(&mut ui, IconLook::Bare, IconOpts::default());
    let rect = ui.rect(r.id).unwrap();
    assert_eq!((rect.w, rect.h, rect.x), (24.0, 32.0, -4.0));
}

#[test]
fn a_joined_square_overlaps_the_one_before_by_a_pixel() {
    let mut ui = Ui::new(Theme::dark());
    let mut ids = Vec::new();
    for _ in 0..2 {
        frame(&mut ui, |ui| {
            ids.clear();
            ui.node("row", Style { w: Dim::Px(200.0), ..Style::default() }, |ui| {
                ids.push(ui.icon_action("a", &icon(), IconLook::Square, IconOpts::default()).id);
                ids.push(ui.icon_action("b", &icon(), IconLook::Square, IconOpts { joined: true, ..IconOpts::default() }).id);
            });
        });
    }
    let (a, b) = (ui.rect(ids[0]).unwrap(), ui.rect(ids[1]).unwrap());
    assert_eq!((a.w, a.h), (ICON_SQUARE, ICON_SQUARE));
    assert_eq!(b.x, a.x + a.w - 1.0);
}

#[test]
fn hover_box_is_lit_only_while_the_pointer_is_over_it() {
    let t = Theme::dark();
    let lit = mix(t.text, FADE_TINT, t.ground);
    let build = |ui: &mut Ui| {
        frame(ui, |ui| {
            ui.hover_box("row", Style { w: Dim::Px(100.0), h: Dim::Px(40.0), ..Style::default() }, lit, |_| {});
        })
    };
    let mut ui = Ui::new(t);
    let has_lit = |l: &DrawList| l.cmds.iter().any(|c| matches!(c, Cmd::Fill { color, .. } if *color == lit));
    assert!(!has_lit(&build(&mut ui)));
    mv(&mut ui, Vec2::new(10.0, 10.0));
    assert!(has_lit(&build(&mut ui)));
    // A held press does not unlight it.
    ui.event(&Event::PointerDown { pos: Vec2::new(10.0, 10.0), button: Button::Left, clicks: 1 });
    assert!(has_lit(&build(&mut ui)));
    ui.event(&Event::PointerUp { pos: Vec2::new(10.0, 10.0), button: Button::Left });
    mv(&mut ui, Vec2::new(300.0, 100.0));
    assert!(!has_lit(&build(&mut ui)));
}

#[test]
fn a_disabled_field_takes_no_focus_and_paints_its_frame_at_half() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let mut s = String::from("abc");
    let build = |ui: &mut Ui, s: &mut String| {
        let mut r = None;
        let l = frame(ui, |ui| {
            ui.node("row", Style { w: Dim::Px(200.0), ..Style::default() }, |ui| {
                r = Some(ui.text_field("f", s, FieldOpts { disabled: true, ..FieldOpts::default() }));
            });
        });
        (l, r.unwrap())
    };
    let (_, r) = build(&mut ui, &mut s);
    let rect = ui.rect(r.resp.id).unwrap();
    let p = Vec2::new(rect.x + 5.0, rect.y + 5.0);
    mv(&mut ui, p);
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    let (l, r) = build(&mut ui, &mut s);
    assert!(!r.focused && ui.focused().is_none());
    let frame_colour = l.cmds.iter().find_map(|c| if let Cmd::Frame { color, .. } = c { Some(*color) } else { None }).unwrap();
    assert_eq!(frame_colour, t.faded(t.edge, FADE_DISABLED));
}

#[test]
fn the_52_px_field_is_one_line_at_the_top() {
    let mut ui = Ui::new(Theme::dark());
    let mut s = String::from("-XX:+UseZGC");
    let mut id = None;
    for _ in 0..2 {
        frame(&mut ui, |ui| {
            ui.node("row", Style { w: Dim::Px(200.0), ..Style::default() }, |ui| {
                id = Some(ui.text_field("j", &mut s, FieldOpts { height: Dim::Px(52.0), width: Dim::Pct(100.0), ..FieldOpts::default() }).resp.id);
            });
        });
    }
    let r = ui.rect(id.unwrap()).unwrap();
    assert_eq!((r.w, r.h), (200.0, 52.0));
}

#[test]
fn number_with_left_aligned_starts_at_the_left_padding() {
    let mut ui = Ui::new(Theme::dark());
    let mut v = 12.0;
    let mut l = DrawList::default();
    let mut left_x = 0.0;
    for right in [false, true] {
        l = frame(&mut ui, |ui| {
            ui.node("row", Style { w: Dim::Px(200.0), ..Style::default() }, |ui| {
                ui.number_with("n", &mut v, NumberSpec::ANY, NumberOpts { width: Dim::Pct(100.0), right, pad_v: 4.0, disabled: false });
            });
        });
        let x = l.cmds.iter().find_map(|c| if let Cmd::Text { at, .. } = c { Some(at.x) } else { None }).unwrap();
        if right {
            // "12" is two cells of 7.2, hugging the right padding.
            assert!((x - (200.0 - (FRAME + INSET) - 14.4)).abs() < 0.01, "{x}");
        } else {
            left_x = x;
        }
    }
    let _ = l;
    assert!((left_x - (FRAME + INSET)).abs() < 0.01, "{left_x}");
}

#[test]
fn toggle_with_disabled_does_not_flip() {
    let mut ui = Ui::new(Theme::dark());
    let mut on = true;
    let build = |ui: &mut Ui, on: &mut bool| {
        let mut r = None;
        frame(ui, |ui| {
            ui.node("row", Style { w: Dim::Px(200.0), ..Style::default() }, |ui| {
                r = Some(ui.toggle_with("t", on, "on", "off", ToggleOpts { min_w: Dim::Px(57.6), pad: Edges::vh(4.0, 7.0), disabled: true }));
            });
        });
        r.unwrap()
    };
    let r = build(&mut ui, &mut on);
    let rect = ui.rect(r.id).unwrap();
    assert_eq!((rect.w, rect.h), (64.4, 26.0), "7 + switch 28 + 8 + ON + 7 is past the 8ch minimum; 4 + 18 + 4 tall");
    let p = Vec2::new(rect.x + 5.0, rect.y + 5.0);
    mv(&mut ui, p);
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    build(&mut ui, &mut on);
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    let r = build(&mut ui, &mut on);
    assert!(!r.clicked && on);
}

#[test]
fn the_underlined_button_is_underlined_and_muted_on_hover() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    let mut id = None;
    let build = |ui: &mut Ui, id: &mut Option<_>| {
        frame(ui, |ui| {
            ui.node("row", Style { w: Dim::Px(200.0), ..Style::default() }, |ui| {
                *id = Some(ui.action_with("o", "Open", ActionLook::Underlined, ActionOpts::default()).id);
            });
        })
    };
    let l = build(&mut ui, &mut id);
    let r: Rect = ui.rect(id.unwrap()).unwrap();
    assert_eq!(r.h, 22.0, "2 + 18 + 2");
    assert!(l.cmds.iter().any(|c| matches!(c, Cmd::Text { color, .. } if *color == t.text)));
    mv(&mut ui, Vec2::new(r.x + 3.0, r.y + 3.0));
    let l = build(&mut ui, &mut id);
    assert!(l.cmds.iter().any(|c| matches!(c, Cmd::Text { color, .. } if *color == t.muted)));
}
