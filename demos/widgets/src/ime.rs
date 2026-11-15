//! The input method sheet: the text field while an input method composes in it, as a sheet. The system's
//! input method cannot be driven here, so each row replays its messages (`Ime::Start`, `Update`, `Commit`,
//! `End`) through `drive`, the way the backend would send them, and the picture shows the field's real state.

use nui_base::{DrawList, Ime, Vec2};
use nui_layout::Dim;
use nui_ui::{FieldOpts, Id, TextLook, Theme, Tone, Ui};

use crate::fields::{Ev, FieldsState, NONE, SCALE, SHIFT, drive, rep, setting_row};
use crate::{absolute_at, text};

pub const IME: (u32, u32) = (600, 260);

/// The captions' x, the fields' x and width, the first row's y and the distance between rows.
pub const CAP_X: f32 = 20.0;
pub const FIELD_X: f32 = 330.0;
pub const FIELD_W: f32 = 250.0;
pub const ROW_Y0: f32 = 20.0;
pub const ROW_STEP: f32 = 38.0;

fn start() -> Ev {
    Ev::Ime(Ime::Start)
}
fn update(text: &str, cursor: usize) -> Ev {
    Ev::Ime(Ime::Update { text: text.to_string(), cursor })
}
fn commit(text: &str) -> Ev {
    Ev::Ime(Ime::Commit { text: text.to_string() })
}

/// A text field with no placeholder and no limit, at (FIELD_X, y), editing the state's seed.
fn field_at(ui: &mut Ui, s: &mut FieldsState, y: f32) -> Id {
    let mut id = None;
    let opts = FieldOpts { width: Dim::Px(FIELD_W), ..FieldOpts::default() };
    ui.node("ime", absolute_at(Vec2::new(FIELD_X, y)), |ui| id = Some(ui.text_field("seed", &mut s.seed, opts).resp.id));
    id.expect("the node builds its field")
}

/// A `.setting` row whose number field ends where the text fields do: the field's own rect is what the
/// script clicks.
fn number_at(ui: &mut Ui, s: &mut FieldsState, y: f32) -> Id {
    let mut id = None;
    ui.node("ime-number", absolute_at(Vec2::new(FIELD_X - 80.0, y)), |ui| {
        id = Some(setting_row(ui, "row", Dim::Px(330.0), text::VERTICAL_SCALE, &mut s.values[0], SCALE));
    });
    id.expect("the row builds its field")
}

/// The rows of the sheet: seed text, steps (Click(30, 1) focuses the field with the caret at its end).
fn rows() -> Vec<Vec<Vec<Ev>>> {
    let click = || vec![Ev::Click(30.0, 1)];
    let mut over_selection = vec![Ev::Key(nui_base::Key::Home, NONE)];
    over_selection.extend(rep(nui_base::Key::Right, NONE, 2));
    over_selection.extend(rep(nui_base::Key::Right, SHIFT, 2));
    vec![
        vec![click(), vec![start(), update("にほんご", 4)]],
        vec![click(), vec![start(), update("日本語", 1)]],
        vec![click(), vec![start(), update("日本語", 3), commit("日本語"), Ev::Ime(Ime::End)]],
        vec![click(), vec![start(), update("にほ", 2)], vec![update("", 0), Ev::Ime(Ime::End)]],
        vec![click(), over_selection, vec![start(), update("かな", 2)]],
        vec![vec![Ev::Click(0.0, 1)], vec![start(), update("に", 1), commit("に"), Ev::Ime(Ime::End)]],
    ]
}

/// Every row in its state, driven by its messages, as one draw list.
pub fn ime_sheet(theme: Theme) -> DrawList {
    let size = Vec2::new(IME.0 as f32, IME.1 as f32);
    let mut list = DrawList::default();
    let mut ui = Ui::new(theme);
    ui.begin_frame(size);
    for (i, words) in text::CAP_IME.into_iter().enumerate() {
        let y = ROW_Y0 + ROW_STEP * i as f32;
        ui.node(&format!("cap{i}"), absolute_at(Vec2::new(CAP_X, y + 5.0)), |ui| ui.text(words, TextLook::status(Tone::Muted)));
    }
    list.cmds.extend(ui.end_frame().cmds.iter().cloned());

    for (i, steps) in rows().into_iter().enumerate() {
        let y = ROW_Y0 + ROW_STEP * i as f32;
        let mut state = FieldsState::default();
        if i < 5 {
            state.seed = if i == 4 { "abcdef" } else { "world " }.to_string();
            let build = move |ui: &mut Ui, s: &mut FieldsState| field_at(ui, s, y);
            list.cmds.extend(drive(theme, &mut state, &build, &steps).cmds);
        } else {
            let build = move |ui: &mut Ui, s: &mut FieldsState| number_at(ui, s, y);
            list.cmds.extend(drive(theme, &mut state, &build, &steps).cmds);
        }
    }
    list
}
