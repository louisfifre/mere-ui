//! The launcher's Home controls in their states: the icon buttons, the select and its menu, the disabled and
//! left-aligned fields, the toggle, the lit row, the underlined word. Each cell is its own `Ui` so that one
//! pointer can rest on each; `--launcher` opens them live.

use std::cell::{Cell, RefCell};

use nui_base::{Button, DrawList, Event, Vec2};
use nui_layout::{Align, Dim, Edges, Style};
use nui_ui::{ButtonLook, ButtonOpts, FieldOpts, Icon, IconLook, IconOpts, Id, NumberOpts, NumberSpec, SelectOpts, TextLook, Theme, ToggleOpts, Tone, Ui, mix};

use crate::absolute_at;

pub const LAUNCHER: (u32, u32) = (860, 460);

/// Stand-in shapes: the sheet shows the buttons' looks, not the launcher's own icons (those are in preview::bar).
const PLAY: &str = "M6 4h2v16H6zm2 2h2v12H8zm2 2h2v8h-2zm2 2h2v4h-2z";
const SETTINGS: &str = "M10 2h4v2h-4zM8 4h2v2H8zm6 0h2v2h-2zM6 6h2v2H6zm10 0h2v2h-2zM4 8h2v2H4zm14 0h2v2h-2zM2 10h2v4H2zm18 0h2v4h-2zM4 14h2v2H4zm14 0h2v2h-2zM6 16h2v2H6zm10 0h2v2h-2zM8 18h2v2H8zm6 0h2v2h-2zm-4 2h4v2h-4z";
const BACK: &str = "M11 5h2v2h-2zM9 7h2v2H9zM7 9h2v2H7zm0 2h10v2H7zm0 2h2v2H7zm2 2h2v2H9zm2 2h2v2h-2z";

#[derive(Copy, Clone, PartialEq)]
enum Mode {
    Rest,
    Hover,
    /// Click the cell's id, then rest the pointer on row `n` of the menu that opened.
    Menu(usize),
}

fn icon(path: &str) -> Icon {
    Icon::parse(path, 24.0).expect("the sheet's icon parses")
}

fn cell(theme: Theme, at: Vec2, w: f32, mode: Mode, build: impl Fn(&mut Ui) -> Id) -> DrawList {
    let size = Vec2::new(LAUNCHER.0 as f32, LAUNCHER.1 as f32);
    let mut ui = Ui::new(theme);
    let id: Cell<Option<Id>> = Cell::new(None);
    let frame = |ui: &mut Ui| {
        ui.begin_frame(size);
        ui.node("cell", Style { column: true, w: Dim::Px(w), ..absolute_at(at) }, |ui| id.set(Some(build(ui))));
        ui.end_frame().clone()
    };
    let mut list = frame(&mut ui);
    let r = ui.rect(id.get().unwrap()).expect("the cell was built once");
    let centre = Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0);
    match mode {
        Mode::Rest => {}
        Mode::Hover => {
            ui.event(&Event::PointerMoved { pos: centre });
            list = frame(&mut ui);
        }
        Mode::Menu(row) => {
            ui.event(&Event::PointerMoved { pos: centre });
            ui.event(&Event::PointerDown { pos: centre, button: Button::Left, clicks: 1 });
            frame(&mut ui);
            ui.event(&Event::PointerUp { pos: centre, button: Button::Left });
            frame(&mut ui);
            let p = Vec2::new(r.x + 30.0, r.y + r.h + 1.0 + row as f32 * nui_ui::MENU_ROW + 10.0);
            ui.event(&Event::PointerMoved { pos: p });
            list = frame(&mut ui);
        }
    }
    list
}

fn square(look: IconLook, path: &'static str, opts: IconOpts) -> impl Fn(&mut Ui) -> Id {
    move |ui| ui.icon_button("b", &icon(path), look, opts).id
}

fn caption(ui: &mut Ui, at: Vec2, words: &str) {
    ui.node(&format!("cap-{words}"), absolute_at(at), |ui| ui.text(words, TextLook::status(Tone::Muted)));
}

fn options() -> Vec<String> {
    ["the game menu", "world: New World", "a server"].map(String::from).to_vec()
}

/// Every cell in its state, with the pointer where it says.
pub fn launcher_sheet(theme: Theme) -> DrawList {
    let size = Vec2::new(LAUNCHER.0 as f32, LAUNCHER.1 as f32);
    let rest = IconOpts::default();
    let mut list = DrawList::default();
    let mut caps = Ui::new(theme);
    caps.begin_frame(size);
    for (x, y, w) in [
        (20.0, 10.0, "Square: rest, hover, expanded, disabled, joined"),
        (20.0, 70.0, "Filled: rest, hover, disabled; Bare"),
        (20.0, 130.0, "Select: closed, open (row 2 hot), disabled"),
        (20.0, 270.0, "Fields: disabled, 8ch right, 100% left, 52 px"),
        (20.0, 350.0, "Toggle on, off, disabled; lit row; Open"),
    ] {
        caption(&mut caps, Vec2::new(x, y), w);
    }
    list.cmds.extend(caps.end_frame().cmds.iter().cloned());

    let at = |x: f32, y: f32| Vec2::new(x, y);
    let mut add = |l: DrawList| list.cmds.extend(l.cmds);
    // Row 1: Square.
    add(cell(theme, at(20.0, 30.0), 30.0, Mode::Rest, square(IconLook::Square, SETTINGS, rest)));
    add(cell(theme, at(80.0, 30.0), 30.0, Mode::Hover, square(IconLook::Square, SETTINGS, rest)));
    add(cell(theme, at(140.0, 30.0), 30.0, Mode::Rest, square(IconLook::Square, SETTINGS, IconOpts { pressed: true, ..rest })));
    add(cell(theme, at(200.0, 30.0), 30.0, Mode::Rest, square(IconLook::Square, SETTINGS, IconOpts { disabled: true, ..rest })));
    add(cell(theme, at(260.0, 30.0), 80.0, Mode::Rest, |ui| {
        ui.node("pair", Style { align_items: Align::Start, ..Style::default() }, |ui| {
            ui.icon_button("a", &icon(PLAY), IconLook::Filled, rest);
            ui.icon_button("b", &icon(SETTINGS), IconLook::Square, IconOpts { joined: true, ..rest });
        })
    }));
    // Row 2: Filled and Bare.
    add(cell(theme, at(20.0, 90.0), 30.0, Mode::Rest, square(IconLook::Filled, PLAY, rest)));
    add(cell(theme, at(80.0, 90.0), 30.0, Mode::Hover, square(IconLook::Filled, PLAY, rest)));
    add(cell(theme, at(140.0, 90.0), 30.0, Mode::Rest, square(IconLook::Filled, PLAY, IconOpts { disabled: true, ..rest })));
    add(cell(theme, at(220.0, 90.0), 24.0, Mode::Rest, square(IconLook::Bare, BACK, rest)));
    // Row 3: the select.
    for (x, mode, disabled) in [(20.0, Mode::Rest, false), (300.0, Mode::Menu(2), false), (580.0, Mode::Rest, true)] {
        let chosen = Cell::new(1usize);
        add(cell(theme, at(x, 150.0), 240.0, mode, |ui| {
            let mut c = chosen.get();
            let r = ui.select_menu("sel", &options(), &mut c, SelectOpts { disabled, ..SelectOpts::default() });
            chosen.set(c);
            r.resp.id
        }));
    }
    // Row 4: fields.
    let s = RefCell::new(String::from("localhost"));
    add(cell(theme, at(20.0, 290.0), 150.0, Mode::Rest, |ui| {
        ui.text_field("t", &mut s.borrow_mut(), FieldOpts { width: Dim::Pct(100.0), disabled: true, ..FieldOpts::default() }).resp.id
    }));
    let spec = NumberSpec { min: 2.0, max: 24.0, step: 1.0 };
    add(cell(theme, at(190.0, 290.0), 80.0, Mode::Rest, |ui| {
        let mut v = 6.0;
        ui.number_with("m", &mut v, spec, NumberOpts { width: Dim::Px(57.6), right: true, pad_v: 4.0, disabled: false }).resp.id
    }));
    add(cell(theme, at(300.0, 290.0), 150.0, Mode::Rest, |ui| {
        let mut v = 12.0;
        ui.number_with("v", &mut v, NumberSpec { min: 3.0, max: 32.0, step: 1.0 }, NumberOpts { width: Dim::Pct(100.0), right: false, pad_v: 4.0, disabled: false }).resp.id
    }));
    let java = RefCell::new(String::from("-XX:+UseZGC"));
    add(cell(theme, at(480.0, 290.0), 300.0, Mode::Rest, |ui| {
        ui.text_field("j", &mut java.borrow_mut(), FieldOpts { width: Dim::Pct(100.0), height: Dim::Px(52.0), ..FieldOpts::default() }).resp.id
    }));
    // Row 5: toggles, the lit row and the underlined word.
    for (x, on, disabled) in [(20.0, true, false), (110.0, false, false), (200.0, true, true)] {
        let on = Cell::new(on);
        add(cell(theme, at(x, 370.0), 58.0, Mode::Rest, |ui| {
            let mut v = on.get();
            let r = ui.toggle_with("t", &mut v, "on", "off", ToggleOpts { min_w: Dim::Px(57.6), pad: Edges::vh(4.0, 7.0), disabled });
            on.set(v);
            r.id
        }));
    }
    add(cell(theme, at(300.0, 360.0), 320.0, Mode::Hover, |ui| {
        let lit = mix(theme.text, 0.06, theme.ground);
        ui.hover_box("row", Style { align_items: Align::Center, pad: Edges::vh(10.0, 8.0), ..Style::default() }, lit, |ui| {
            ui.text("Exploration", TextLook { size: 13.0, line: 19.5, ..TextLook::body(Tone::Text) });
        })
    }));
    add(cell(theme, at(660.0, 370.0), 60.0, Mode::Rest, |ui| ui.button_with("o", "Open", ButtonLook::Underlined, ButtonOpts::default()).id));
    list
}

/// What the live window keeps.
#[derive(Clone, Debug)]
pub struct LauncherState {
    pub chosen: usize,
    pub on: bool,
    pub memory: f64,
    pub view: f64,
    pub address: String,
    pub java: String,
    pub expanded: bool,
}

impl Default for LauncherState {
    fn default() -> Self {
        LauncherState { chosen: 1, on: true, memory: 6.0, view: 12.0, address: String::new(), java: "-XX:+UseZGC".into(), expanded: false }
    }
}

/// The controls under a real pointer and keyboard.
pub fn launcher_live(ui: &mut Ui, s: &mut LauncherState) {
    let t = *ui.theme();
    let col = Style { column: true, w: Dim::Px(360.0), ..absolute_at(Vec2::new(20.0, 20.0)) };
    ui.node("col", col, |ui| {
        ui.node("icons", Style { align_items: Align::Center, ..Style::default() }, |ui| {
            ui.icon_button("play", &icon(PLAY), IconLook::Filled, IconOpts::default());
            if ui.icon_button("set", &icon(SETTINGS), IconLook::Square, IconOpts { pressed: s.expanded, joined: true, ..IconOpts::default() }).clicked {
                s.expanded = !s.expanded;
            }
        });
        ui.select_menu("sel", &options(), &mut s.chosen, SelectOpts::default());
        if s.chosen == 2 {
            ui.text_field("addr", &mut s.address, FieldOpts { placeholder: "address, for example localhost", width: Dim::Pct(100.0), ..FieldOpts::default() });
        }
        ui.number_with("mem", &mut s.memory, NumberSpec { min: 2.0, max: 24.0, step: 1.0 }, NumberOpts { width: Dim::Px(57.6), right: true, pad_v: 4.0, disabled: false });
        ui.number_with("view", &mut s.view, NumberSpec { min: 3.0, max: 32.0, step: 1.0 }, NumberOpts { width: Dim::Pct(100.0), right: false, pad_v: 4.0, disabled: false });
        ui.text_field("java", &mut s.java, FieldOpts { width: Dim::Pct(100.0), height: Dim::Px(52.0), ..FieldOpts::default() });
        ui.toggle_with("on", &mut s.on, "on", "off", ToggleOpts { min_w: Dim::Px(57.6), pad: Edges::vh(4.0, 7.0), disabled: false });
        ui.hover_box("row", Style { pad: Edges::vh(10.0, 8.0), ..Style::default() }, mix(t.text, 0.06, t.ground), |ui| ui.text("Exploration", TextLook::body(Tone::Text)));
        ui.button_with("open", "Open", ButtonLook::Underlined, ButtonOpts::default());
    });
}