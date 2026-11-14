//! Fold, strip, choice, details and number field: sizes from the page's CSS, colours by state, paint order.

use nui_base::{Button, Cmd, Color, DrawList, Event, Rect, Vec2, Weight};
use nui_layout::{Align, Dim, Display, Edges, Insets, Position, Style, Track, Tracks};
use nui_ui::{FoldOpts, Id, NumberSpec, TabLook, TabOpts, TextLook, Theme, Tone, Ui, choice_style, details_style, strip_style};

const SIZE: Vec2 = Vec2::new(860.0, 440.0);

fn frame(ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> DrawList {
    ui.begin_frame(SIZE);
    build(ui);
    ui.end_frame().clone()
}

fn dark() -> Ui {
    Ui::new(Theme::dark())
}

/// Places `style`'s box at a point, like the demo's cells.
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
fn down(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
}
fn up(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
}

fn centre(ui: &Ui, id: Id) -> Vec2 {
    let r = ui.rect(id).expect("drawn once");
    Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn fills(l: &DrawList) -> Vec<(Rect, Color)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Fill { rect, color } = c { Some((*rect, *color)) } else { None }).collect()
}
fn texts(l: &DrawList) -> Vec<(Vec2, String, Color, Weight)> {
    l.cmds
        .iter()
        .filter_map(|c| if let Cmd::Text { at, text, color, weight, .. } = c { Some((*at, text.clone(), *color, *weight)) } else { None })
        .collect()
}
fn word(l: &DrawList, s: &str) -> (Vec2, String, Color, Weight) {
    texts(l).into_iter().find(|t| t.1 == s).unwrap_or_else(|| panic!("no text {s:?}"))
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

fn col320(x: f32, y: f32) -> Style {
    at(x, y, Style { column: true, w: Dim::Px(320.0), ..Style::default() })
}

// ---- fold ----

fn world_fold(ui: &mut Ui, open: &mut bool, ran: &mut bool) -> Id {
    let mut head = None;
    ui.node("cell", col320(20.0, 234.0), |ui| {
        head = Some(ui.fold("world", "World", open, |ui| {
            *ran = true;
            ui.label("Seed", Tone::Text);
        }).id);
    });
    head.unwrap()
}

#[test]
fn fold_closed_is_a_rule_a_title_and_a_sign() {
    let mut ui = dark();
    let t = Theme::dark();
    let (mut open, mut ran) = (false, false);
    let mut head = None;
    let list = frame(&mut ui, |ui| head = Some(world_fold(ui, &mut open, &mut ran)));
    rect_is(ui.rect(head.unwrap()).unwrap(), 20.0, 235.0, 320.0, 33.0);
    let f = fills(&list);
    assert_eq!(f.len(), 1);
    rect_is(f[0].0, 20.0, 234.0, 320.0, 1.0);
    assert_eq!(f[0].1, t.edge);
    let tx = texts(&list);
    assert_eq!(tx.len(), 6);
    for (i, c) in "WORLD".chars().enumerate() {
        let g = &tx[i];
        assert_eq!(g.1, c.to_string());
        near(g.0.x, 20.0 + i as f32 * 7.92);
        near(g.0.y, 244.0 - 0.3);
        assert_eq!(g.3, Weight::SemiBold);
        assert_eq!(g.2, t.text);
    }
    let sign = &tx[5];
    assert_eq!(sign.1, "+");
    near(sign.0.x, 332.08);
    near(sign.0.y, 243.7);
    assert_eq!(sign.3, Weight::Regular);
    assert_eq!(sign.2, t.muted);
    assert!(!ran, "a closed fold does not build its body");
    assert!(!ui.dump().contains("body"));
}

#[test]
fn fold_click_opens_in_the_same_frame() {
    let mut ui = dark();
    let (mut open, mut ran) = (false, false);
    let mut head = None;
    frame(&mut ui, |ui| head = Some(world_fold(ui, &mut open, &mut ran)));
    let c = centre(&ui, head.unwrap());
    mv(&mut ui, c);
    down(&mut ui, c);
    up(&mut ui, c);
    let list = frame(&mut ui, |ui| head = Some(world_fold(ui, &mut open, &mut ran)));
    assert!(open && ran);
    word(&list, "Seed");
    word(&list, "\u{2212}");
    // Another click closes it.
    ran = false;
    let c = centre(&ui, head.unwrap());
    mv(&mut ui, c);
    down(&mut ui, c);
    up(&mut ui, c);
    let list = frame(&mut ui, |ui| head = Some(world_fold(ui, &mut open, &mut ran)));
    assert!(!open && !ran);
    word(&list, "+");
}

#[test]
fn fold_open_sizes() {
    let mut ui = dark();
    let mut open = true;
    let (mut head, mut after) = (None, None);
    let list = frame(&mut ui, |ui| {
        ui.node("cell", col320(20.0, 234.0), |ui| {
            head = Some(ui.fold("world", "World", &mut open, |ui| {
                ui.label("Seed", Tone::Text);
                ui.label("Land", Tone::Text);
            }).id);
            after = Some(ui.node("after", Style { h: Dim::Px(10.0), shrink: 0.0, ..Style::default() }, |_| {}));
        });
    });
    rect_is(ui.rect(head.unwrap()).unwrap(), 20.0, 235.0, 320.0, 33.0);
    // A line top is the text's y less its half leading (1.2 for 12/18).
    near(word(&list, "Land").0.y - 1.2, 234.0 + 34.0 + 2.0 + 18.0 + 8.0);
    near(ui.rect(after.unwrap()).unwrap().y, 234.0 + 94.0);
}

#[test]
fn fold_hover_lights_the_sign_only() {
    let mut ui = dark();
    let t = Theme::dark();
    let (mut open, mut ran) = (false, false);
    let mut head = None;
    frame(&mut ui, |ui| head = Some(world_fold(ui, &mut open, &mut ran)));
    let c = centre(&ui, head.unwrap());
    mv(&mut ui, c);
    let list = frame(&mut ui, |ui| head = Some(world_fold(ui, &mut open, &mut ran)));
    assert_eq!(word(&list, "+").2, t.text);
    for c in "WORLD".chars() {
        assert_eq!(word(&list, &c.to_string()).2, t.text);
    }
    assert_eq!(fills(&list).len(), 1, "only the rule is filled");
}

#[test]
fn fold_grow_takes_the_free_height() {
    for (start, want) in [(true, 280.0), (false, 34.0)] {
        let mut ui = dark();
        let mut open = start;
        let mut after = None;
        frame(&mut ui, |ui| {
            let s = at(0.0, 0.0, Style { column: true, w: Dim::Px(320.0), h: Dim::Px(300.0), ..Style::default() });
            ui.node("cell", s, |ui| {
                ui.fold_with("f", "Grow", &mut open, FoldOpts { grow: true }, |_| {});
                after = Some(ui.node("after", Style { h: Dim::Px(20.0), shrink: 0.0, ..Style::default() }, |_| {}));
            });
        });
        near(ui.rect(after.unwrap()).unwrap().y, want);
    }
}

// ---- strip ----

/// The page's strip: views, then the separator, then the shapes (lines 588, 619, 589).
fn strip(ui: &mut Ui, x: f32, y: f32, view: usize, in3d: bool, disabled: &[usize]) -> Vec<Id> {
    let mut ids = Vec::new();
    ui.node("strip", at(x, y, strip_style()), |ui| {
        for (i, w) in ["Biomes", "Flat", "Relief", "Temperature", "Structures", "Section"].into_iter().enumerate() {
            let o = TabOpts { pressed: i == view, disabled: disabled.contains(&i) };
            ids.push(ui.tab(w, w, TabLook::Strip, o).id);
        }
        ui.strip_gap();
        for (i, w) in ["2D", "3D"].into_iter().enumerate() {
            ids.push(ui.tab(w, w, TabLook::Strip, TabOpts { pressed: (i == 1) == in3d, disabled: false }).id);
        }
    });
    ids
}

#[test]
fn strip_geometry() {
    let mut ui = dark();
    let mut ids = Vec::new();
    let list = frame(&mut ui, |ui| ids = strip(ui, 20.0, 36.0, 1, false, &[]));
    let want = [
        ("Biomes", 20.0, 69.2),
        ("Flat", 88.2, 54.8),
        ("Relief", 142.0, 69.2),
        ("Temperature", 210.2, 105.2),
        ("Structures", 314.4, 98.0),
        ("Section", 411.4, 76.4),
        ("2D", 498.8, 40.4),
        ("3D", 538.2, 40.4),
    ];
    for (id, (w, x, width)) in ids.iter().zip(want) {
        rect_is(ui.rect(*id).unwrap(), x, 36.0, width, 28.0);
        let g = word(&list, w);
        near(g.0.x, x + 13.0);
        near(g.0.y, 42.2);
    }
}

fn one_tab(ui: &mut Ui, opts: TabOpts) -> (Id, DrawList) {
    let mut id = None;
    let list = frame(ui, |ui| {
        ui.node("strip", at(20.0, 36.0, strip_style()), |ui| id = Some(ui.tab("t", "Flat", TabLook::Strip, opts).id));
    });
    (id.unwrap(), list)
}

fn frames_of(l: &DrawList) -> Vec<Color> {
    l.cmds.iter().filter_map(|c| if let Cmd::Frame { color, .. } = c { Some(*color) } else { None }).collect()
}

#[test]
fn strip_colours() {
    let t = Theme::dark();
    let plain = TabOpts::default();
    let pressed = TabOpts { pressed: true, disabled: false };
    // Normal, then hovered.
    let mut ui = dark();
    let (id, l) = one_tab(&mut ui, plain);
    assert_eq!(frames_of(&l), vec![t.edge]);
    assert!(fills(&l).is_empty());
    assert_eq!(word(&l, "Flat").2, t.muted);
    let c = centre(&ui, id);
    mv(&mut ui, c);
    assert_eq!(word(&one_tab(&mut ui, plain).1, "Flat").2, t.text);
    // Pressed, and pressed under the pointer: the word stays in the ground colour.
    let mut ui = dark();
    let (id, l) = one_tab(&mut ui, pressed);
    assert_eq!(fills(&l)[0].1, t.text);
    assert_eq!(frames_of(&l), vec![t.text]);
    assert_eq!(word(&l, "Flat").2, t.ground);
    let c = centre(&ui, id);
    mv(&mut ui, c);
    assert_eq!(word(&one_tab(&mut ui, pressed).1, "Flat").2, t.ground);
    // Disabled: faded, and no click.
    let off = TabOpts { pressed: false, disabled: true };
    let mut ui = dark();
    let (id, l) = one_tab(&mut ui, off);
    assert_eq!(frames_of(&l), vec![t.faded(t.edge, 0.35)]);
    assert_eq!(word(&l, "Flat").2, t.faded(t.muted, 0.35));
    let c = centre(&ui, id);
    mv(&mut ui, c);
    down(&mut ui, c);
    up(&mut ui, c);
    let mut clicked = true;
    frame(&mut ui, |ui| {
        ui.node("strip", at(20.0, 36.0, strip_style()), |ui| clicked = ui.tab("t", "Flat", TabLook::Strip, off).clicked);
    });
    assert!(!clicked);
}

#[test]
fn pressed_tab_paints_after_its_neighbours() {
    let t = Theme::dark();
    let mut ui = dark();
    let mut ids = Vec::new();
    let list = frame(&mut ui, |ids_ui| ids = strip(ids_ui, 20.0, 36.0, 1, false, &[]));
    let flat = ui.rect(ids[1]).unwrap();
    let relief = ui.rect(ids[2]).unwrap();
    let index = |r: Rect, c: Color| {
        list.cmds
            .iter()
            .position(|k| matches!(k, Cmd::Frame { rect, color, .. } if (rect.x - r.x).abs() < 1e-3 && *color == c))
            .expect("frame drawn")
    };
    assert!(index(flat, t.text) > index(relief, t.edge));
}

#[test]
fn tab_click_is_reported_and_the_app_selects() {
    let t = Theme::dark();
    let mut ui = dark();
    let mut sel = 1;
    let mut ids = Vec::new();
    frame(&mut ui, |ui| ids = strip(ui, 20.0, 36.0, sel, false, &[]));
    let c = centre(&ui, ids[2]);
    mv(&mut ui, c);
    down(&mut ui, c);
    up(&mut ui, c);
    let mut clicked = false;
    frame(&mut ui, |ui| {
        ui.node("strip", at(20.0, 36.0, strip_style()), |ui| {
            for (i, w) in ["Biomes", "Flat", "Relief"].into_iter().enumerate() {
                if ui.tab(w, w, TabLook::Strip, TabOpts { pressed: i == sel, disabled: false }).clicked {
                    clicked = true;
                    sel = i;
                }
            }
        });
    });
    assert!(clicked && sel == 2);
    let list = frame(&mut ui, |ui| ids = strip(ui, 20.0, 36.0, sel, false, &[]));
    let relief = ui.rect(ids[2]).unwrap();
    let flat = ui.rect(ids[1]).unwrap();
    let has_fill = |r: Rect| fills(&list).iter().any(|(f, c)| (f.x - r.x).abs() < 1e-3 && *c == t.text && f.h > 2.0);
    assert!(has_fill(relief));
    assert!(!has_fill(flat));
}

// ---- choice ----

#[test]
fn choice_looks() {
    let t = Theme::dark();
    let build = |ui: &mut Ui| {
        let mut ids = Vec::new();
        let l = frame(ui, |ui| {
            ui.node("choice", at(20.0, 156.0, choice_style()), |ui| {
                ids.push(ui.tab("pack", "Pack", TabLook::Choice, TabOpts { pressed: true, disabled: false }).id);
                ids.push(ui.tab("t2", "Try 2", TabLook::Choice, TabOpts::default()).id);
                ids.push(ui.tab("t4", "Try 4", TabLook::Choice, TabOpts { pressed: false, disabled: true }).id);
            });
        });
        (ids, l)
    };
    let mut ui = dark();
    let (ids, l) = build(&mut ui);
    rect_is(ui.rect(ids[0]).unwrap(), 20.0, 156.0, 40.8, 20.0);
    let t2 = ui.rect(ids[1]).unwrap();
    near(t2.x, 64.8);
    near(t2.w, 48.0);
    near(ui.rect(ids[2]).unwrap().x, 168.8 - 52.0);
    let f = fills(&l);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].1, t.text);
    assert_eq!(word(&l, "Pack").2, t.ground);
    assert_eq!(word(&l, "Try 2").2, t.muted);
    assert_eq!(word(&l, "Try 4").2, t.faded(t.muted, 0.35));
    let c = centre(&ui, ids[1]);
    mv(&mut ui, c);
    assert_eq!(word(&build(&mut ui).1, "Try 2").2, t.text);
}

// ---- details ----

#[test]
fn details_geometry_and_colours() {
    let t = Theme::dark();
    let mut ids = Vec::new();
    let mut ui = dark();
    let list = frame(&mut ui, |ui| {
        let s = at(460.0, 156.0, Style { w: Dim::Px(300.0), ..details_style() });
        ui.node("details", s, |ui| {
            ids.push(ui.detail_tab("a", "64 blocks", "10 s", TabOpts::default()).id);
            ids.push(ui.detail_tab("b", "32 blocks", "25 s", TabOpts { pressed: true, disabled: false }).id);
            ids.push(ui.detail_tab("c", "16 blocks", "1 min 30 s", TabOpts { pressed: false, disabled: true }).id);
        });
    });
    for (id, x) in ids.iter().zip([460.0, 561.0, 662.0]) {
        rect_is(ui.rect(*id).unwrap(), x, 156.0, 98.0, 42.5);
    }
    let w = word(&list, "64 blocks");
    near(w.0.x, 467.0);
    near(w.0.y - 1.2, 160.0);
    let s = word(&list, "10 s");
    near(s.0.y - 1.1, 178.0);
    assert_eq!(s.2, t.muted);
    assert_eq!(word(&list, "32 blocks").2, t.ground);
    assert_eq!(word(&list, "25 s").2, nui_ui::mix(t.ground, 0.7, t.text));
    assert!(fills(&list).iter().any(|(_, c)| *c == t.text));
    assert!(frames_of(&list).contains(&t.faded(t.edge, 0.35)));
}

// ---- number ----

fn setting(ui: &mut Ui, mut v: f64) -> Id {
    let style = at(
        460.0,
        298.0,
        Style {
            display: Display::Grid,
            cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Px(57.6)]),
            gap: Vec2::new(10.0, 0.0),
            align_items: Align::Center,
            pad: Edges::vh(2.0, 0.0),
            w: Dim::Px(330.0),
            ..Style::default()
        },
    );
    let mut id = None;
    ui.node("setting", style, |ui| {
        ui.label("Vertical scale", Tone::Text);
        ui.text("", TextLook::body(Tone::Muted));
        id = Some(ui.number("v", &mut v, NumberSpec::ANY).resp.id);
    });
    id.unwrap()
}

#[test]
fn number_field_right_aligned() {
    let t = Theme::dark();
    let mut ui = dark();
    let mut id = None;
    let list = frame(&mut ui, |ui| id = Some(setting(ui, -0.45)));
    rect_is(ui.rect(id.unwrap()).unwrap(), 732.4, 300.0, 57.6, 24.0);
    assert_eq!(frames_of(&list), vec![t.edge]);
    let g = word(&list, "-0.45");
    near(g.0.x, 747.0);
    near(g.0.y - 1.2, 303.0);
    for (v, s) in [(1.0, "1"), (-64.0, "-64"), (0.025, "0.025"), (-0.0, "0")] {
        let l = frame(&mut ui, |ui| {
            setting(ui, v);
        });
        word(&l, s);
    }
}
