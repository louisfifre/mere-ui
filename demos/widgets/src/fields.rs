//! The page's text and number fields: live in a window, and as a sheet. In the sheet every cell is its own
//! `Ui` driven by a script of events, the way a person would drive it, so the picture shows real states:
//! a double click's selection, a Tab's, a long text scrolled with its caret at the edge.

use nui_base::{Button, DrawList, Event, Ime, Key, Mods, Vec2};
use nui_layout::{Dim, Style};
use nui_ui::{FieldLook, FieldOpts, Id, NumberSpec, TextLook, Theme, Tone, Ui};

use crate::{absolute_at, setting_style, text};

pub const FIELDS: (u32, u32) = (860, 500);

/// The script column: the field of row `i` is at (SCRIPT_X, SCRIPT_Y0 + SCRIPT_STEP i), SCRIPT_W wide.
pub const SCRIPT_X: f32 = 600.0;
pub const SCRIPT_Y0: f32 = 44.0;
pub const SCRIPT_STEP: f32 = 34.0;
pub const SCRIPT_W: f32 = 240.0;
/// The left column: x of every cell, y of each cell's caption, and how far under it the field sits.
pub const LEFT_X: f32 = 20.0;
pub const CELL_Y: [f32; 6] = [20.0, 80.0, 140.0, 200.0, 260.0, 360.0];
pub const FIELD_DY: f32 = 20.0;

/// What the application owns: the fields keep no text of their own.
#[derive(Clone, Debug)]
pub struct FieldsState {
    pub seed: String,
    pub filter: String,
    pub name: String,
    pub values: [f64; 2],
}

impl Default for FieldsState {
    fn default() -> FieldsState {
        FieldsState { seed: String::new(), filter: String::new(), name: text::TRY[1].to_string(), values: [1.0, 0.5] }
    }
}

/// `vertical_scale` and `elevation_boost` of the page's `known` table (line 655).
pub(crate) const SCALE: NumberSpec = NumberSpec { min: 0.5, max: 3.0, step: 0.025 };
const BOOST: NumberSpec = NumberSpec { min: 0.0, max: 1.0, step: 0.05 };

fn seed_opts(width: f32) -> FieldOpts<'static> {
    FieldOpts { placeholder: text::SEED_RANDOM, max_chars: 64, width: Dim::Px(width), ..FieldOpts::default() }
}

fn filter_opts() -> FieldOpts<'static> {
    FieldOpts { placeholder: text::FILTER, width: Dim::Px(260.0), ..FieldOpts::default() }
}

/// The profile name as the page creates it: compact, 24 chars, focused with all of it selected.
fn name_opts() -> FieldOpts<'static> {
    FieldOpts { look: FieldLook::Compact, placeholder: text::PROFILE_NAME, max_chars: 24, width: Dim::Px(130.0), focus: true }
}

/// A `.setting` row: its label, an empty output, and a number field.
pub(crate) fn setting_row(ui: &mut Ui, key: &str, w: Dim, label: &str, v: &mut f64, spec: NumberSpec) -> Id {
    let mut id = None;
    ui.node(key, setting_style(w), |ui| {
        ui.label(label, Tone::Text);
        ui.text("", TextLook::body(Tone::Muted));
        id = Some(ui.number("field", v, spec).resp.id);
    });
    id.expect("the row builds its field")
}

/// What the window shows: every field live, one state, the system clipboard.
pub fn fields_live(ui: &mut Ui, s: &mut FieldsState) {
    let col = Style { column: true, gap: Vec2::new(0.0, 8.0), w: Dim::Px(300.0), ..absolute_at(Vec2::new(20.0, 20.0)) };
    ui.node("column", col, |ui| {
        ui.label(text::SEED, Tone::Text);
        ui.text_field("seed", &mut s.seed, seed_opts(300.0));
        ui.text_field("filter", &mut s.filter, FieldOpts { width: Dim::Auto, ..filter_opts() });
        ui.text_field("name", &mut s.name, FieldOpts { focus: false, ..name_opts() });
        let [scale, boost] = &mut s.values;
        setting_row(ui, "scale", Dim::Auto, text::VERTICAL_SCALE, scale, SCALE);
        setting_row(ui, "boost", Dim::Auto, text::ELEVATION_BOOST, boost, BOOST);
    });
}

// ---- the scripted cells ----

/// What a step does to the field, by the char column under the pointer (0 is the start of the content box).
#[derive(Clone)]
pub(crate) enum Ev {
    Click(f32, u8),
    Key(Key, Mods),
    Type(&'static str),
    /// The system's input method, one message at a time.
    Ime(Ime),
}

pub(crate) const NONE: Mods = Mods { ctrl: false, shift: false };
const CTRL: Mods = Mods { ctrl: true, shift: false };
pub(crate) const SHIFT: Mods = Mods { ctrl: false, shift: true };

pub(crate) fn rep(key: Key, mods: Mods, n: usize) -> Vec<Ev> {
    vec![Ev::Key(key, mods); n]
}

/// Builds one frame of a cell and answers the id of the field the script's clicks aim at.
pub(crate) type Build<'a> = dyn Fn(&mut Ui, &mut FieldsState) -> Id + 'a;

/// A fresh `Ui` per cell: frame 0 builds with no events (so every widget knows where it stands), then each
/// step injects its events and builds one frame, and one more when the frame asked for it (a Tab's focus
/// only shows on the next frame). The last frame's list is the cell's picture.
pub(crate) fn drive(theme: Theme, st: &mut FieldsState, build: &Build, steps: &[Vec<Ev>]) -> DrawList {
    let size = Vec2::new(FIELDS.0 as f32, FIELDS.1 as f32);
    let mut ui = Ui::new(theme);
    let frame = |ui: &mut Ui, st: &mut FieldsState| {
        ui.begin_frame(size);
        let id = build(ui, st);
        (id, ui.end_frame().clone())
    };
    let (mut id, mut list) = frame(&mut ui, st);
    for step in steps {
        for ev in step {
            match ev {
                Ev::Click(col, clicks) => {
                    let r = ui.rect(id).expect("frame 0 laid the field out");
                    // The content box starts 7 in (padding 6 and the border); the extra 1 puts the pointer inside
                    // the char rather than on the gap between two, so a word click means the char at `col`.
                    let p = Vec2::new(r.x + 7.0 + 7.2 * *col + 1.0, r.y + r.h / 2.0);
                    ui.event(&Event::PointerMoved { pos: p });
                    ui.event(&Event::PointerDown { pos: p, button: Button::Left, clicks: *clicks });
                    ui.event(&Event::PointerUp { pos: p, button: Button::Left });
                }
                Ev::Key(key, mods) => {
                    ui.event(&Event::Key { key: *key, mods: *mods });
                }
                Ev::Ime(i) => {
                    ui.event(&Event::Ime(i.clone()));
                }
                Ev::Type(s) => {
                    for ch in s.chars() {
                        ui.event(&Event::Char { ch });
                    }
                }
            }
        }
        (id, list) = frame(&mut ui, st);
        if ui.want_redraw() {
            (id, list) = frame(&mut ui, st);
        }
    }
    list
}

fn at_cell(i: usize) -> Vec2 {
    Vec2::new(LEFT_X, CELL_Y[i] + FIELD_DY)
}

fn l0_filter(ui: &mut Ui, s: &mut FieldsState) -> Id {
    let mut id = None;
    ui.node("l0", absolute_at(at_cell(0)), |ui| id = Some(ui.text_field("filter", &mut s.filter, filter_opts()).resp.id));
    id.unwrap()
}

fn l1_name(ui: &mut Ui, s: &mut FieldsState) -> Id {
    let mut id = None;
    ui.node("l1", absolute_at(at_cell(1)), |ui| id = Some(ui.text_field("name", &mut s.name, name_opts()).resp.id));
    id.unwrap()
}

fn l2_setting(ui: &mut Ui, s: &mut FieldsState) -> Id {
    let mut id = None;
    ui.node("l2", absolute_at(at_cell(2)), |ui| id = Some(setting_row(ui, "row", Dim::Px(330.0), text::VERTICAL_SCALE, &mut s.values[0], SCALE)));
    id.unwrap()
}

fn l3_setting(ui: &mut Ui, s: &mut FieldsState) -> Id {
    let mut id = None;
    ui.node("l3", absolute_at(at_cell(3)), |ui| id = Some(setting_row(ui, "row", Dim::Px(330.0), text::VERTICAL_SCALE, &mut s.values[0], SCALE)));
    id.unwrap()
}

/// The seed field with a setting row 8 under it: where Tab goes from one to the other.
fn l4_tab(ui: &mut Ui, s: &mut FieldsState) -> Id {
    let mut id = None;
    let col = Style { column: true, gap: Vec2::new(0.0, 8.0), w: Dim::Px(260.0), ..absolute_at(at_cell(4)) };
    ui.node("l4", col, |ui| {
        id = Some(ui.text_field("seed", &mut s.seed, FieldOpts { width: Dim::Auto, ..seed_opts(260.0) }).resp.id);
        setting_row(ui, "row", Dim::Auto, text::VERTICAL_SCALE, &mut s.values[1], SCALE);
    });
    id.unwrap()
}

fn l5_empty(ui: &mut Ui, s: &mut FieldsState) -> Id {
    let mut id = None;
    ui.node("l5", absolute_at(at_cell(5)), |ui| id = Some(ui.text_field("seed", &mut s.seed, seed_opts(260.0)).resp.id));
    id.unwrap()
}

fn script_field(ui: &mut Ui, s: &mut FieldsState, y: f32) -> Id {
    let mut id = None;
    ui.node("script", absolute_at(Vec2::new(SCRIPT_X, y)), |ui| id = Some(ui.text_field("seed", &mut s.seed, seed_opts(SCRIPT_W)).resp.id));
    id.unwrap()
}

/// The 13 rows of the script: row 0 is the start, row `i` the step done after the rows before it.
fn script() -> Vec<Vec<Ev>> {
    let mut rows = vec![Vec::new()];
    rows.push(vec![Ev::Click(0.0, 1)]);
    rows.push(vec![Ev::Type(text::DEMO_DIGITS)]);
    rows.push(vec![Ev::Key(Key::Home, NONE)]);
    rows.push(rep(Key::Right, NONE, 4));
    rows.push(rep(Key::Left, NONE, 2));
    rows.push(rep(Key::Right, SHIFT, 4));
    rows.push(vec![Ev::Key(Key::C, CTRL), Ev::Key(Key::End, NONE), Ev::Key(Key::V, CTRL)]);
    rows.push(rep(Key::Backspace, NONE, 2));
    rows.push(vec![Ev::Type(text::DEMO_LONG)]);
    rows.push(vec![Ev::Key(Key::Backspace, CTRL)]);
    rows.push(vec![Ev::Key(Key::A, CTRL)]);
    rows.push(vec![Ev::Key(Key::Escape, NONE)]);
    rows
}

fn captions(ui: &mut Ui) {
    let caps = [text::CAP_FILTER, text::CAP_NAME, text::CAP_STEP, text::CAP_CLAMP, text::CAP_TAB, text::CAP_EMPTY];
    for (i, words) in caps.into_iter().enumerate() {
        ui.node(&format!("cap{i}"), absolute_at(Vec2::new(LEFT_X, CELL_Y[i])), |ui| ui.text(words, TextLook::status(Tone::Muted)));
    }
    ui.node("cap-script", absolute_at(Vec2::new(440.0, 20.0)), |ui| ui.text(text::CAP_SCRIPT, TextLook::status(Tone::Muted)));
    for (i, words) in text::SCRIPT.into_iter().enumerate() {
        let y = SCRIPT_Y0 + SCRIPT_STEP * i as f32 + 5.0;
        ui.node(&format!("row{i}"), absolute_at(Vec2::new(440.0, y)), |ui| ui.text(words, TextLook::status(Tone::Muted)));
    }
}

/// Every cell in its state, driven by its script, as one draw list.
pub fn fields_sheet(theme: Theme) -> DrawList {
    let size = Vec2::new(FIELDS.0 as f32, FIELDS.1 as f32);
    let mut list = DrawList::default();
    let mut ui = Ui::new(theme);
    ui.begin_frame(size);
    captions(&mut ui);
    list.cmds.extend(ui.end_frame().cmds.iter().cloned());

    let click = |col: f32, clicks: u8| vec![Ev::Click(col, clicks)];
    let seeded = |f: fn(&mut FieldsState)| {
        let mut s = FieldsState::default();
        f(&mut s);
        s
    };
    type Cell = (&'static Build<'static>, FieldsState, Vec<Vec<Ev>>);
    let cells: Vec<Cell> = vec![
        // The filter: a first click puts the caret in "ocean", the second (a double click) selects the word.
        (&l0_filter, seeded(|s| s.filter = text::DEMO_FILTER.into()), vec![click(7.0, 1), click(7.0, 2)]),
        (&l1_name, FieldsState::default(), vec![]),
        // Two steps up from 1: 1.025, then 1.05.
        (&l2_setting, FieldsState::default(), vec![click(0.0, 1), vec![Ev::Key(Key::Up, NONE)], vec![Ev::Key(Key::Up, NONE)]]),
        (
            &l3_setting,
            FieldsState::default(),
            vec![click(0.0, 1), vec![Ev::Key(Key::A, CTRL)], vec![Ev::Type("9")], vec![Ev::Key(Key::Enter, NONE)]],
        ),
        (&l4_tab, seeded(|s| { s.seed = text::DEMO_SEED.into(); s.values[1] = 1.5; }), vec![click(0.0, 1), vec![Ev::Key(Key::Tab, NONE)]]),
        (&l5_empty, FieldsState::default(), vec![]),
    ];
    for (build, mut state, steps) in cells {
        list.cmds.extend(drive(theme, &mut state, build, &steps).cmds);
    }

    let script = script();
    for i in 0..script.len() {
        let y = SCRIPT_Y0 + SCRIPT_STEP * i as f32;
        let mut state = FieldsState::default();
        let build = move |ui: &mut Ui, s: &mut FieldsState| script_field(ui, s, y);
        list.cmds.extend(drive(theme, &mut state, &build, &script[1..1 + i]).cmds);
    }
    list
}
