//! The scroll area, the ellipsis, the list rows and the trend row, without a GPU: the numbers come from the
//! page's CSS (preview.html lines 133-137, 310-325, 337-363) and each is derived in its test's comment.

use nui_base::{Button, Cmd, Color, DrawList, Event, Rect, Vec2};
use nui_layout::{Dim, Display, Edges, Insets, Position, Style};
use nui_ui::{ListRow, Resp, RowOpts, ScrollOpts, ScrollResp, TextLook, Theme, Tone, Ui, ellipsize, scroll_style};

const SIZE: Vec2 = Vec2::new(860.0, 600.0);

fn body() -> TextLook {
    TextLook::body(Tone::Text)
}

fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 0.01, "{a} != {b}");
}

fn same(a: Color, b: Color) {
    let px = |c: Color| [c.r, c.g, c.b].map(|v| (v * 255.0).round() as i32);
    assert_eq!(px(a), px(b), "{a:?} != {b:?}");
}

fn texts(l: &DrawList) -> Vec<(Vec2, String, Color)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Text { at, text, color, .. } = c { Some((*at, text.clone(), *color)) } else { None }).collect()
}

fn text_at(l: &DrawList, s: &str) -> (Vec2, Color) {
    let (at, _, c) = texts(l).into_iter().find(|t| t.1 == s).unwrap_or_else(|| panic!("no text {s:?} in {:?}", texts(l)));
    (at, c)
}

fn fills(l: &DrawList) -> Vec<(Rect, Color)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Fill { rect, color } = c { Some((*rect, *color)) } else { None }).collect()
}

fn abs_at(x: f32, y: f32, w: f32) -> Style {
    Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(x), top: Dim::Px(y), ..Insets::default() },
        column: true,
        w: Dim::Px(w),
        gap: Vec2::new(0.0, 6.0),
        ..Style::default()
    }
}

fn mv(ui: &mut Ui, x: f32, y: f32) {
    ui.event(&Event::PointerMoved { pos: Vec2::new(x, y) });
}
fn wheel(ui: &mut Ui, x: f32, y: f32, delta: f32) {
    ui.event(&Event::Wheel { pos: Vec2::new(x, y), delta });
}

/// The page's list block at 1400 px: a legend and a 200 px scroll of 20 rows, all in a 330 px column.
fn fixture(ui: &mut Ui, y: &mut f32, opts: impl Fn(usize) -> RowOpts) -> (DrawList, ScrollResp, Vec<Resp>) {
    ui.begin_frame(SIZE);
    let mut sr = None;
    let mut rows = Vec::new();
    ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| {
        ui.list_head("head", "biome", "typical width", "share", false);
        // basis Auto: the page's `flex: 1` would otherwise win over the 200 px height in a column.
        let style = Style { h: Dim::Px(200.0), grow: 0.0, basis: Dim::Auto, ..scroll_style() };
        sr = Some(ui.scroll("list", y, style, ScrollOpts::LIST, |ui| {
            for i in 0..20 {
                let (name, width, share) =
                    if i == 0 { ("Windswept gravelly hills".to_string(), "1,250 blocks", "12.3%") } else { (format!("Biome {i}"), "350 blocks", "1.0%") };
                let row = ListRow { swatch: Color::hex(0x6a9a4a), name: &name, width, share };
                rows.push(ui.list_row(&format!("row{i}"), &row, opts(i)));
            }
        }));
    });
    let list = ui.end_frame().clone();
    (list, sr.unwrap(), rows)
}

fn plain(_: usize) -> RowOpts {
    RowOpts::default()
}

#[test]
fn ellipsize_cases() {
    // "Windswept gravelly hills" is 24 chars of 7.2 px: 172.8.
    let s = "Windswept gravelly hills";
    assert_eq!(ellipsize(s, body(), 172.8), s);
    // 172.7: n = floor((172.7 - 7.2) / 7.2) = floor(22.99) = 22.
    assert_eq!(ellipsize(s, body(), 172.7), format!("{}\u{2026}", &s[..22]));
    // 153.4: floor((153.4 - 7.2) / 7.2) = floor(20.3) = 20.
    assert_eq!(ellipsize(s, body(), 153.4), "Windswept gravelly h\u{2026}");
    // Narrower than the ellipsis cell: nothing but the ellipsis (the painter clips it).
    assert_eq!(ellipsize(s, body(), 5.0), "\u{2026}");
    assert_eq!(ellipsize("", body(), 5.0), "");
}

#[test]
fn first_frame_has_no_gutter_and_asks_again() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    let (list, sr, _) = fixture(&mut ui, &mut y, plain);
    // Column 330 - 12 (swatch) - 3 x 8 (gaps) - 86.4 ("1,250 blocks") - 43.2 (share min) = 164.4;
    // n = floor((164.4 - 7.2) / 7.2) = 21.
    let (at, _) = text_at(&list, "Windswept gravelly hi\u{2026}");
    near(at.x, 20.0 + 12.0 + 8.0);
    assert!(ui.want_redraw(), "the list overflows, so the gutter comes one frame later");
    assert_eq!(sr.max, 0.0);
}

#[test]
fn second_frame_reserves_the_gutter() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    let (list, sr, _) = fixture(&mut ui, &mut y, plain);
    assert!(!ui.want_redraw());
    // The box is 340 wide (margin -10), its padding is 10 + 11, so a row is 319 wide and rows end at 339.
    // Name column: 319 - 12 - 24 - 86.4 - 43.2 = 153.4, n = 20.
    text_at(&list, "Windswept gravelly h\u{2026}");
    // 20 + 12 + 8 + 153.4 + 8 = 201.4.
    near(text_at(&list, "1,250 blocks").0.x, 201.4);
    // The share cell starts at 201.4 + 86.4 + 8 = 295.8, is 43.2 wide, and "12.3%" is 36: 295.8 + 7.2 = 303.
    near(text_at(&list, "12.3%").0.x, 303.0);
    near(303.0 + 36.0, 339.0);
    // 20 rows of 24 + the 28 px bottom padding = 508, in a view of 200.
    near(sr.max, 308.0);
    assert!(sr.overflow);
}

#[test]
fn thumb_at_thirty_percent() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    let (l2, _, _) = fixture(&mut ui, &mut y, plain);
    let row0 = text_at(&l2, "Windswept gravelly h\u{2026}").0.y;
    y = 0.3 * 308.0;
    let (l3, _, _) = fixture(&mut ui, &mut y, plain);
    // thumb height = 200 x 200 / 508 = 78.74; top = 42.5 + (200 - 78.74) x 0.3; x = 20 + 340 - 11 + 2 = 351.
    let line = Theme::dark().line;
    let thumb = fills(&l3).into_iter().find(|(r, c)| r.x == 351.0 && (c.r - line.r).abs() < 1e-4).expect("a thumb").0;
    near(thumb.x, 351.0);
    near(thumb.y, 42.5 + (200.0 - 78.74) * 0.3);
    near(thumb.w, 7.0);
    near(thumb.h, 78.74);
    near(text_at(&l3, "Windswept gravelly h\u{2026}").0.y, row0 - 92.4);
}

#[test]
fn fade_bands() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    let (l, _, _) = fixture(&mut ui, &mut y, plain);
    let g = Theme::dark().ground;
    let bands: Vec<_> = fills(&l).into_iter().filter(|(r, c)| r.h == 1.0 && r.w == 340.0 && c.r == g.r && c.a < 1.0).collect();
    assert_eq!(bands.len(), 28);
    // Box bottom 42.5 + 200 = 242.5; the first band is 28 px above it, and alpha rises by 1/28 per band.
    near(bands[0].0.y, 42.5 + 172.0);
    near(bands[0].1.a, 0.5 / 28.0);
    near(bands[27].0.y, 42.5 + 199.0);
    near(bands[27].1.a, 27.5 / 28.0);
    assert!(bands.iter().all(|(r, _)| r.x == 20.0));
}

#[test]
fn wheel_scrolls_clamps_and_needs_the_pointer_inside() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    fixture(&mut ui, &mut y, plain);
    // One notch away from the user (-1 here) moves the content up by 100.
    wheel(&mut ui, 100.0, 100.0, -1.0);
    fixture(&mut ui, &mut y, plain);
    near(y, 100.0);
    wheel(&mut ui, 100.0, 100.0, -10.0);
    fixture(&mut ui, &mut y, plain);
    near(y, 308.0);
    // Outside the box (x 500 is right of it): nothing moves.
    wheel(&mut ui, 500.0, 100.0, 5.0);
    fixture(&mut ui, &mut y, plain);
    near(y, 308.0);
    wheel(&mut ui, 100.0, 100.0, 10.0);
    fixture(&mut ui, &mut y, plain);
    near(y, 0.0);
}

#[test]
fn hidden_rows_take_no_pointer() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    fixture(&mut ui, &mut y, plain);
    y = 92.4;
    fixture(&mut ui, &mut y, plain);
    // Row 3 spans 42.5 + 72 - 92.4 = 22.1 to 46.1: the legend (20 to 36.5) lies over its top, which is
    // scrolled out of the box. A pointer at y 30 is on the row's rect but not on its visible part.
    mv(&mut ui, 100.0, 30.0);
    let (_, _, rows) = fixture(&mut ui, &mut y, plain);
    assert!(!rows[3].hovered);
    // Row 4 spans 46.1 to 70.1, inside the box.
    mv(&mut ui, 100.0, 50.0);
    let (l, _, rows) = fixture(&mut ui, &mut y, plain);
    assert!(rows[4].hovered);
    let name = text_at(&l, "Biome 4");
    let w = body().width("Biome 4");
    assert!(fills(&l).iter().any(|(r, _)| r.h == 1.0 && (r.x - name.0.x).abs() < 0.01 && (r.w - w).abs() < 0.01), "an underline under the name");
}

#[test]
fn thumb_drag() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    fixture(&mut ui, &mut y, plain);
    y = 92.4;
    fixture(&mut ui, &mut y, plain);
    // The thumb is 78.74 high at 42.5 + 36.378 = 78.878: its centre is at y 118.248, x 354.5.
    let c = Vec2::new(354.5, 118.248);
    ui.event(&Event::PointerDown { pos: c, button: Button::Left, clicks: 1 });
    fixture(&mut ui, &mut y, plain);
    near(y, 92.4);
    // 50 px down the track of 200 - 78.74 = 121.26 is 50 / 121.26 of the range 308.
    mv(&mut ui, c.x, c.y + 50.0);
    fixture(&mut ui, &mut y, plain);
    near(y, 92.4 + 50.0 / 121.26 * 308.0);
    ui.event(&Event::PointerUp { pos: Vec2::new(c.x, c.y + 50.0), button: Button::Left });
    fixture(&mut ui, &mut y, plain);
    let held = y;
    mv(&mut ui, c.x, c.y + 80.0);
    fixture(&mut ui, &mut y, plain);
    near(y, held);
}

#[test]
fn opacity_states() {
    let t = Theme::dark();
    let swatch = Color::hex(0x6a9a4a);
    let scene = |ui: &mut Ui, opts: RowOpts, pointer: Option<Vec2>| {
        if let Some(p) = pointer {
            ui.event(&Event::PointerMoved { pos: p });
        }
        ui.begin_frame(SIZE);
        let mut r = None;
        ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| {
            r = Some(ui.list_row("row", &ListRow { swatch, name: "Plains", width: "350 blocks", share: "1.0%" }, opts));
        });
        let l = ui.end_frame().clone();
        (l, r.unwrap())
    };
    let choosing = RowOpts { choosing: true, ..RowOpts::default() };
    let mut ui = Ui::new(t);
    let (l, _) = scene(&mut ui, choosing, None);
    same(text_at(&l, "Plains").1, t.faded(t.text, 0.5));
    // The row is 24 high at y 20: a pointer at y 32 is on it.
    scene(&mut ui, choosing, Some(Vec2::new(100.0, 32.0)));
    let (l, r) = scene(&mut ui, choosing, None);
    assert!(r.hovered);
    same(text_at(&l, "Plains").1, t.faded(t.text, 0.85));
    let (l, _) = scene(&mut ui, RowOpts { pressed: true, ..choosing }, None);
    same(text_at(&l, "Plains").1, t.text);

    let mut ui = Ui::new(t);
    let absent = RowOpts { absent: true, ..RowOpts::default() };
    let (l, _) = scene(&mut ui, absent, None);
    same(text_at(&l, "Plains").1, t.muted);
    assert!(fills(&l).iter().any(|(r, c)| r.w == 12.0 && r.h == 12.0 && c.r == t.faded(swatch, 0.5).r && c.g == t.faded(swatch, 0.5).g));
    assert!(texts(&l).iter().all(|t| t.1 != "350 blocks" && t.1 != "1.0%"), "an absent row has no width or share");
    ui.event(&Event::PointerMoved { pos: Vec2::new(100.0, 32.0) });
    scene(&mut ui, absent, None);
    ui.event(&Event::PointerDown { pos: Vec2::new(100.0, 32.0), button: Button::Left, clicks: 1 });
    scene(&mut ui, absent, None);
    ui.event(&Event::PointerUp { pos: Vec2::new(100.0, 32.0), button: Button::Left });
    let (_, r) = scene(&mut ui, absent, None);
    assert!(!r.clicked);
}

#[test]
fn legend_columns() {
    let t = Theme::dark();
    for narrow in [false, true] {
        let mut ui = Ui::new(t);
        ui.begin_frame(SIZE);
        ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| ui.list_head("head", "biome", "typical width", "share", narrow));
        let l = ui.end_frame().clone();
        // The row's right edge is 350. "share" is 5 x 6.6 = 33 wide in a cell of at least 39.6 (33 narrow).
        near(text_at(&l, "share").0.x, 350.0 - 33.0);
        // "typical width" is 13 x 6.6 = 85.8, wider than its minimum, so its cell is its text:
        // 350 - cell(39.6 or 33) - 8 - 85.8.
        near(text_at(&l, "typical width").0.x, 350.0 - if narrow { 33.0 } else { 39.6 } - 8.0 - 85.8);
        let (at, color) = text_at(&l, "biome");
        near(at.x, 20.0 + 12.0 + 8.0);
        same(color, t.muted);
    }
    // The widest case of the narrow minimum 46.2 for the width cell: a short word is held to it.
    let mut ui = Ui::new(t);
    ui.begin_frame(SIZE);
    ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| ui.list_head("head", "biome", "w", "s", true));
    let l = ui.end_frame().clone();
    // Cells of 46.2 and 33, right-aligned: "w" ends at 350 - 33 - 8 = 309, so it starts at 309 - 6.6.
    near(text_at(&l, "w").0.x, 309.0 - 6.6);
    near(text_at(&l, "s").0.x, 350.0 - 6.6);
}

#[test]
fn trend_row_layout() {
    let mut ui = Ui::new(Theme::dark());
    ui.begin_frame(SIZE);
    let mut plots = Vec::new();
    let col = Style { display: Display::Flex, pad: Edges::default(), ..abs_at(20.0, 20.0, 140.0) };
    ui.node("history", Style { gap: Vec2::new(0.0, 8.0), ..col }, |ui| {
        plots.push(ui.trend_row("a", "Typical flat zone", "41.2%", Vec::new()));
        plots.push(ui.trend_row("b", "Sea", "5.0%", Vec::new()));
    });
    let l = ui.end_frame().clone();
    // Name column: 140 - 10 (gap) - 36 ("41.2%") = 94; n = floor((94 - 7.2) / 7.2) = 12.
    let (name, _) = text_at(&l, "Typical flat\u{2026}");
    near(name.x, 20.0);
    near(text_at(&l, "41.2%").0.x, 20.0 + 104.0);
    // The plot slot sits under the line of 18 px and is 30 high and as wide as the row.
    let p = ui.rect(plots[0]).unwrap();
    near(p.y, 20.0 + 18.0);
    near(p.h, 30.0);
    near(p.w, 140.0);
    // The row is 48 high; the next one starts after the gap of 8.
    near(ui.rect(plots[1]).unwrap().y, 20.0 + 56.0 + 18.0);
}
