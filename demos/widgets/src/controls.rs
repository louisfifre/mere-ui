//! The page's folds, strips, choices, details and number fields: live in a window, and as a sheet where each
//! cell is its own `Ui` so that one pointer can rest on each.

use nui_base::{DrawList, Event, Vec2};
use nui_layout::{Dim, Style};
use nui_ui::{FoldOpts, Id, NumberSpec, TabLook, TabOpts, TextLook, Theme, Tone, Ui, choice_style, details_style, strip_style};

use crate::{absolute_at, setting_style, text};

pub const CONTROLS: (u32, u32) = (860, 440);

/// What the app owns: the widgets keep no state, as the page keeps its own.
#[derive(Clone, Debug)]
pub struct ControlsState {
    pub view: usize,
    pub in3d: bool,
    pub profile: usize,
    pub detail: usize,
    pub world_open: bool,
    pub history_open: bool,
    pub settings_open: bool,
    pub rolling: bool,
    /// The three number settings: vertical scale, bottom of the world, land against ocean.
    pub values: [f64; 3],
}

impl Default for ControlsState {
    fn default() -> ControlsState {
        // View 1 is Flat: the build plan's "second tab selected".
        ControlsState { view: 1, in3d: false, profile: 0, detail: 1, world_open: false, history_open: false, settings_open: true, rolling: false, values: [1.0, -64.0, -0.45] }
    }
}

type Cell = fn(&mut Ui, &mut ControlsState) -> Option<Id>;

/// A column of width `w` placed at (x, y).
fn col_at(x: f32, y: f32, w: f32) -> Style {
    Style { column: true, w: Dim::Px(w), ..absolute_at(Vec2::new(x, y)) }
}

/// The strip: views, a gap, the shapes. Returns the tabs' ids in order and the index of the one clicked
/// (0..6 views, 6 is 2D, 7 is 3D). `coarse` disables what needs the full map (`needsFull`, line 506).
fn strip_row(ui: &mut Ui, view: usize, in3d: bool, coarse: bool) -> (Vec<Id>, Option<usize>) {
    let (mut ids, mut clicked) = (Vec::new(), None);
    ui.node("strip", strip_style(), |ui| {
        for (i, w) in text::VIEWS.into_iter().chain([text::SECTION]).enumerate() {
            let disabled = coarse && matches!(i, 1 | 4 | 5);
            let r = ui.tab(w, w, TabLook::Strip, TabOpts { pressed: i == view, disabled });
            ids.push(r.id);
            if r.clicked {
                clicked = Some(i);
            }
        }
        ui.strip_gap();
        for (i, w) in text::SHAPES.into_iter().enumerate() {
            let r = ui.tab(w, w, TabLook::Strip, TabOpts { pressed: (i == 1) == in3d, disabled: false });
            ids.push(r.id);
            if r.clicked {
                clicked = Some(6 + i);
            }
        }
    });
    (ids, clicked)
}

fn c1_strip(ui: &mut Ui, s: &mut ControlsState) -> Option<Id> {
    let mut ids = Vec::new();
    ui.node("c1", absolute_at(Vec2::new(20.0, 36.0)), |ui| {
        let (i, clicked) = strip_row(ui, s.view, s.in3d, false);
        ids = i;
        match clicked {
            Some(i) if i < 6 => s.view = i,
            Some(i) => s.in3d = i == 7,
            None => {}
        }
    });
    Some(ids[3])
}

fn c2_coarse(ui: &mut Ui, _: &mut ControlsState) -> Option<Id> {
    let mut ids = Vec::new();
    ui.node("c2", absolute_at(Vec2::new(20.0, 96.0)), |ui| ids = strip_row(ui, 0, true, true).0);
    Some(ids[0])
}

/// A `.choice` row of Pack and the first `n - 1` profiles.
fn choice_row(ui: &mut Ui, profile: &mut usize, n: usize) -> Vec<Id> {
    let mut ids = Vec::new();
    ui.node("choice", choice_style(), |ui| {
        let words = std::iter::once(text::PACK).chain(text::TRY);
        for (i, w) in words.take(n).enumerate() {
            // The last of four is the unreachable one in the sheet (line 202).
            let r = ui.tab(w, w, TabLook::Choice, TabOpts { pressed: i == *profile, disabled: n == 4 && i == 3 });
            ids.push(r.id);
            if r.clicked {
                *profile = i;
            }
        }
    });
    ids
}

fn c3_choice(ui: &mut Ui, s: &mut ControlsState) -> Option<Id> {
    let mut ids = Vec::new();
    ui.node("c3", absolute_at(Vec2::new(20.0, 156.0)), |ui| ids = choice_row(ui, &mut s.profile, 4));
    Some(ids[2])
}

fn c4_details(ui: &mut Ui, s: &mut ControlsState) -> Option<Id> {
    ui.node("c4", absolute_at(Vec2::new(460.0, 156.0)), |ui| {
        ui.node("details", Style { w: Dim::Px(300.0), ..details_style() }, |ui| {
            for (i, (word, sub)) in text::DETAILS.into_iter().enumerate() {
                let r = ui.detail_tab(word, word, sub, TabOpts { pressed: i == s.detail, disabled: i == 2 });
                if r.clicked {
                    s.detail = i;
                }
            }
        });
    });
    None
}

fn c5_world(ui: &mut Ui, s: &mut ControlsState) -> Option<Id> {
    ui.node("c5", col_at(20.0, 234.0, 320.0), |ui| {
        ui.fold("world", text::WORLD, &mut s.world_open, |ui| ui.label(text::SEED, Tone::Text));
    });
    None
}

fn c6_history(ui: &mut Ui, s: &mut ControlsState) -> Option<Id> {
    let mut head = None;
    ui.node("c6", col_at(20.0, 330.0, 320.0), |ui| {
        head = Some(ui.fold("history", text::HISTORY, &mut s.history_open, |ui| ui.text(text::NO_RUN, TextLook::status(Tone::Muted))).id);
    });
    head
}

fn c7_settings(ui: &mut Ui, s: &mut ControlsState) -> Option<Id> {
    ui.node("c7", col_at(460.0, 234.0, 330.0), |ui| {
        let ControlsState { settings_open, profile, rolling, values, .. } = s;
        ui.fold_with("settings", text::SETTINGS, settings_open, FoldOpts::default(), |ui| {
            choice_row(ui, profile, 2);
            ui.node("rows", Style { column: true, ..Style::default() }, |ui| {
                // The page's `known` table (line 655): vertical_scale, min_y and ocean_offset.
                let specs = [
                    NumberSpec { min: 0.5, max: 3.0, step: 0.025 },
                    NumberSpec { min: -256.0, max: 0.0, step: 16.0 },
                    NumberSpec { min: -1.0, max: -0.2, step: 0.05 },
                ];
                let rows = [("scale", text::VERTICAL_SCALE), ("bottom", text::BOTTOM), ("land", text::LAND)];
                for (i, (key, label)) in rows.into_iter().enumerate() {
                    ui.node(key, setting_style(Dim::Auto), |ui| {
                        ui.label(label, Tone::Text);
                        ui.text("", TextLook::body(Tone::Muted));
                        ui.number("field", &mut values[i], specs[i]);
                    });
                }
                ui.node("rolling", setting_style(Dim::Auto), |ui| {
                    ui.label(text::ROLLING, Tone::Text);
                    ui.text("", TextLook::body(Tone::Muted));
                    ui.toggle("field", rolling, text::ON, text::OFF);
                });
            });
        });
    });
    None
}

/// Cells with the caption's point and words.
const CELLS: [(Cell, (f32, f32), &str); 7] = [
    (c1_strip, (20.0, 16.0), text::CAP_STRIP),
    (c2_coarse, (20.0, 76.0), text::CAP_COARSE),
    (c3_choice, (20.0, 136.0), text::CAP_CHOICE),
    (c4_details, (460.0, 136.0), text::CAP_DETAILS),
    (c5_world, (20.0, 214.0), text::CAP_FOLD),
    (c6_history, (20.0, 310.0), text::CAP_FOLD_HOVER),
    (c7_settings, (460.0, 214.0), text::CAP_FOLD_OPEN),
];

fn captions(ui: &mut Ui) {
    for (i, (_, p, words)) in CELLS.iter().enumerate() {
        ui.node(&format!("caption{i}"), absolute_at(Vec2::new(p.0, p.1)), |ui| ui.text(words, TextLook::status(Tone::Muted)));
    }
}

/// What the window shows: every cell live, sharing one state.
pub fn controls_live(ui: &mut Ui, s: &mut ControlsState) {
    for (cell, _, _) in CELLS {
        cell(ui, s);
    }
    captions(ui);
}

/// Every cell in its state, with the pointer where its caption says.
pub fn controls_sheet(theme: Theme) -> DrawList {
    let size = Vec2::new(CONTROLS.0 as f32, CONTROLS.1 as f32);
    let mut list = DrawList::default();
    let mut ui = Ui::new(theme);
    ui.begin_frame(size);
    captions(&mut ui);
    list.cmds.extend(ui.end_frame().cmds.iter().cloned());
    for (cell, _, _) in CELLS {
        let mut s = ControlsState::default();
        let mut ui = Ui::new(theme);
        ui.begin_frame(size);
        let id = cell(&mut ui, &mut s);
        ui.end_frame();
        if let Some(r) = id.and_then(|id| ui.rect(id)) {
            let c = Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0);
            ui.event(&Event::PointerMoved { pos: c });
        }
        ui.begin_frame(size);
        cell(&mut ui, &mut s);
        list.cmds.extend(ui.end_frame().cmds.iter().cloned());
    }
    list
}
