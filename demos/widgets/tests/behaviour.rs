//! What the widgets do, checked in pixels. The cards show every state as a picture; these tests drive a widget
//! in a `Ui` with the events a person would send (pointer, keys, characters, the input method) and read the
//! frame the renderer draws, so that a colour or a caret that goes wrong fails here. Positions come from the
//! widget's own rect and the kit's constants; colours from the theme.

use mere_base::{Button, Cmd, Color, DrawList, Event, Gfx, Ime, Key, Mods, Rect, Vec2};
use mere_layout::{Dim, Insets, Position, Style};
use mere_ui::tokens::{FADE_DIM, FADE_LIT, FRAME, INSET};
use mere_ui::{SWITCH_H, SWITCH_W};
use mere_ui::{
    ButtonLook, ButtonOpts, FieldOpts, GUTTER, Icon, IconLook, IconOpts, Id, ListRow, NumberSpec, RowOpts, ScrollOpts, THUMB_W, Theme, Ui,
    scroll_style, text_width,
};
use mere_win32::Renderer;

const SIZE: (u32, u32) = (400, 300);
const NONE: Mods = Mods { ctrl: false, shift: false };
const CTRL: Mods = Mods { ctrl: true, shift: false };
const SHIFT: Mods = Mods { ctrl: false, shift: true };

fn both(check: fn(Theme)) {
    check(Theme::dark());
    check(Theme::light());
}

fn at(x: f32, y: f32) -> Style {
    Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(x), top: Dim::Px(y), right: Dim::Auto, bottom: Dim::Auto },
        ..Style::default()
    }
}

/// A frame as pixels.
struct Pic {
    rgba: Vec<u8>,
}

impl Pic {
    fn of(theme: Theme, list: &DrawList) -> Pic {
        let mut r = Renderer::offscreen(SIZE.0, SIZE.1, 1.0).expect("offscreen renderer");
        r.begin(theme.ground);
        r.draw_list(list);
        r.end();
        Pic { rgba: r.read_rgba().unwrap() }
    }

    fn at(&self, x: f32, y: f32) -> [u8; 3] {
        let i = (y as usize * SIZE.0 as usize + x as usize) * 4;
        [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]]
    }

    fn is(&self, x: f32, y: f32, want: Color, what: &str) {
        let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
        let got = self.at(x, y);
        assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what} at ({x}, {y}): got {got:?}, want {w:?}");
    }
}

/// One `Ui` driven by events, the way the backend drives an application.
struct Scene {
    ui: Ui,
    theme: Theme,
}

impl Scene {
    fn new(theme: Theme) -> Scene {
        Scene { ui: Ui::new(theme), theme }
    }

    fn frame(&mut self, build: impl FnOnce(&mut Ui)) -> DrawList {
        self.ui.begin_frame(Vec2::new(SIZE.0 as f32, SIZE.1 as f32));
        build(&mut self.ui);
        self.ui.end_frame().clone()
    }

    fn pic(&self, list: &DrawList) -> Pic {
        Pic::of(self.theme, list)
    }

    fn rect(&self, id: Id) -> Rect {
        self.ui.rect(id).expect("the widget was laid out")
    }

    fn pointer(&mut self, p: Vec2) {
        self.ui.event(&Event::PointerMoved { pos: p });
    }

    fn click(&mut self, p: Vec2, clicks: u8) {
        self.pointer(p);
        self.ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks });
        self.ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    }

    fn key(&mut self, key: Key, mods: Mods, n: usize) {
        for _ in 0..n {
            self.ui.event(&Event::Key { key, mods });
        }
    }

    fn ime(&mut self, i: Ime) {
        self.ui.event(&Event::Ime(i));
    }
}

fn centre(r: Rect) -> Vec2 {
    Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
}

// ---- buttons, toggle, icon buttons ----

/// One primary button alone, at (20, 20) and 180 wide, with the pointer put as asked.
fn primary(theme: Theme, disabled: bool, pointer: bool, held: bool) -> (Pic, Rect) {
    let mut sc = Scene::new(theme);
    let build = |ui: &mut Ui| {
        let w = Style { w: Dim::Px(180.0), ..at(20.0, 20.0) };
        let mut id = None;
        ui.node("go", w, |ui| id = Some(ui.button_with("generate", "Generate", ButtonLook::Primary, ButtonOpts { disabled, grow: 1.0 }).id));
        id.unwrap()
    };
    let id = {
        let mut id = None;
        sc.frame(|ui| id = Some(build(ui)));
        id.unwrap()
    };
    let r = sc.rect(id);
    if pointer {
        sc.pointer(centre(r));
        if held {
            sc.ui.event(&Event::PointerDown { pos: centre(r), button: Button::Left, clicks: 1 });
        }
    }
    let list = sc.frame(|ui| {
        build(ui);
    });
    (sc.pic(&list), r)
}

#[test]
fn the_primary_button_is_filled_lit_under_the_pointer_and_set_back_when_disabled() {
    both(|t| {
        // A point inside the box, left of the word.
        let (x, y) = (|r: Rect| (r.x + 3.0, r.y + r.h / 2.0))(primary(t, false, false, false).1);
        let (rest, _) = primary(t, false, false, false);
        rest.is(x, y, t.fill, "primary rest fill");
        let (hover, _) = primary(t, false, true, false);
        hover.is(x, y, t.faded(t.fill, FADE_LIT), "primary hover");
        // Held is the hover look: the two boxes are the same picture.
        let (held, r) = primary(t, false, true, true);
        for py in r.y as usize..(r.y + r.h) as usize {
            for px in r.x as usize..(r.x + r.w) as usize {
                let (a, b) = (hover.at(px as f32, py as f32), held.at(px as f32, py as f32));
                assert!(a.iter().zip(b).all(|(p, q)| (*p as i32 - q as i32).abs() <= 2), "held differs from hover at {px},{py}");
            }
        }
        // Disabled: no fill, a frame in the edge colour, and the pointer on it changes nothing.
        for pointer in [false, true] {
            let (off, r) = primary(t, true, pointer, false);
            off.is(x, y, t.ground, "primary disabled has no fill");
            off.is(r.x, y, t.edge, "primary disabled frame");
        }
    });
}

fn toggle(theme: Theme, on: bool, pointer: bool) -> (Pic, Rect) {
    let mut sc = Scene::new(theme);
    let mut on = on;
    let mut id = None;
    sc.frame(|ui| {
        ui.node("t", at(20.0, 20.0), |ui| id = Some(ui.toggle("field", &mut on, "on", "off").id));
    });
    let r = sc.rect(id.unwrap());
    if pointer {
        sc.pointer(centre(r));
    }
    let list = sc.frame(|ui| {
        ui.node("t", at(20.0, 20.0), |ui| {
            ui.toggle("field", &mut on, "on", "off");
        });
    });
    (sc.pic(&list), r)
}

#[test]
fn the_off_switch_frame_takes_the_text_colour_under_the_pointer() {
    both(|t| {
        let (rest, r) = toggle(t, false, false);
        // The frame's top line, away from the corners; the switch is the box's left part, centred in its height.
        let (x, top) = (r.x + SWITCH_W / 2.0, r.y + (r.h - SWITCH_H) / 2.0);
        rest.is(x, top, t.edge, "switch frame at rest");
        rest.is(x, top + SWITCH_H / 2.0, t.ground, "an off switch is hollow");
        let (hover, _) = toggle(t, false, true);
        hover.is(x, top, t.text, "switch frame on hover");
    });
}

#[test]
fn the_on_switch_is_filled_with_a_square_knob_on_the_right() {
    both(|t| {
        let (pic, r) = toggle(t, true, false);
        let top = r.y + (r.h - SWITCH_H) / 2.0;
        let mid = top + SWITCH_H / 2.0;
        pic.is(r.x + 2.0, mid, t.fill, "the fill, left of the knob");
        pic.is(r.x + SWITCH_W - 8.0, mid, t.on_fill, "the knob's body, right");
        pic.is(r.x + 8.0, mid, t.fill, "no knob on the left");
    });
}
const PLAY: &str = "M15 11h-2V9h2zm0 4h-2v-2h2zm-2 2h-2v-2h2zm0-8h-2V7h2zm-2-2H9V5h2zM9 21H7V3h2zm6-8h2v-2h-2zm-6 4h2v2H9z";

fn icon_button(theme: Theme, look: IconLook, pressed: bool, pointer: bool) -> (Pic, Rect) {
    let mut sc = Scene::new(theme);
    let icon = Icon::parse(PLAY, 24.0).unwrap();
    let opts = IconOpts { pressed, disabled: false, joined: false, marker: None };
    let mut id = None;
    sc.frame(|ui| {
        ui.node("i", at(20.0, 20.0), |ui| id = Some(ui.icon_button("b", &icon, look, opts).id));
    });
    let r = sc.rect(id.unwrap());
    if pointer {
        sc.pointer(centre(r));
    }
    let list = sc.frame(|ui| {
        ui.node("i", at(20.0, 20.0), |ui| {
            ui.icon_button("b", &icon, look, opts);
        });
    });
    (sc.pic(&list), r)
}

#[test]
fn the_icon_buttons_frame_fill_and_lighten_as_the_design_says() {
    both(|t| {
        // Square: framed in the edge, the frame and the icon take the text colour under the pointer, and a
        // pressed one is filled in reverse.
        let (rest, r) = icon_button(t, IconLook::Square, false, false);
        rest.is(r.x + r.w / 2.0, r.y, t.edge, "square rest frame");
        let (hover, _) = icon_button(t, IconLook::Square, false, true);
        hover.is(r.x + r.w / 2.0, r.y, t.text, "square hover frame");
        let (pressed, _) = icon_button(t, IconLook::Square, true, false);
        pressed.is(r.x + 3.0, r.y + 3.0, t.fill, "square pressed fill");
        // Filled: the fill at rest, lit under the pointer.
        let (rest, r) = icon_button(t, IconLook::Filled, false, false);
        rest.is(r.x + 3.0, r.y + 3.0, t.fill, "filled rest");
        let (hover, _) = icon_button(t, IconLook::Filled, false, true);
        hover.is(r.x + 3.0, r.y + 3.0, t.faded(t.fill, FADE_LIT), "filled hover");
        // Caption: no frame and no fill at rest.
        let (rest, r) = icon_button(t, IconLook::Caption, false, false);
        rest.is(r.x, r.y, t.ground, "caption rest has no frame");
    });
}

// ---- fields ----

fn field_at(ui: &mut Ui, s: &mut String, opts: FieldOpts) -> Id {
    let mut id = None;
    ui.node("f", at(20.0, 20.0), |ui| id = Some(ui.text_field("seed", s, FieldOpts { width: Dim::Px(240.0), ..opts }).resp.id));
    id.unwrap()
}

/// A field of ten spaces (no ink to confuse a caret or a selection), clicked into focus.
fn spaces(theme: Theme) -> (Scene, String, Rect) {
    let mut sc = Scene::new(theme);
    let mut s = " ".repeat(10);
    let mut id = None;
    sc.frame(|ui| id = Some(field_at(ui, &mut s, FieldOpts::default())));
    let r = sc.rect(id.unwrap());
    // The content box starts FRAME + INSET in; the click is on the field's far right, so the caret goes to the end.
    sc.click(Vec2::new(r.x + r.w - 2.0, r.y + r.h / 2.0), 1);
    (sc, s, r)
}

/// The x of the content box's column `col`.
fn col_x(theme: Theme, r: Rect, col: usize) -> f32 {
    r.x + FRAME + INSET + text_width(&" ".repeat(col), theme.body)
}

#[test]
fn a_focused_field_frames_in_the_text_colour_and_shows_its_caret_where_the_keys_put_it() {
    both(|t| {
        let (mut sc, mut s, r) = spaces(t);
        let mid = r.y + r.h / 2.0;
        let list = sc.frame(|ui| {
            field_at(ui, &mut s, FieldOpts::default());
        });
        sc.pic(&list).is(r.x + r.w / 2.0, r.y, t.text, "focused frame");
        // Home then Right x4: the caret stands at column 4.
        sc.key(Key::Home, NONE, 1);
        sc.key(Key::Right, NONE, 4);
        let list = sc.frame(|ui| {
            field_at(ui, &mut s, FieldOpts::default());
        });
        let pic = sc.pic(&list);
        let x = col_x(t, r, 4).round();
        pic.is(x, mid, t.text, "caret at column 4");
        pic.is(col_x(t, r, 2).round(), mid, t.ground, "no caret at column 2");
        // Shift+Right x4 from column 4: the caret hides and the selection covers columns 4..8.
        sc.key(Key::Right, SHIFT, 4);
        let list = sc.frame(|ui| {
            field_at(ui, &mut s, FieldOpts::default());
        });
        let pic = sc.pic(&list);
        pic.is(col_x(t, r, 6), mid, t.select, "selection inside columns 4..8");
        pic.is(col_x(t, r, 2), mid, t.ground, "no selection before column 4");
        // Escape leaves the field: the frame goes back to the hairline.
        sc.key(Key::Escape, NONE, 1);
        let list = sc.frame(|ui| {
            field_at(ui, &mut s, FieldOpts::default());
        });
        sc.pic(&list).is(r.x + r.w / 2.0, r.y, t.edge, "blurred frame");
    });
}

#[test]
fn a_long_text_scrolls_so_that_the_caret_stays_in_the_content_box() {
    both(|t| {
        let mut sc = Scene::new(t);
        let mut s = String::new();
        let mut id = None;
        sc.frame(|ui| id = Some(field_at(ui, &mut s, FieldOpts::default())));
        let r = sc.rect(id.unwrap());
        sc.click(centre(r), 1);
        for ch in "the quick brown fox jumps over the lazy dog".chars() {
            sc.ui.event(&Event::Char { ch });
        }
        let list = sc.frame(|ui| {
            field_at(ui, &mut s, FieldOpts::default());
        });
        let pic = sc.pic(&list);
        // The caret is the last thing on the line: the last text-coloured column inside the box is at its right
        // edge, and nothing is drawn right of the frame.
        let mid = r.y + r.h / 2.0;
        let right = r.x + r.w - FRAME - INSET;
        let lit = (right as usize - 2..=right as usize + 1).any(|x| pic.at(x as f32, mid) == [(t.text.r * 255.0).round() as u8, (t.text.g * 255.0).round() as u8, (t.text.b * 255.0).round() as u8]);
        assert!(lit, "the caret is not at the right edge of the content box");
        pic.is(r.x + r.w + 2.0, mid, t.ground, "nothing right of the frame");
    });
}

fn number_row(ui: &mut Ui, v: &mut f64) -> Id {
    let mut id = None;
    let spec = NumberSpec { min: 0.5, max: 3.0, step: 0.025 };
    ui.node("n", at(20.0, 100.0), |ui| id = Some(ui.number("field", v, spec).resp.id));
    id.unwrap()
}

#[test]
fn a_number_field_steps_with_the_arrows_and_clamps_on_enter() {
    both(|t| {
        let mut sc = Scene::new(t);
        let mut v = 1.0;
        let mut id = None;
        sc.frame(|ui| id = Some(number_row(ui, &mut v)));
        let r = sc.rect(id.unwrap());
        sc.click(centre(r), 1);
        sc.key(Key::Up, NONE, 1);
        sc.frame(|ui| {
            number_row(ui, &mut v);
        });
        sc.key(Key::Up, NONE, 1);
        let list = sc.frame(|ui| {
            number_row(ui, &mut v);
        });
        assert!((v - 1.05).abs() < 1e-9, "two steps up from 1 give {v}");
        sc.pic(&list).is(r.x + r.w / 2.0, r.y, t.text, "focused number frame");
        // 9 typed over everything, then Enter: clamped to the maximum, and the field leaves the keyboard.
        sc.key(Key::A, CTRL, 1);
        sc.ui.event(&Event::Char { ch: '9' });
        sc.frame(|ui| {
            number_row(ui, &mut v);
        });
        sc.key(Key::Enter, NONE, 1);
        let list = sc.frame(|ui| {
            number_row(ui, &mut v);
        });
        assert_eq!(v, 3.0);
        sc.pic(&list).is(r.x + r.w / 2.0, r.y, t.edge, "blurred number frame");
    });
}

// ---- the input method ----

/// The fills one pixel high under the field's text: a composition's underline.
fn underlines(list: &DrawList, r: Rect) -> Vec<Rect> {
    list.cmds
        .iter()
        .filter_map(|c| match c {
            Cmd::Fill { rect, .. } if rect.h == 1.0 && rect.x > r.x + FRAME && rect.x + rect.w < r.x + r.w && rect.y > r.y + FRAME && rect.y < r.y + r.h - FRAME => Some(*rect),
            _ => None,
        })
        .collect()
}

#[test]
fn a_composition_is_underlined_and_a_commit_or_a_cancel_removes_the_underline() {
    both(|t| {
        let mut sc = Scene::new(t);
        let mut s = "world ".to_string();
        let mut id = None;
        sc.frame(|ui| id = Some(field_at(ui, &mut s, FieldOpts::default())));
        let r = sc.rect(id.unwrap());
        sc.click(Vec2::new(r.x + r.w - 2.0, r.y + r.h / 2.0), 1);
        let build = |sc: &mut Scene, s: &mut String| {
            sc.frame(|ui| {
                field_at(ui, s, FieldOpts::default());
            })
        };
        build(&mut sc, &mut s);
        // Composing: the composition starts where the text ends, is as wide as its cells and is underlined in the text colour.
        sc.ime(Ime::Start);
        sc.ime(Ime::Update { text: "にほんご".into(), cursor: 4 });
        let list = build(&mut sc, &mut s);
        let start = r.x + FRAME + INSET + text_width("world ", t.body);
        let lines = underlines(&list, r);
        assert_eq!(lines.len(), 1, "one underline under the composition: {lines:?}");
        assert!((lines[0].x - start).abs() < 0.6, "the underline starts at the end of the text");
        assert!(lines[0].w > text_width("にほんご", t.body) - 1.0, "the underline spans the composition");
        let pic = sc.pic(&list);
        pic.is((lines[0].x + 4.0).round(), lines[0].y, t.text, "underline pixel");
        assert!(sc.ui.ime_spot().is_some(), "the input method follows the caret of the focused field");
        assert_eq!(s, "world ", "the text is not the field's value until it is committed");

        // Committed: the text joins the value and the underline goes.
        sc.ime(Ime::Update { text: "日本語".into(), cursor: 3 });
        sc.ime(Ime::Commit { text: "日本語".into() });
        sc.ime(Ime::End);
        let list = build(&mut sc, &mut s);
        assert_eq!(s, "world 日本語");
        assert!(underlines(&list, r).is_empty(), "a commit leaves no underline");

        // Cancelled: an empty update and the end leave the value as it was.
        sc.ime(Ime::Start);
        sc.ime(Ime::Update { text: "にほ".into(), cursor: 2 });
        build(&mut sc, &mut s);
        sc.ime(Ime::Update { text: String::new(), cursor: 0 });
        sc.ime(Ime::End);
        let list = build(&mut sc, &mut s);
        assert_eq!(s, "world 日本語");
        assert!(underlines(&list, r).is_empty(), "a cancel leaves no underline");
    });
}

#[test]
fn a_number_field_takes_no_composition() {
    both(|t| {
        let mut sc = Scene::new(t);
        let mut v = 1.0;
        let mut id = None;
        sc.frame(|ui| id = Some(number_row(ui, &mut v)));
        let r = sc.rect(id.unwrap());
        sc.click(centre(r), 1);
        sc.frame(|ui| {
            number_row(ui, &mut v);
        });
        sc.ime(Ime::Start);
        sc.ime(Ime::Update { text: "に".into(), cursor: 1 });
        sc.ime(Ime::Commit { text: "に".into() });
        sc.ime(Ime::End);
        let list = sc.frame(|ui| {
            number_row(ui, &mut v);
        });
        assert!(sc.ui.ime_spot().is_none(), "the input method is off over a number field");
        assert_eq!(v, 1.0, "the number is unchanged");
        assert!(underlines(&list, r).is_empty());
        sc.pic(&list).is(r.x + r.w / 2.0, r.y, t.text, "the number field keeps the keyboard");
    });
}

// ---- the scroll area and the list ----

const ROWS: usize = 20;
const BOX_H: f32 = 200.0;
const COL_W: f32 = 300.0;

/// A scroll box of `ROWS` list rows, choosing, with `pressed` rows chosen. Answers the box's id, its range and
/// the rows' ids.
fn list_scene(ui: &mut Ui, y: &mut f32, pressed: &[usize]) -> (Id, f32, Vec<Id>) {
    let mut out = None;
    ui.node("col", Style { column: true, w: Dim::Px(COL_W), ..at(20.0, 20.0) }, |ui| {
        let style = Style { h: Dim::Px(BOX_H), grow: 0.0, basis: Dim::Auto, ..scroll_style() };
        let mut rows = Vec::new();
        let r = ui.scroll("list", y, style, ScrollOpts::LIST, |ui| {
            for i in 0..ROWS {
                let row = ListRow { swatch: swatch(i), name: "Biome", width: "350 blocks", share: "1.0%" };
                rows.push(ui.list_row(&format!("row{i}"), &row, RowOpts { pressed: pressed.contains(&i), choosing: true, ..RowOpts::default() }).id);
            }
        });
        out = Some((r.id, r.max, rows));
    });
    out.unwrap()
}

fn swatch(i: usize) -> Color {
    Color::hex(0x3a6ea5 + i as u32 * 0x040404)
}

/// The box and its rows after the gutter has come (the second frame) and the offset is `at` of the range.
fn settled(sc: &mut Scene, y: &mut f32, pressed: &[usize], at: f32) -> (Id, f32, Vec<Id>) {
    sc.frame(|ui| {
        list_scene(ui, y, pressed);
    });
    let mut got = None;
    sc.frame(|ui| got = Some(list_scene(ui, y, pressed)));
    let (id, max, rows) = got.unwrap();
    *y = (max * at).round();
    (id, max, rows)
}

#[test]
fn the_scrollbar_has_a_thin_track_a_thumb_that_follows_the_offset_and_brightens_under_the_pointer() {
    both(|t| {
        let mut sc = Scene::new(t);
        let mut y = 0.0f32;
        let (id, max, rows) = settled(&mut sc, &mut y, &[], 0.5);
        assert!(max > 0.0, "twenty rows overflow a box of {BOX_H}");
        let _ = rows;
        let list = sc.frame(|ui| {
            list_scene(ui, &mut y, &[]);
        });
        let view = sc.rect(id);
        let pic = sc.pic(&list);
        let gx = view.x + view.w - GUTTER;
        // The thumb is a run of THUMB_W columns of the muted colour at the track's middle; find it there.
        let mid = view.y + view.h / 2.0;
        let is = |p: &Pic, px: f32, py: f32, c: Color| {
            let w = [c.r, c.g, c.b].map(|v| (v * 255.0).round() as i32);
            p.at(px, py).iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1)
        };
        let muted: Vec<f32> = (0..GUTTER as usize).map(|i| gx + i as f32).filter(|px| is(&pic, *px, mid, t.muted)).collect();
        assert_eq!(muted.len(), THUMB_W as usize, "the thumb is {THUMB_W} px of the muted colour: {muted:?}");
        assert!(muted.windows(2).all(|w| w[1] - w[0] == 1.0), "the thumb is one run of columns");
        // The 1 px track runs in the thumb's middle column: the line colour above the thumb, ground beside it.
        let track_x = muted[muted.len() / 2];
        pic.is(track_x, view.y + mere_ui::ARROW + 4.0, t.line, "track above the thumb");
        pic.is(gx, view.y + mere_ui::ARROW + 4.0, t.ground, "gutter left of the track");
        // The pointer on the thumb: the thumb takes the text colour.
        sc.pointer(Vec2::new(muted[0], mid));
        sc.frame(|ui| {
            list_scene(ui, &mut y, &[]);
        });
        let list = sc.frame(|ui| {
            list_scene(ui, &mut y, &[]);
        });
        let pic = sc.pic(&list);
        assert!((0..THUMB_W as usize).all(|i| is(&pic, muted[0] + i as f32, mid, t.text)), "the thumb is not in the text colour under the pointer");
    });
}

#[test]
fn a_list_without_overflow_has_no_scrollbar() {
    both(|t| {
        let mut sc = Scene::new(t);
        let mut y = 0.0f32;
        let mut got = None;
        for _ in 0..2 {
            sc.frame(|ui| {
                ui.node("col", Style { column: true, w: Dim::Px(COL_W), ..at(20.0, 20.0) }, |ui| {
                    let style = Style { h: Dim::Px(BOX_H), grow: 0.0, basis: Dim::Auto, ..scroll_style() };
                    got = Some(ui.scroll("list", &mut y, style, ScrollOpts::LIST, |ui| ui.label("Item", mere_ui::Tone::Text)));
                });
            });
        }
        let r = got.unwrap();
        assert!(!r.overflow && r.max == 0.0);
        let list = sc.frame(|ui| {
            ui.node("col", Style { column: true, w: Dim::Px(COL_W), ..at(20.0, 20.0) }, |ui| {
                let style = Style { h: Dim::Px(BOX_H), grow: 0.0, basis: Dim::Auto, ..scroll_style() };
                ui.scroll("list", &mut y, style, ScrollOpts::LIST, |ui| ui.label("Item", mere_ui::Tone::Text));
            });
        });
        let view = sc.rect(r.id);
        let pic = sc.pic(&list);
        for dx in 0..GUTTER as usize {
            pic.is(view.x + view.w - GUTTER + dx as f32, view.y + view.h / 2.0, t.ground, "gutter of a list that fits");
        }
    });
}

#[test]
fn a_chosen_row_keeps_its_swatch_the_others_are_set_back_and_the_hovered_one_is_lit_and_underlined() {
    both(|t| {
        let mut sc = Scene::new(t);
        let mut y = 0.0f32;
        let pressed = [5];
        let (_, _, rows) = settled(&mut sc, &mut y, &pressed, 0.0);
        let rects: Vec<Rect> = rows.iter().take(8).map(|id| sc.rect(*id)).collect();
        // Hover row 3 and read the frame after it.
        sc.pointer(centre(rects[3]));
        sc.frame(|ui| {
            list_scene(ui, &mut y, &pressed);
        });
        let list = sc.frame(|ui| {
            list_scene(ui, &mut y, &pressed);
        });
        let pic = sc.pic(&list);
        // The swatch is SWATCH (12) wide and centred in the row's height; sample its middle.
        let probe = |r: Rect| (r.x + mere_ui::tokens::SPACE_4 + mere_ui::tokens::SWATCH / 2.0, r.y + r.h / 2.0);
        let (px, py) = probe(rects[5]);
        pic.is(px, py, swatch(5), "the chosen row's swatch is at full strength");
        let (px, py) = probe(rects[1]);
        pic.is(px, py, t.faded(swatch(1), FADE_DIM), "an unchosen row's swatch is set back");
        let (px, py) = probe(rects[3]);
        pic.is(px, py, t.faded(swatch(3), FADE_LIT), "the hovered row's swatch is lit");
        // Its name is underlined with a 1 px line in the lit text colour; no other row has one.
        let lit = t.faded(t.text, FADE_LIT);
        let lines_in = |r: Rect| list.cmds.iter().filter(|c| matches!(c, Cmd::Fill { rect, color } if rect.h == 1.0 && *color == lit && rect.y >= r.y && rect.y < r.y + r.h)).count();
        assert_eq!(lines_in(rects[3]), 1, "the hovered row underlines its name");
        assert_eq!(lines_in(rects[1]), 0, "a row under no pointer has no underline");
    });
}
