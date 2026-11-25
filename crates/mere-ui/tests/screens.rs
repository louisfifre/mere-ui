//! The pieces the launcher's screens are built from: a section whose head carries actions and does not fold, the
//! list line with its fixed slots, and a page's title line. Numbers come from the boards and the kit's tokens.

use mere_base::{Button, Cmd, DrawList, Event, Rect, Vec2};
use mere_layout::{Dim, Style};
use mere_ui::tokens::{GAP, ICON_SQUARE};
use mere_ui::{ActionLook, Icon, LINE_ACTION_W, LINE_END_W, LINE_NEW_W, LineAction, LineEnd, ListLine, ListLineResp, Theme, Tone, Ui};

const SIZE: Vec2 = Vec2::new(520.0, 400.0);
const COL: f32 = 480.0;
const ICON: &str = "M9 2h6v2h5v2H4V4h5zM5 8h2v12h10V8h2v14H5zm4 2h2v8H9zm4 0h2v8h-2z";

fn icon() -> Icon {
    Icon::parse(ICON, 24.0).unwrap()
}

fn column(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
    ui.node("col", Style { column: true, w: Dim::Px(COL), ..Style::default() }, body);
}

fn texts(l: &DrawList) -> Vec<(Vec2, String)> {
    l.cmds.iter().filter_map(|c| if let Cmd::Text { at, text, .. } = c { Some((*at, text.clone())) } else { None }).collect()
}

/// Where a text starts, whole or drawn glyph by glyph (a look with tracking draws each glyph apart).
fn text_at(l: &DrawList, s: &str) -> Vec2 {
    let all = texts(l);
    if let Some(t) = all.iter().find(|t| t.1 == s) {
        return t.0;
    }
    let flat: Vec<(Vec2, char)> = all.iter().flat_map(|t| t.1.chars().map(move |c| (t.0, c))).collect();
    let want: Vec<char> = s.chars().collect();
    flat.windows(want.len()).find(|w| w.iter().map(|g| g.1).eq(want.iter().copied())).map(|w| w[0].0).unwrap_or_else(|| panic!("no text {s:?} in {all:?}"))
}

fn said(l: &DrawList, s: &str) -> bool {
    let flat: String = texts(l).iter().flat_map(|t| t.1.chars()).collect();
    flat.contains(s)
}

fn click(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerMoved { pos: p });
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
}

fn release(ui: &mut Ui, p: Vec2) {
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
}

fn centre(r: Rect) -> Vec2 {
    Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
}

/// A head that carries actions does not fold: pressing its title closes nothing, the body stays, and the actions
/// stand at the head's right end, on the line of the title.
#[test]
fn a_section_with_actions_keeps_its_body_and_puts_the_actions_at_the_end_of_the_head() {
    let mut ui = Ui::new(Theme::dark());
    let frame = |ui: &mut Ui| {
        ui.begin_frame(SIZE);
        let mut got = None;
        let mut ids = None;
        column(ui, |ui| {
            let section = ui.section(
                "mods",
                "Mods \u{b7} 13",
                |ui| {
                    let a = ui.action("update", "Update all (2)", ActionLook::Underlined);
                    let b = ui.action("add", "Add a mod", ActionLook::Underlined);
                    got = Some((a.id, b.id));
                },
                |ui| ui.label("The mods", Tone::Text),
            );
            ids = got.map(|g| (section, g.0, g.1));
        });
        (ui.end_frame().clone(), ids.unwrap())
    };
    frame(&mut ui);
    let (list, (section, update, add)) = frame(&mut ui);
    let whole = ui.rect(section).unwrap();
    let (u, a) = (ui.rect(update).unwrap(), ui.rect(add).unwrap());
    assert!((whole.x + whole.w - (a.x + a.w)).abs() < 0.5, "the last action ends where the head does");
    assert!(u.x + u.w < a.x, "the actions do not overlap");
    assert!((a.x - (u.x + u.w) - mere_ui::ACTIONS_GAP).abs() < 0.5, "the actions are one gap apart");
    let title = text_at(&list, "MODS \u{b7} 13");
    assert!(title.x < whole.x + 1.0 && text_at(&list, "The mods").y > title.y);
    // A press on the title and the sign's place changes nothing: no fold.
    let p = Vec2::new(whole.x + 20.0, title.y + 6.0);
    click(&mut ui, p);
    frame(&mut ui);
    release(&mut ui, p);
    let (after, _) = frame(&mut ui);
    assert!(said(&after, "The mods"), "the body stays");
    assert!(!said(&after, "\u{2212}") && !said(&after, "+"), "no fold sign");
}

/// The slots of a line have one width whatever the line holds, so the words of different states stand in the same
/// columns, and an empty slot keeps its width.
#[test]
fn lines_of_different_states_keep_their_slots_aligned() {
    let trash = icon();
    let mut ui = Ui::new(Theme::dark());
    let mut resp: Vec<ListLineResp> = Vec::new();
    let mut frame = |ui: &mut Ui| {
        ui.begin_frame(SIZE);
        resp.clear();
        column(ui, |ui| {
            let end = Some(LineEnd { icon: &trash, disabled: false });
            resp.push(ui.list_line("a", ListLine { name: "Sodium", version: "0.7.1", new_version: "\u{2192} 0.7.3", action: LineAction::Word("Update"), end, ..ListLine::default() }));
            resp.push(ui.list_line("b", ListLine { name: "Tectonic", version: "3.1.0", end, ..ListLine::default() }));
            resp.push(ui.list_line("c", ListLine { name: "Third", version: "1", action: LineAction::Done(&trash, "Added"), ..ListLine::default() }));
        });
        ui.end_frame().clone()
    };
    frame(&mut ui);
    let list = frame(&mut ui);
    let lines: Vec<Rect> = resp.iter().map(|r| ui.rect(r.line).unwrap()).collect();
    assert!(lines.iter().all(|l| (l.w - COL).abs() < 0.01), "every line is as wide as its column");
    assert!((lines[1].y - lines[0].y - lines[0].h).abs() < 0.01, "the lines are stacked");
    // The icon action sits at the end of its slot, which is there with or without it.
    let end = ui.rect(resp[0].end.id).unwrap();
    assert!((end.x + end.w - COL).abs() < 0.01, "the icon action ends at the line's end");
    assert!(ui.rect(resp[2].end.id).is_none(), "a line without an icon action has none");
    // The action's word ends where its slot ends, the same slot for every state.
    let slot_end = COL - LINE_END_W - GAP;
    let update = ui.rect(resp[0].action.id).unwrap();
    assert!((update.x + update.w - slot_end).abs() < 0.01, "the action ends at its slot's end: {update:?}");
    // The new version is right-aligned in its own slot, to the left of the action's.
    let new_end = slot_end - LINE_ACTION_W - GAP;
    let at = text_at(&list, "\u{2192} 0.7.3");
    let width = TextLookWidth::small("\u{2192} 0.7.3");
    assert!((at.x + width - new_end).abs() < 0.5, "the new version ends at its slot's end: {at:?}");
    assert!(new_end - LINE_NEW_W >= GAP, "the name has room");
    // "Added": the word ends at the action slot's end whatever the icon.
    let added = text_at(&list, "Added");
    assert!((added.x + TextLookWidth::body("Added") - slot_end).abs() < 0.5, "{added:?}");
    // The name and its version sit side by side on one line.
    let (name, version) = (text_at(&list, "Sodium"), text_at(&list, "0.7.1"));
    assert!(version.x > name.x + TextLookWidth::body("Sodium") && (version.y - name.y).abs() < 3.0);
}

struct TextLookWidth;

impl TextLookWidth {
    fn body(s: &str) -> f32 {
        mere_ui::TextLook::body(Tone::Text).width(s)
    }
    fn small(s: &str) -> f32 {
        mere_ui::TextLook::small(Tone::Text).width(s)
    }
}

/// A press on the action, the icon and the link name each answers for itself; a name that is no link answers for
/// nothing, and a part that is not there never fires.
#[test]
fn the_parts_of_a_line_answer_a_press_each() {
    let trash = icon();
    let mut resp = None;
    let mut frame = |ui: &mut Ui, link: bool| {
        ui.begin_frame(SIZE);
        column(ui, |ui| {
            let end = Some(LineEnd { icon: &trash, disabled: false });
            resp = Some(ui.list_line("a", ListLine { name: "Sodium", version: "0.7.1", link, new_version: "\u{2192} 0.7.3", action: LineAction::Word("Update"), end, ..ListLine::default() }));
        });
        ui.end_frame();
        resp.unwrap()
    };
    // The click itself, press and release in the frames a window sends them.
    let mut clicked = (false, false, false);
    for target in 0..3 {
        let mut ui = Ui::new(Theme::dark());
        frame(&mut ui, true);
        let r = frame(&mut ui, true);
        let rect = [ui.rect(r.action.id).unwrap(), ui.rect(r.end.id).unwrap(), ui.rect(r.name.id).unwrap()][target];
        let p = centre(rect);
        ui.event(&Event::PointerMoved { pos: p });
        frame(&mut ui, true);
        ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
        frame(&mut ui, true);
        ui.event(&Event::PointerUp { pos: p, button: Button::Left });
        let r = frame(&mut ui, true);
        match target {
            0 => clicked.0 = r.action.clicked && !r.end.clicked && !r.name.clicked,
            1 => clicked.1 = r.end.clicked && !r.action.clicked && !r.name.clicked,
            _ => clicked.2 = r.name.clicked && !r.action.clicked && !r.end.clicked,
        }
    }
    assert_eq!(clicked, (true, true, true));
    // A name that is no link is not a part to press.
    let mut ui = Ui::new(Theme::dark());
    frame(&mut ui, false);
    let r = frame(&mut ui, false);
    assert!(r.name.rect.is_none() && !r.name.clicked);
}

/// A line with a description is two lines tall and one without it is a square tall; the note is a last line.
#[test]
fn a_line_is_a_square_tall_and_two_lines_with_a_description() {
    let trash = icon();
    let mut ui = Ui::new(Theme::dark());
    let mut got = Vec::new();
    let mut frame = |ui: &mut Ui| {
        ui.begin_frame(SIZE);
        got.clear();
        column(ui, |ui| {
            got.push(ui.list_line("a", ListLine { name: "One", ..ListLine::default() }).line);
            got.push(ui.list_line("b", ListLine { name: "Two", detail: "About it.", ..ListLine::default() }).line);
            got.push(ui.list_line("c", ListLine { name: "Three", detail: "About it.", note: "And more.", end: Some(LineEnd { icon: &trash, disabled: false }), ..ListLine::default() }).line);
        });
        ui.end_frame();
    };
    frame(&mut ui);
    frame(&mut ui);
    let h: Vec<f32> = got.iter().map(|id| ui.rect(*id).unwrap().h).collect();
    // The separator under the line is part of it: one frame.
    assert!((h[0] - (ICON_SQUARE + 1.0)).abs() < 0.01, "{h:?}");
    assert!(h[1] > h[0] + 10.0 && h[2] > h[1] + 10.0, "{h:?}");
}

/// The title line is as tall as a large control with or without an action at its end, its back action answers a
/// press, and the title keeps its case.
#[test]
fn a_page_head_keeps_its_height_and_the_titles_case() {
    let back = icon();
    let mut ui = Ui::new(Theme::dark());
    let mut got = None;
    let mut frame = |ui: &mut Ui, action: bool| {
        ui.begin_frame(SIZE);
        column(ui, |ui| {
            got = Some(ui.page_head("h", &back, "Exploration", |ui| {
                if action {
                    ui.action("play", "Play", ActionLook::Primary);
                }
            }));
        });
        let list = ui.end_frame().clone();
        (got.unwrap(), list)
    };
    frame(&mut ui, false);
    let (plain, list) = frame(&mut ui, false);
    let h = ui.rect(plain.head).unwrap().h;
    assert!(said(&list, "Exploration") && !said(&list, "EXPLORATION"), "the title keeps its case");
    frame(&mut ui, true);
    let (with, _) = frame(&mut ui, true);
    assert!((ui.rect(with.head).unwrap().h - h).abs() < 0.01, "an action at the end adds no height");
    let p = centre(ui.rect(with.back.id).unwrap());
    ui.event(&Event::PointerMoved { pos: p });
    frame(&mut ui, true);
    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
    frame(&mut ui, true);
    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
    let (after, _) = frame(&mut ui, true);
    assert!(after.back.clicked, "the back action reports its press");
}
