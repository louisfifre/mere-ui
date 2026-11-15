//! The widgets of `nui-ui` in the page's looks: live in a window, and as a sheet of every state.

pub mod controls;
pub mod fields;
pub mod ime;
pub mod lists;
pub mod text;

pub use controls::{CONTROLS, ControlsState, controls_live, controls_sheet};
pub use fields::{FIELDS, FieldsState, fields_live, fields_sheet};
pub use ime::{IME, ime_sheet};
pub use lists::{LISTS, ListsState, lists_live, lists_marks, lists_sheet};

use nui_base::{DrawList, Event, Vec2};
use nui_layout::{Align, Dim, Display, Edges, Insets, Position, Style, Track, Tracks};
use nui_ui::{ButtonLook, ButtonOpts, Id, TextLook, Theme, Tone, Ui};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Sample {
    Primary,
    Plain,
    Link,
    ToggleOff,
    ToggleOn,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Normal,
    Hover,
    Held,
    Disabled,
}

pub const SHEET: (u32, u32) = (860, 330);
/// Left edge of the four columns: normal, hover, held, disabled.
pub const COLS: [f32; 4] = [20.0, 230.0, 440.0, 650.0];

const SAMPLES: [Sample; 5] = [Sample::Primary, Sample::Plain, Sample::Link, Sample::ToggleOff, Sample::ToggleOn];

fn row(s: Sample) -> usize {
    SAMPLES.iter().position(|x| *x == s).unwrap()
}

pub fn at(s: Sample, col: usize) -> Vec2 {
    let y = match s {
        Sample::Primary => 44.0,
        Sample::Plain => 98.0,
        Sample::Link => 132.0,
        Sample::ToggleOff => 166.0,
        Sample::ToggleOn => 204.0,
    };
    Vec2::new(COLS[col], y)
}

/// `.setting` (line 164): label, output, field; the field column is `8ch`.
pub(crate) fn setting_style(w: Dim) -> Style {
    Style {
        display: Display::Grid,
        cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Px(57.6)]),
        gap: Vec2::new(10.0, 0.0),
        align_items: Align::Center,
        pad: Edges::vh(2.0, 0.0),
        w,
        ..Style::default()
    }
}

fn absolute_at(p: Vec2) -> Style {
    Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(p.x), top: Dim::Px(p.y), right: Dim::Auto, bottom: Dim::Auto },
        ..Style::default()
    }
}

/// One sample at its cell. Returns the widget's own id (the button's or the toggle's).
pub fn sample(ui: &mut Ui, s: Sample, col: usize, disabled: bool, on: &mut bool) -> Id {
    let key = format!("s{}-{col}", row(s));
    let mut id = None;
    ui.node(&key, absolute_at(at(s, col)), |ui| match s {
        Sample::Primary => {
            // The `.go` row, line 209.
            let go = Style { w: Dim::Px(180.0), ..Style::default() };
            ui.node("go", go, |ui| {
                id = Some(ui.button_with("generate", text::GENERATE, ButtonLook::Primary, ButtonOpts { disabled, grow: 1.0 }).id);
            });
        }
        Sample::Plain => id = Some(ui.button_with("detail", text::DETAIL, ButtonLook::Plain, ButtonOpts { disabled, grow: 0.0 }).id),
        Sample::Link => id = Some(ui.button_with("export", text::EXPORT, ButtonLook::Link, ButtonOpts { disabled, grow: 0.0 }).id),
        Sample::ToggleOff | Sample::ToggleOn => {
            // A `.setting` row (line 164): label, output, field.
            let setting = Style {
                display: Display::Grid,
                cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Px(57.6)]),
                gap: Vec2::new(10.0, 0.0),
                align_items: Align::Center,
                pad: Edges::vh(2.0, 0.0),
                w: Dim::Px(180.0),
                ..Style::default()
            };
            ui.node("setting", setting, |ui| {
                ui.label(text::ISLANDS, Tone::Text);
                ui.text("", TextLook::body(Tone::Muted));
                id = Some(ui.toggle("field", on, text::ON, text::OFF).id);
            });
        }
    });
    id.expect("every sample builds one widget")
}

/// The column captions and the three labels, each placed absolutely.
pub fn captions_and_labels(ui: &mut Ui) {
    for (i, caption) in [text::NORMAL, text::HOVER, text::HELD, text::DISABLED].into_iter().enumerate() {
        ui.node(&format!("caption{i}"), absolute_at(Vec2::new(COLS[i], 16.0)), |ui| ui.text(caption, TextLook::status(Tone::Muted)));
    }
    ui.node("seed", absolute_at(Vec2::new(20.0, 256.0)), |ui| ui.label(text::SEED, Tone::Text));
    ui.node("up-to-date", absolute_at(Vec2::new(20.0, 280.0)), |ui| ui.text(text::UP_TO_DATE, TextLook::status(Tone::Muted)));
    ui.node("failed", absolute_at(Vec2::new(20.0, 300.0)), |ui| ui.text(text::EXPORT_FAILED, TextLook::status(Tone::Warn)));
}

/// What the window shows: every sample live in column 0, and two disabled ones in column 3.
pub fn live(ui: &mut Ui, ons: &mut [bool; 2]) {
    let mut unused = false;
    for s in SAMPLES {
        let on = match s {
            Sample::ToggleOff => &mut ons[0],
            Sample::ToggleOn => &mut ons[1],
            _ => &mut unused,
        };
        sample(ui, s, 0, false, on);
    }
    for s in [Sample::Primary, Sample::Plain] {
        sample(ui, s, 3, true, &mut unused);
    }
    captions_and_labels(ui);
}

/// Every sample in every state, as one draw list. One pointer can hover one widget, so each cell is its
/// own `Ui` with only that sample, and the harness injects the pointer by events.
pub fn sheet(theme: Theme) -> DrawList {
    let size = Vec2::new(SHEET.0 as f32, SHEET.1 as f32);
    let mut list = DrawList::default();
    let mut ui = Ui::new(theme);
    ui.begin_frame(size);
    captions_and_labels(&mut ui);
    list.cmds.extend(ui.end_frame().cmds.iter().cloned());

    let mut cells = Vec::new();
    for s in SAMPLES {
        for (col, state) in [State::Normal, State::Hover, State::Held].into_iter().enumerate() {
            cells.push((s, col, state));
        }
    }
    cells.push((Sample::Primary, 3, State::Disabled));
    cells.push((Sample::Plain, 3, State::Disabled));

    for (s, col, state) in cells {
        let mut ui = Ui::new(theme);
        let mut on = s == Sample::ToggleOn;
        let disabled = state == State::Disabled;
        ui.begin_frame(size);
        let id = sample(&mut ui, s, col, disabled, &mut on);
        ui.end_frame();
        let r = ui.rect(id).expect("the widget was laid out");
        let c = Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0);
        match state {
            State::Normal => {}
            State::Hover | State::Disabled => {
                ui.event(&Event::PointerMoved { pos: c });
            }
            State::Held => {
                ui.event(&Event::PointerMoved { pos: c });
                ui.event(&Event::PointerDown { pos: c, button: nui_base::Button::Left, clicks: 1 });
            }
        }
        ui.begin_frame(size);
        sample(&mut ui, s, col, disabled, &mut on);
        list.cmds.extend(ui.end_frame().cmds.iter().cloned());
    }
    list
}
