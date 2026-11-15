//! The scroll area, the ellipsis, the list rows and the trend row, without a GPU: the numbers come from the
//! page's CSS (preview.html lines 133-137, 310-325, 337-363) and each is derived in its test's comment.

#[allow(unused_imports)]
use nui_ui::tokens::*;
use nui_base::{Button, Cmd, Color, DrawList, Event, Rect, Vec2};
use nui_layout::{Dim, Display, Edges, Insets, Position, Style};
use nui_ui::{ARROW, GUTTER, ListRow, PLOT_H, Resp, RowOpts, ScrollOpts, ScrollResp, THUMB_W, TextLook, Theme, Tone, Ui, ellipsize, scroll_style};

/// A list row: the 18 px line and `INSET_TIGHT` above and below.
const ROW_H: f32 = 18.0 + 2.0 * INSET_TIGHT;
/// 20 rows and the scroll box's bottom padding (`SECTION`), in a view of 200.
const CONTENT_H: f32 = 20.0 * ROW_H + SECTION;
const MAX: f32 = CONTENT_H - 200.0;
/// The box's width: the 330 px column and the gutter it reaches into the margin.
const BOX_W: f32 = 330.0 + GUTTER;
/// The track between the two arrows, and the thumb's length in it.
const TRACK_H: f32 = 200.0 - 2.0 * ARROW;
const THUMB_H: f32 = TRACK_H * 200.0 / CONTENT_H;
/// The box starts under the 16.5 legend line and the column's gap of 6.
const BOX_Y: f32 = 20.0 + 16.5 + 6.0;

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

fn same_px(a: Color, b: Color) -> bool {
    let px = |c: Color| [c.r, c.g, c.b].map(|v| (v * 255.0).round() as i32);
    px(a) == px(b)
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
        ui.list_head("head", "biome", "typical width", "share");
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
    // The box is 342 wide (margin -12), its padding is 12 + 12, so a row is 318 wide and rows end at 338.
    // Name column: 318 - 12 - 24 - 86.4 - 43.2 = 152.4, n = 20.
    text_at(&list, "Windswept gravelly h\u{2026}");
    // 20 + 12 + 8 + 152.4 + 8 = 200.4.
    near(text_at(&list, "1,250 blocks").0.x, 200.4);
    // The share cell starts at 200.4 + 86.4 + 8 = 294.8, is 43.2 wide, and "12.3%" is 36: 294.8 + 7.2 = 302.
    near(text_at(&list, "12.3%").0.x, 302.0);
    near(302.0 + 36.0, 338.0);
    // 20 rows of 26 + the 24 px bottom padding = 544, in a view of 200.
    near(sr.max, 344.0);
    assert!(sr.overflow);
}

#[test]
fn thumb_at_thirty_percent() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    let (l2, _, _) = fixture(&mut ui, &mut y, plain);
    let row0 = text_at(&l2, "Windswept gravelly h\u{2026}").0.y;
    y = 0.3 * MAX;
    let (l3, _, _) = fixture(&mut ui, &mut y, plain);
    // The track runs between two 12 px arrows: 176 long from 42.5 + 12 = 54.5. Thumb height = 176 x 200 / 544 = 64.71;
    // top = 54.5 + (176 - 64.71) x 0.3; the gutter is 350 to 362: the 4 px thumb is at 354 and the 1 px track at 356.
    let t = Theme::dark();
    let thumb = fills(&l3).into_iter().find(|(r, c)| r.w == THUMB_W && same_px(*c, t.muted)).expect("a thumb").0;
    near(thumb.x, 354.0);
    near(thumb.y, BOX_Y + ARROW + (TRACK_H - THUMB_H) * 0.3);
    near(thumb.h, THUMB_H);
    // The track is a 1 px line in the theme's line colour, inside the thumb, the full length between the arrows.
    let track = fills(&l3).into_iter().find(|(r, c)| r.w == 1.0 && r.h > 100.0 && same_px(*c, t.line)).expect("a track").0;
    near(track.x, 356.0);
    near(track.y, BOX_Y + ARROW);
    near(track.h, TRACK_H);
    near(text_at(&l3, "Windswept gravelly h\u{2026}").0.y, row0 - 0.3 * MAX);
}

/// The chevrons drawn this frame: (is it in the top half of the box, its colour).
fn chevrons(l: &DrawList) -> Vec<(bool, Color)> {
    l.cmds
        .iter()
        .filter_map(|c| match c {
            Cmd::Polyline { points, width, color } if points.len() == 3 && *width == 1.0 => Some((points[1].y < 142.5, *color)),
            _ => None,
        })
        .collect()
}

#[test]
fn arrows_show_only_where_content_is_left() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    fixture(&mut ui, &mut y, plain);
    // At the top: nothing above, so the bottom arrow alone.
    let (l, _, _) = fixture(&mut ui, &mut y, plain);
    assert_eq!(chevrons(&l).iter().map(|c| c.0).collect::<Vec<_>>(), [false]);
    // Midway: both.
    y = 150.0;
    let (l, _, _) = fixture(&mut ui, &mut y, plain);
    let mut both: Vec<_> = chevrons(&l).iter().map(|c| c.0).collect();
    both.sort();
    assert_eq!(both, [false, true]);
    // At the end: the top arrow alone.
    y = MAX;
    let (l, _, _) = fixture(&mut ui, &mut y, plain);
    assert_eq!(chevrons(&l).iter().map(|c| c.0).collect::<Vec<_>>(), [true]);
    // Resting, an arrow is the muted colour.
    assert!(same_px(chevrons(&l)[0].1, Theme::dark().muted));
}

#[test]
fn an_arrow_click_scrolls_one_wheel_step() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    fixture(&mut ui, &mut y, plain);
    let click = |ui: &mut Ui, y: &mut f32, p: Vec2| {
        ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
        fixture(ui, y, plain);
        ui.event(&Event::PointerUp { pos: p, button: Button::Left });
        fixture(ui, y, plain);
    };
    // The box is 42.5 to 242.5: the top arrow is absent at the start, so a press there does nothing.
    let (top, bottom) = (Vec2::new(354.5, 47.5), Vec2::new(354.5, 237.5));
    click(&mut ui, &mut y, top);
    near(y, 0.0);
    // The bottom arrow moves the content up by one step, and then the top arrow brings it back.
    click(&mut ui, &mut y, bottom);
    near(y, 100.0);
    click(&mut ui, &mut y, top);
    near(y, 0.0);
}

#[test]
fn the_thumb_takes_the_text_colour_while_hovered() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    fixture(&mut ui, &mut y, plain);
    let t = Theme::dark();
    mv(&mut ui, 356.0, 60.0);
    let (l, _, _) = fixture(&mut ui, &mut y, plain);
    assert!(fills(&l).iter().any(|(r, c)| r.w == THUMB_W && same_px(*c, t.text)));
    mv(&mut ui, 100.0, 60.0);
    let (l, _, _) = fixture(&mut ui, &mut y, plain);
    assert!(fills(&l).iter().any(|(r, c)| r.w == THUMB_W && same_px(*c, t.muted)));
}

#[test]
fn fade_bands() {
    let mut ui = Ui::new(Theme::dark());
    let mut y = 0.0;
    fixture(&mut ui, &mut y, plain);
    let (l, _, _) = fixture(&mut ui, &mut y, plain);
    let g = Theme::dark().ground;
    let bands: Vec<_> = fills(&l).into_iter().filter(|(r, c)| r.h == 1.0 && r.w == BOX_W && c.r == g.r && c.a < 1.0).collect();
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
    near(y, MAX);
    // Outside the box (x 500 is right of it): nothing moves.
    wheel(&mut ui, 500.0, 100.0, 5.0);
    fixture(&mut ui, &mut y, plain);
    near(y, MAX);
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
    // Row 3 spans 42.5 + 78 - 92.4 = 28.1 to 54.1: the legend (20 to 36.5) lies over its top, which is
    // scrolled out of the box. A pointer at y 30 is on the row's rect but not on its visible part.
    mv(&mut ui, 100.0, 30.0);
    let (_, _, rows) = fixture(&mut ui, &mut y, plain);
    assert!(!rows[3].hovered);
    // Row 4 spans 54.1 to 80.1, inside the box.
    mv(&mut ui, 100.0, 60.0);
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
    // The thumb is 64.71 high at 54.5 + 111.29 x (92.4 / 344) = 84.39: its centre is half its height lower, at x 356.
    let room = TRACK_H - THUMB_H;
    let c = Vec2::new(356.0, BOX_Y + ARROW + room * 92.4 / MAX + THUMB_H / 2.0);
    ui.event(&Event::PointerDown { pos: c, button: Button::Left, clicks: 1 });
    fixture(&mut ui, &mut y, plain);
    near(y, 92.4);
    // 50 px down the room of 176 - 64.71 = 111.29 is 50 / 111.29 of the range 344.
    mv(&mut ui, c.x, c.y + 50.0);
    fixture(&mut ui, &mut y, plain);
    near(y, 92.4 + 50.0 / room * MAX);
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
    same(text_at(&l, "Plains").1, t.faded(t.text, FADE_DIM));
    // The row is 26 high at y 20: a pointer at y 32 is on it.
    scene(&mut ui, choosing, Some(Vec2::new(100.0, 32.0)));
    let (l, r) = scene(&mut ui, choosing, None);
    assert!(r.hovered);
    same(text_at(&l, "Plains").1, t.faded(t.text, FADE_LIT));
    let (l, _) = scene(&mut ui, RowOpts { pressed: true, ..choosing }, None);
    same(text_at(&l, "Plains").1, t.text);

    let mut ui = Ui::new(t);
    let absent = RowOpts { absent: true, ..RowOpts::default() };
    let (l, _) = scene(&mut ui, absent, None);
    same(text_at(&l, "Plains").1, t.muted);
    assert!(fills(&l).iter().any(|(r, c)| r.w == 12.0 && r.h == 12.0 && c.r == t.faded(swatch, FADE_DIM).r && c.g == t.faded(swatch, FADE_DIM).g));
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
    let mut ui = Ui::new(t);
    ui.begin_frame(SIZE);
    ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| {
        ui.list_head("head", "biome", "typical width", "share");
    });
    let l = ui.end_frame().clone();
    // The row's right edge is 350. "share" is 5 x 6.6 = 33 wide in a cell of at least 39.6.
    near(text_at(&l, "share").0.x, 350.0 - 33.0);
    // "typical width" is 13 x 6.6 = 85.8, wider than its minimum, so its cell is its text: 350 - 39.6 - 8 - 85.8.
    near(text_at(&l, "typical width").0.x, 350.0 - 39.6 - 8.0 - 85.8);
    let (at, color) = text_at(&l, "biome");
    near(at.x, 20.0 + 12.0 + 8.0);
    same(color, t.muted);
    // A short word is held to the minimum cells 52.8 and 39.6, right-aligned.
    let mut ui = Ui::new(t);
    ui.begin_frame(SIZE);
    ui.node("block", abs_at(20.0, 20.0, 330.0), |ui| {
        ui.list_head("head", "biome", "w", "s");
    });
    let l = ui.end_frame().clone();
    near(text_at(&l, "w").0.x, 350.0 - 39.6 - 8.0 - 6.6);
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
    // Name column: 140 - 8 (gap) - 36 ("41.2%") = 96; n = floor((96 - 7.2) / 7.2) = 12.
    let (name, _) = text_at(&l, "Typical flat\u{2026}");
    near(name.x, 20.0);
    near(text_at(&l, "41.2%").0.x, 20.0 + 104.0);
    // The plot slot sits under the line of 18 px and is PLOT_H (32) high and as wide as the row.
    let p = ui.rect(plots[0]).unwrap();
    near(p.y, 20.0 + 18.0);
    near(p.h, PLOT_H);
    near(p.w, 140.0);
    // The row is 50 high (18 + 32); the next one starts after the gap of 8.
    near(ui.rect(plots[1]).unwrap().y, 20.0 + 58.0 + 18.0);
}
