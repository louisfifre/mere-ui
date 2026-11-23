//! The pieces step 18 added to the list: hover areas, part heads, round mark swatches, the pick links and
//! the plotted trend row (preview.html lines 310-317, 328-335, 341-348). No GPU: each check reads the draw
//! list or the last frame's rects.

use mere_base::{Button, Cmd, Color, DrawList, Event, Rect, Vec2};
use mere_layout::{Dim, Insets, Position, Style};
use mere_ui::{ActionLook, ListRow, PLOT_H, PartOpts, Resp, RowOpts, Theme, Ui};

const SIZE: Vec2 = Vec2::new(500.0, 400.0);

fn abs_at(x: f32, y: f32, w: f32) -> Style {
    Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(x), top: Dim::Px(y), ..Insets::default() },
        column: true,
        w: Dim::Px(w),
        ..Style::default()
    }
}

fn mv(ui: &mut Ui, x: f32, y: f32) {
    ui.event(&Event::PointerMoved { pos: Vec2::new(x, y) });
}

fn click(ui: &mut Ui, x: f32, y: f32) {
    let pos = Vec2::new(x, y);
    ui.event(&Event::PointerMoved { pos });
    ui.event(&Event::PointerDown { pos, button: Button::Left, clicks: 1 });
    ui.event(&Event::PointerUp { pos, button: Button::Left });
}

fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 0.01, "{a} != {b}");
}

fn fills(l: &DrawList) -> Vec<(Rect, Color)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Fill { rect, color } = c { Some((*rect, *color)) } else { None }).collect()
}

// U1
#[test]
fn hover_area_tells_its_body_where_the_pointer_is_across_it() {
    let mut ui = Ui::new(Theme::dark());
    let mut seen = Vec::new();
    let frame = |ui: &mut Ui| {
        ui.begin_frame(SIZE);
        let mut got = None;
        ui.node("block", abs_at(20.0, 20.0, 200.0), |ui| {
            ui.hover_area("h", Style { column: true, ..Style::default() }, |ui, at| {
                got = Some(at);
                ui.node("fill", Style { h: Dim::Px(50.0), ..Style::default() }, |_| {});
            });
        });
        ui.end_frame();
        got.unwrap()
    };
    seen.push(frame(&mut ui));
    mv(&mut ui, 70.0, 40.0);
    seen.push(frame(&mut ui));
    mv(&mut ui, 300.0, 40.0);
    seen.push(frame(&mut ui));
    assert!(seen[0].is_none(), "first frame: the area has no place yet");
    let at = seen[1].expect("pointer inside");
    assert!((at - 0.25).abs() < 0.001, "x 70 of 20..220 is a quarter, got {at}");
    assert!(seen[2].is_none(), "pointer outside");
}

fn part_frame(ui: &mut Ui, opts: PartOpts) -> (Resp, DrawList) {
    ui.begin_frame(SIZE);
    let mut resp = None;
    ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| {
        resp = Some(ui.list_part("p", "On the surface", "12", opts));
    });
    let list = ui.end_frame().clone();
    (resp.unwrap(), list)
}

// U2
#[test]
fn list_part_pads_colours_and_clicks_as_the_part_rule() {
    let t = Theme::dark();
    let mut ui = Ui::new(t);
    // Not first: 12 above, 4 below, one 16.5 line.
    part_frame(&mut ui, PartOpts { action: true, ..PartOpts::default() });
    let (r, _) = part_frame(&mut ui, PartOpts { action: true, ..PartOpts::default() });
    near(r.rect.unwrap().h, 12.0 + 16.5 + 4.0);
    let share = ui.rect(r.id.child("share")).expect("share cell");
    assert!(share.w >= 39.6 - 0.01, "the share cell is at least 6 ch of 11 px: {}", share.w);
    near(share.x + share.w, 350.0);

    let mut ui = Ui::new(t);
    part_frame(&mut ui, PartOpts { first: true, action: true, pressed: true });
    let (r, list) = part_frame(&mut ui, PartOpts { first: true, action: true, pressed: true });
    near(r.rect.unwrap().h, 2.0 + 16.5 + 4.0);
    let name = list.cmds.iter().find_map(|c| match c {
        Cmd::Text { at, text, color, .. } if text == "On the surface" => Some((*at, *color)),
        _ => None,
    });
    let (at, color) = name.expect("the name is drawn");
    assert_eq!(color, t.text, "a pressed part reads in the text colour");
    // 20 + 12 + 8 across; 2 down, then the half leading of a 16.5 line over 11 px.
    near(at.x, 40.0);
    assert!(at.y >= 22.0 && at.y < 24.0, "content starts 2 px down, got {}", at.y);

    // A note is never clicked; a button is.
    let mut ui = Ui::new(t);
    let note = PartOpts { action: false, ..PartOpts::default() };
    part_frame(&mut ui, note);
    click(&mut ui, 100.0, 40.0);
    let (r, list) = part_frame(&mut ui, note);
    assert!(!r.clicked, "a note takes no click");
    let name = list.cmds.iter().find_map(|c| match c {
        Cmd::Text { color, text, .. } if text == "On the surface" => Some(*color),
        _ => None,
    });
    assert_eq!(name, Some(t.muted), "not pressed: muted");
    assert!(!fills(&list).iter().any(|(r, _)| r.h == 1.0), "no hover look on a note");

    let mut ui = Ui::new(t);
    let button = PartOpts { action: true, ..PartOpts::default() };
    part_frame(&mut ui, button);
    click(&mut ui, 100.0, 40.0);
    let (r, _) = part_frame(&mut ui, button);
    assert!(r.clicked, "a button part takes the click");
}

// U3
#[test]
fn a_mark_swatch_is_a_disc_in_a_ring() {
    let mut ui = Ui::new(Theme::dark());
    let mut list = DrawList::default();
    for _ in 0..2 {
        ui.begin_frame(SIZE);
        ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| {
            let row = ListRow { swatch: Color::hex(0x6a9a4a), name: "Village", width: "1,250 blocks", share: "12" };
            ui.list_row("r", &row, RowOpts { mark: true, ..RowOpts::default() });
        });
        list = ui.end_frame().clone();
    }
    // Row padding 4, the 12 px swatch centred in the 18 px line: its centre is 13 px down and 6 across.
    let centre = Vec2::new(26.0, 33.0);
    let disc = list.cmds.iter().find_map(|c| match c {
        Cmd::Disc { center, radius, .. } => Some((*center, *radius)),
        _ => None,
    });
    let ring = list.cmds.iter().find_map(|c| match c {
        Cmd::Ring { center, radius, width, color } => Some((*center, *radius, *width, *color)),
        _ => None,
    });
    assert_eq!(disc, Some((centre, 5.0)));
    let (c, r, w, col) = ring.expect("a ring");
    assert_eq!((c, r, w), (centre, 5.5, 1.0));
    near(col.a, 0.55);
    assert!(!list.cmds.iter().any(|c| matches!(c, Cmd::Frame { .. })), "no square frame behind the round mark");
}

fn pick_frame(ui: &mut Ui, look: ActionLook) -> DrawList {
    ui.begin_frame(SIZE);
    ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| {
        ui.action("b", "Show all", look);
    });
    ui.end_frame().clone()
}

// U4
#[test]
fn pick_strikes_on_hover_and_pick_none_is_underlined_at_rest() {
    let t = Theme::dark();
    let rule = |l: &DrawList| fills(l).into_iter().filter(|(r, _)| r.h == 1.0).collect::<Vec<_>>();
    let mut ui = Ui::new(t);
    pick_frame(&mut ui, ActionLook::Pick);
    let rest = pick_frame(&mut ui, ActionLook::Pick);
    assert!(rule(&rest).is_empty(), "Pick has no decoration at rest");
    mv(&mut ui, 30.0, 28.0);
    let hot = pick_frame(&mut ui, ActionLook::Pick);
    let lines = rule(&hot);
    assert_eq!(lines.len(), 1, "one strike rule on hover");
    near(lines[0].0.w, 8.0 * 6.6);
    assert_eq!(lines[0].1, t.text);

    let mut ui = Ui::new(t);
    pick_frame(&mut ui, ActionLook::PickNone);
    let rest = pick_frame(&mut ui, ActionLook::PickNone);
    let under = rule(&rest);
    assert_eq!(under.len(), 1, "PickNone is underlined at rest");
    mv(&mut ui, 30.0, 28.0);
    let hot = pick_frame(&mut ui, ActionLook::PickNone);
    assert_eq!(rule(&hot).len(), 1, "and gains no strike on hover");
}

// U5
#[test]
fn a_trend_row_draws_its_plot_at_the_slot() {
    let mut ui = Ui::new(Theme::dark());
    let line = vec![Vec2::new(0.0, 26.0), Vec2::new(50.0, 4.0)];
    let mut slot = None;
    let mut list = DrawList::default();
    for _ in 0..2 {
        ui.begin_frame(SIZE);
        ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| {
            let plot = vec![Cmd::Polyline { points: line.clone(), width: 1.5, color: Color::hex(0xececec) }];
            slot = Some(ui.trend_row("t", "Flat ground", "21.4%", plot));
        });
        list = ui.end_frame().clone();
    }
    let origin = ui.rect(slot.unwrap()).expect("the slot has a rect");
    near(origin.h, PLOT_H);
    let got = list.cmds.iter().find_map(|c| if let Cmd::Polyline { points, width, .. } = c { Some((points.clone(), *width)) } else { None });
    let (points, width) = got.expect("a polyline");
    assert_eq!(width, 1.5);
    assert_eq!(points, line.iter().map(|p| Vec2::new(p.x + origin.x, p.y + origin.y)).collect::<Vec<_>>());
}
