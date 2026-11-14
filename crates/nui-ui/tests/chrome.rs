//! Icons as paths, the pointer's shapes, the tips and the window's edges, without a GPU.

use nui_base::{Button, Cmd, DrawList, Edge, Event, Hit, Vec2, Cursor};
use nui_ui::{ButtonLook, ButtonOpts, FieldOpts, Icon, TIP_DELAY, Theme, Ui, grip};

const SIZE: Vec2 = Vec2::new(600.0, 300.0);

fn frame(ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> DrawList {
    ui.begin_frame(SIZE);
    build(ui);
    ui.end_frame().clone()
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

// ---- I1: the reduced path grammar ----

#[test]
fn i1_implicit_repeats_and_signs() {
    // "l14 7-14 7": a sign starts a new number, and the pairs after an l are l's.
    let i = Icon::parse("M0 0l14 7-14 7z", 32.0).unwrap();
    assert_eq!(i.figures, vec![vec![v(0.0, 0.0), v(14.0, 7.0), v(0.0, 14.0)]]);
    // Extra pairs after an m are lines.
    let i = Icon::parse("m1 1 2 0 0 2z", 8.0).unwrap();
    assert_eq!(i.figures, vec![vec![v(1.0, 1.0), v(3.0, 1.0), v(3.0, 3.0)]]);
    // Separators: commas and spaces.
    let i = Icon::parse("M1,1,L5,1 5 5", 8.0).unwrap();
    assert_eq!(i.figures[0], vec![v(1.0, 1.0), v(5.0, 1.0), v(5.0, 5.0)]);
}

#[test]
fn i1_z_puts_the_pen_back_for_the_next_relative_move() {
    let i = Icon::parse("M5 5h2v2H5zm2 2h2v2H7z", 24.0).unwrap();
    assert_eq!(i.figures.len(), 2);
    assert_eq!(i.figures[0], vec![v(5.0, 5.0), v(7.0, 5.0), v(7.0, 7.0), v(5.0, 7.0)]);
    // The second square starts at (5, 5) + (2, 2).
    assert_eq!(i.figures[1][0], v(7.0, 7.0));
    assert_eq!(i.figures[1][2], v(9.0, 9.0));
}

#[test]
fn i1_the_preview_icon_parses_and_has_its_shape() {
    let i = Icon::parse("M4 20h2v2H2V6h2v14Z", 24.0).unwrap();
    assert_eq!(i.figures, vec![vec![v(4.0, 20.0), v(6.0, 20.0), v(6.0, 22.0), v(2.0, 22.0), v(2.0, 6.0), v(4.0, 6.0), v(4.0, 20.0)]]);
    // The z closes it: its own area is 2x16 + 2x2 + ... counted by the mask.
    let m = i.mask().unwrap();
    assert!(m[20 * 24 + 5] && m[21 * 24 + 2] && !m[10 * 24 + 5]);
}

#[test]
fn i1_an_unclosed_subpath_is_closed_when_filled() {
    let i = Icon::parse("M2 2h4v4H2", 8.0).unwrap();
    assert_eq!(i.mask().unwrap().iter().filter(|c| **c).count(), 16);
}

#[test]
fn i1_other_commands_are_an_error_naming_them() {
    let e = Icon::parse("M1 1c1 1 2 2 3 3z", 8.0).unwrap_err();
    assert!(e.contains('c'), "{e}");
    assert!(Icon::parse("h4", 8.0).is_err(), "a path starts with a move");
    assert!(Icon::parse("M1 1 L2", 8.0).is_err(), "a missing number");
}

// ---- I2: the launcher's own paths ----

const HOME: &str = "M4 20h16v2H4zm16-10h2v10h-2zM2 10h2v10H2zm2-2h2v2H4zm2-2h2v2H6zm2-2h2v2H8zm2-2h4v2h-4zm4 2h2v2h-2zm2 2h2v2h-2zm2 2h2v2h-2zM8 14h2v6H8zm2-2h4v2h-4zm4 2h2v6h-2z";
const PREVIEW: &str = "M4 20h2v2H2V6h2v14Zm12 0h2v2h-4v-2h-2v-2h2V8h-2V6h4v14Zm-8 0H6v-2h2v2Zm12 0h-2v-2h2v2ZM10 4h2v2h-2v10h2v2H8V4H6V2h4v2Zm12 14h-2V4h-2V2h4v16ZM6 6H4V4h2v2Zm12 0h-2V4h2v2Z";
const MINIMIZE: &str = "M5 11h14v2H5z";
const MAXIMIZE: &str = "M4 4h16v2H4zm0 14h16v2H4zM4 6h2v12H4zm14 0h2v12h-2z";
const RESTORE: &str = "M8 4h12v2H8zm10 2h2v8h-2zm-2 6h2v2h-2zM8 6h2v2H8zM4 8h12v2H4zm0 10h12v2H4zM4 10h2v8H4zm10 0h2v8h-2z";
const CLOSE: &str = "M5 5h2v2H5zm2 2h2v2H7zm2 2h2v2H9zm2 2h2v2h-2zm2-2h2v2h-2zm2-2h2v2h-2zm2-2h2v2h-2zM9 13h2v2H9zm-2 2h2v2H7zm-2 2h2v2H5zm8-4h2v2h-2zm2 2h2v2h-2zm2 2h2v2h-2z";
const VIEWS: [&str; 6] = [
    "M8 2h8v2H8zM6 4h12v8H6zM8 12h8v2H8zM11 14h2v6h-2zM5 20h14v2H5z",
    "M2 12h20v2H2zM2 16h20v2H2zM2 20h20v2H2zM8 6h8v2H8z",
    "M10 4h4v2h-4zM8 6h2v2H8zm6 0h2v2h-2zM6 8h2v4H6zm10 0h2v4h-2zM4 12h2v4H4zm14 0h2v4h-2zM2 16h2v4H2zm18 0h2v4h-2zM2 20h20v2H2z",
    "M10 2h4v2h-4zM8 4h2v10H8zm6 0h2v10h-2zM6 14h2v6H6zm10 0h2v6h-2zM8 20h8v2H8zM11 8h2v8h-2zM9 16h6v2H9z",
    "M4 2h4v2h2V2h4v2h2V2h4v6H4zM6 8h2v12H6zm10 0h2v12h-2zM10 14h4v6h-4zM2 20h20v2H2z",
    "M2 4h20v2H2zM2 18h20v2H2zM2 6h2v12H2zm18 0h2v12h-2zM4 12h4v2H4zm4-2h4v2H8zm4 2h4v2h-4zm4-4h4v2h-4z",
];
const PLAY: &str = "M15 11h-2V9h2zm0 4h-2v-2h2zm-2 2h-2v-2h2zm0-8h-2V7h2zm-2-2H9V5h2zM9 21H7V3h2zm6-8h2v-2h-2zm-6 4h2v2H9z";
const STOP: &str = "M6 6h12v12H6z";
const SETTINGS: &str = "M8 14H7v6H5v-6H2v-2h6v2Zm5 6h-2V10h2v10Zm9-2h-3v2h-2v-2h-1v-2h6v2Zm-3-4h-2V4h2v10ZM7 10H5V4h2v6Zm6-4h2v2H9V6h2V4h2v2Z";
const PACK: [&str; 3] = ["M16 2l14 7-14 7L2 9z", "M2 9l14 7v14L2 23z", "M30 9l-14 7v14l14-7z"];

fn area(d: &str) -> usize {
    Icon::parse(d, 24.0).unwrap().mask().expect("a pixel icon").iter().filter(|c| **c).count()
}

#[test]
fn i2_every_launcher_path_parses_and_the_pixel_ones_are_axis_aligned() {
    let mut all = vec![HOME, PREVIEW, MINIMIZE, MAXIMIZE, RESTORE, CLOSE, PLAY, STOP, SETTINGS];
    all.extend(VIEWS);
    for d in all {
        let i = Icon::parse(d, 24.0).unwrap_or_else(|e| panic!("{d}: {e}"));
        assert!(i.axis_aligned(), "{d}");
        assert!(i.mask().is_some(), "{d}");
        assert!(!i.figures.is_empty());
    }
    for d in PACK {
        let i = Icon::parse(d, 32.0).unwrap();
        assert!(!i.axis_aligned(), "{d} has slopes");
        assert!(i.mask().is_none());
        assert_eq!(i.figures.len(), 1);
    }
}

#[test]
fn i2_mask_areas_are_the_hand_counted_ones() {
    assert_eq!(area(MINIMIZE), 14 * 2);
    assert_eq!(area(STOP), 12 * 12);
    // The close sign is 13 squares of 2 x 2 (the brief counted 17; counting the path's m commands gives 13).
    assert_eq!(area(CLOSE), 13 * 4);
    // The maximise sign is a frame: a 16 x 16 box less its 12 x 12 inside, 2 thick all round.
    assert_eq!(area(MAXIMIZE), 16 * 16 - 12 * 12);
}

// ---- K1: cursors ----

fn centre(ui: &Ui, id: nui_ui::Id) -> Vec2 {
    let r = ui.rect(id).expect("built once");
    v(r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn build(ui: &mut Ui, text: &mut String) -> (nui_ui::Id, nui_ui::Id, nui_ui::Id) {
    let mut ids = (None, None, None);
    frame(ui, |ui| {
        ui.node("row", Default::default(), |ui| {
            ids.0 = Some(ui.button("go", "Go", ButtonLook::Plain).id);
            ids.1 = Some(ui.button_with("off", "Off", ButtonLook::Plain, ButtonOpts { disabled: true, grow: 0.0 }).id);
            ids.2 = Some(ui.text_field("name", text, FieldOpts::default()).resp.id);
        });
    });
    (ids.0.unwrap(), ids.1.unwrap(), ids.2.unwrap())
}

#[test]
fn k1_a_button_is_a_hand_a_disabled_one_and_nothing_an_arrow_a_field_an_ibeam() {
    let mut ui = Ui::new(Theme::dark());
    let mut text = String::from("abc");
    let (go, off, name) = build(&mut ui, &mut text);
    assert_eq!(ui.cursor(), Cursor::Arrow);
    for (id, want) in [(go, Cursor::Hand), (off, Cursor::Arrow), (name, Cursor::IBeam)] {
        let p = centre(&ui, id);
        ui.event(&Event::PointerMoved { pos: p });
        build(&mut ui, &mut text);
        assert_eq!(ui.cursor(), want, "over {id:?}");
    }
    // Nothing under the pointer.
    ui.event(&Event::PointerMoved { pos: v(590.0, 290.0) });
    build(&mut ui, &mut text);
    assert_eq!(ui.cursor(), Cursor::Arrow);
    // A held button keeps its hand when the pointer is dragged off it.
    let p = centre(&ui, go);
    ui.event(&Event::PointerMoved { pos: p });
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    build(&mut ui, &mut text);
    ui.event(&Event::PointerMoved { pos: v(590.0, 290.0) });
    build(&mut ui, &mut text);
    assert_eq!(ui.cursor(), Cursor::Hand);
    // ...and over another widget the held one still decides.
    let q = centre(&ui, name);
    ui.event(&Event::PointerMoved { pos: q });
    build(&mut ui, &mut text);
    assert_eq!(ui.cursor(), Cursor::Hand);
    ui.event(&Event::PointerUp { pos: q, button: Button::Left });
    build(&mut ui, &mut text);
    // A canvas may ask for its own shape.
    ui.begin_frame(SIZE);
    ui.want_cursor(Cursor::Crosshair);
    ui.end_frame();
    assert_eq!(ui.cursor(), Cursor::Crosshair);
}

// ---- K2: the window's edges ----

#[test]
fn k2_the_grips_of_a_1400_by_940_window() {
    let s = v(1400.0, 940.0);
    let cases = [
        ((1.0, 1.0), Some(Edge::NW)),
        ((7.0, 7.0), Some(Edge::NW)),
        ((9.0, 3.0), Some(Edge::N)),
        ((700.0, 2.0), Some(Edge::N)),
        ((2.0, 470.0), Some(Edge::W)),
        ((1398.0, 470.0), Some(Edge::E)),
        ((700.0, 938.0), Some(Edge::S)),
        ((1392.0, 932.0), Some(Edge::SE)),
        ((1398.0, 1.0), Some(Edge::NE)),
        ((1.0, 938.0), Some(Edge::SW)),
        ((1395.0, 20.0), Some(Edge::E)),
        ((1380.0, 2.0), Some(Edge::N)),
        ((1380.0, 20.0), None),
        ((700.0, 500.0), None),
    ];
    for ((x, y), want) in cases {
        assert_eq!(grip(v(x, y), s, false), want, "({x}, {y})");
        // A maximised window has no edge to pull.
        assert_eq!(grip(v(x, y), s, true), None);
    }
    assert_eq!(Hit::Resize(Edge::NE).cursor(), Cursor::ResizeNESW);
}

// ---- T1: tooltips ----

fn tip_text(l: &DrawList) -> Option<String> {
    // The tip is the last thing drawn: a fill, a frame, then its lines of text.
    let n = l.cmds.len();
    let tail = &l.cmds[..n];
    let frame_at = tail.iter().rposition(|c| matches!(c, Cmd::Frame { .. }))?;
    // Only a tip has text after the last frame.
    let words: Vec<String> = tail[frame_at + 1..].iter().filter_map(|c| if let Cmd::Text { text, .. } = c { Some(text.clone()) } else { None }).collect();
    (!words.is_empty()).then(|| words.join(" "))
}

fn tipped(ui: &mut Ui, clock: f64) -> (nui_ui::Id, nui_ui::Id, DrawList) {
    ui.set_clock(clock);
    let mut ids = (None, None);
    let list = frame(ui, |ui| {
        ui.node("row", Default::default(), |ui| {
            let a = ui.button("go", "Go", ButtonLook::Plain).id;
            ui.title(a, "Starts the run");
            let b = ui.button_with("off", "Off", ButtonLook::Plain, ButtonOpts { disabled: true, grow: 0.0 }).id;
            ui.title(b, "Needs the full detail");
            ids = (Some(a), Some(b));
        });
    });
    (ids.0.unwrap(), ids.1.unwrap(), list)
}

#[test]
fn t1_a_tip_shows_after_its_delay_and_a_press_hides_it() {
    let mut ui = Ui::new(Theme::dark());
    let (go, off, _) = tipped(&mut ui, 0.0);
    let p = centre(&ui, go);
    ui.event(&Event::PointerMoved { pos: p });
    // The first frame with the pointer there learns the candidate: its time starts now.
    tipped(&mut ui, 0.0);
    let (_, _, l) = tipped(&mut ui, 0.4);
    assert_eq!(tip_text(&l), None, "0.4 s is too early");
    let wake = ui.wake_in().expect("a tip waits");
    assert!((wake - 0.1).abs() < 1e-4, "{wake}");
    let (_, _, l) = tipped(&mut ui, 0.6);
    assert_eq!(tip_text(&l).as_deref(), Some("Starts the run"));
    assert_eq!(ui.wake_in(), None, "shown: nothing more to wait for");
    // A press hides it, and it stays hidden while the pointer stays.
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    let (_, _, l) = tipped(&mut ui, 0.7);
    assert_eq!(tip_text(&l), None);
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    let (_, _, l) = tipped(&mut ui, 5.0);
    assert_eq!(tip_text(&l), None);
    // Another widget: a new count, and a disabled button shows its title as the browser does.
    let q = centre(&ui, off);
    ui.event(&Event::PointerMoved { pos: q });
    tipped(&mut ui, 6.0);
    let (_, _, l) = tipped(&mut ui, 6.0 + TIP_DELAY + 0.01);
    assert_eq!(tip_text(&l).as_deref(), Some("Needs the full detail"));
    // Leaving the window removes it.
    ui.event(&Event::PointerLeft);
    let (_, _, l) = tipped(&mut ui, 8.0);
    assert_eq!(tip_text(&l), None);
}
