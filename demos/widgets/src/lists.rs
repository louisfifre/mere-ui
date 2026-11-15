//! The page's list rows, scroll area and history trend rows: live in a window, and as a sheet. One `Ui`
//! builds all three cells; the sheet replays a few frames of it (the gutter comes one frame late, the scroll
//! is set, the pointer moves) and keeps the last picture.
//!
//! Cell A is the biome list at the 1400 px column (330 wide), B a narrower column (262 wide)
//! with absent rows and nothing to scroll, C the history's trend rows.

use nui_base::{Cmd, Color, DrawList, Event, Vec2};
use nui_layout::{Dim, Style};
use nui_ui::{Id, ListRow, RowOpts, ScrollOpts, Theme, Ui, scroll_style};

use crate::{absolute_at, text};

pub const LISTS: (u32, u32) = (860, 520);

/// Where each cell's column starts.
pub const A_POS: Vec2 = Vec2::new(20.0, 20.0);
pub const B_POS: Vec2 = Vec2::new(440.0, 20.0);
pub const C_POS: Vec2 = Vec2::new(440.0, 230.0);
/// The widths of the columns: the page's `#listBlock` at 1400 px, a narrower one, and the history column
/// (PLACEHOLDER: the page's column is wider; 140 shows the ellipsis of "Typical flat zone").
pub const A_W: f32 = 330.0;
pub const B_W: f32 = 262.0;
pub const C_W: f32 = 140.0;
/// The scroll boxes' heights.
pub const A_H: f32 = 240.0;
pub const B_H: f32 = 160.0;
/// Top of A's scroll box: the column's top, the legend's 16.5 and the column's gap of 6.
pub const A_LIST_TOP: f32 = 20.0 + 16.5 + 6.0;
/// The rows of A: how many, how tall, which are pressed and which one the pointer ends on.
pub const A_ROWS: usize = 20;
pub const ROW_H: f32 = 24.0;
/// Rows 5 and 8 rather than the brief's 2 and 5: at 30 % the first three rows are scrolled out of sight.
pub const PRESSED: [usize; 2] = [5, 8];
pub const HOVER_ROW: usize = 6;
/// The rows of A whose names are too long for their column.
pub const LONG_ROWS: [usize; 2] = [4, 9];

/// What the application owns: the scroll offsets and the pressed rows.
#[derive(Clone, Debug)]
pub struct ListsState {
    pub y_a: f32,
    pub y_b: f32,
    /// A's range as the last frame measured it.
    pub max_a: f32,
    pub pressed: [bool; A_ROWS],
    /// The trend rows' plot slots, from the last build.
    pub plots: Vec<Id>,
}

impl Default for ListsState {
    fn default() -> ListsState {
        let mut pressed = [false; A_ROWS];
        for i in PRESSED {
            pressed[i] = true;
        }
        ListsState { y_a: 0.0, y_b: 0.0, max_a: 0.0, pressed, plots: Vec::new() }
    }
}

// ---- PLACEHOLDER data: demo names, colours and values, not the page's ----

/// PLACEHOLDER: biome names; the first is the brief's long one, rows 4 and 9 are made long on purpose.
pub const NAMES: [&str; A_ROWS] = [
    "Windswept gravelly hills",
    "Plains",
    "Sunflower plains",
    "Deep ocean",
    "Sparse jungle woodland edge",
    "Snowy taiga",
    "Dark forest",
    "Warm ocean",
    "Bamboo jungle",
    "Frozen deep ocean near shore",
    "Cherry grove",
    "Meadow",
    "Old growth pine taiga",
    "Stony shore",
    "Mangrove swamp",
    "Eroded badlands",
    "Jagged peaks",
    "Lush caves",
    "Dripstone caves",
    "Mushroom fields",
];
/// PLACEHOLDER.
const WIDTHS: [&str; 4] = ["350 blocks", "1,800 blocks", "50 blocks", "650 blocks"];
/// PLACEHOLDER.
const SHARES: [&str; 4] = ["1.0%", "0.4%", "7.5%", "22.1%"];
/// PLACEHOLDER: a palette of eight greens, blues and warm tones.
const SWATCHES: [u32; 8] = [0x6a9a4a, 0x3a6ea5, 0xc8b35a, 0x8a5a9a, 0xd0d0d0, 0x4a8a7a, 0xb0603a, 0x5a5a8a];

pub fn swatch(i: usize) -> Color {
    Color::hex(SWATCHES[i % SWATCHES.len()])
}

/// The width and the share of row `i` of A (PLACEHOLDER).
pub fn figures(i: usize) -> (&'static str, &'static str) {
    if i == 0 { ("1,250 blocks", "12.3%") } else { (WIDTHS[i % WIDTHS.len()], SHARES[i % SHARES.len()]) }
}

/// PLACEHOLDER: B's rows, with which are absent.
const B_ROWS: [(&str, bool); 5] =
    [("Plains", false), ("Savanna", false), ("Ice spikes", true), ("Desert", false), ("Badlands", true)];
/// PLACEHOLDER: the values; the page's would be a share or a width, whichever the figure is.
const C_VALUES: [&str; 5] = ["12.3%", "41.2%", "8.5%", "30.1%", "4.7%"];

fn column(pos: Vec2, w: f32, gap: f32) -> Style {
    Style { column: true, gap: Vec2::new(0.0, gap), w: Dim::Px(w), ..absolute_at(pos) }
}

/// What the window shows and the sheet drives: A with its offset, pressed rows and scrollbar, B, and C.
pub fn lists_live(ui: &mut Ui, s: &mut ListsState) {
    let ListsState { y_a, y_b, max_a, pressed, plots } = s;
    plots.clear();
    ui.node("a", column(A_POS, A_W, 6.0), |ui| {
        ui.list_head("head", text::COL_BIOME, text::COL_WIDTH, text::COL_SHARE);
        // basis Auto: the page's `flex: 1` would win over the height in a column.
        let style = Style { h: Dim::Px(A_H), grow: 0.0, basis: Dim::Auto, ..scroll_style() };
        let r = ui.scroll("list", y_a, style, ScrollOpts::LIST, |ui| {
            for i in 0..A_ROWS {
                let (width, share) = figures(i);
                let row = ListRow { swatch: swatch(i), name: NAMES[i], width, share };
                let opts = RowOpts { pressed: pressed[i], choosing: true, ..RowOpts::default() };
                if ui.list_row(&format!("row{i}"), &row, opts).clicked {
                    pressed[i] = !pressed[i];
                }
            }
        });
        *max_a = r.max;
    });
    ui.node("b", column(B_POS, B_W, 6.0), |ui| {
        ui.list_head("head", text::COL_BIOME, text::COL_WIDTH, text::COL_SHARE);
        let style = Style { h: Dim::Px(B_H), grow: 0.0, basis: Dim::Auto, ..scroll_style() };
        ui.scroll("list", y_b, style, ScrollOpts::LIST, |ui| {
            for (i, (name, absent)) in B_ROWS.into_iter().enumerate() {
                let (width, share) = figures(i + 1);
                let row = ListRow { swatch: swatch(i + 1), name, width, share };
                ui.list_row(&format!("row{i}"), &row, RowOpts { absent, ..RowOpts::default() });
            }
        });
    });
    ui.node("c", column(C_POS, C_W, 8.0), |ui| {
        for (i, (name, value)) in text::FIGURE_NAMES.into_iter().zip(C_VALUES).enumerate() {
            // A small slope per row, in the slot's own coordinates, as the history draws its lines.
            let y = |k: f32| 4.0 + ((i as f32 * 5.0 + k * 9.0) % 22.0);
            let plot = vec![Cmd::Polyline {
                points: vec![Vec2::new(0.0, y(0.0)), Vec2::new(46.0, y(1.0)), Vec2::new(93.0, y(2.0)), Vec2::new(139.0, y(3.0))],
                width: 1.5,
                color: ui.theme().text,
            }];
            plots.push(ui.trend_row(&format!("t{i}"), name, value, plot));
        }
    });
}

/// A hairline round each plot slot, so that the sheet shows where the 30 px slots are. Drawn over the Ui's list.
pub fn lists_marks(ui: &Ui, s: &ListsState, theme: Theme) -> DrawList {
    let mut list = DrawList::default();
    for id in &s.plots {
        if let Some(r) = ui.rect(*id) {
            list.frame(r, 1.0, theme.line);
        }
    }
    list
}

/// A frame of the live build, as the sheet drives it.
fn frame(ui: &mut Ui, s: &mut ListsState) -> DrawList {
    ui.begin_frame(Vec2::new(LISTS.0 as f32, LISTS.1 as f32));
    lists_live(ui, s);
    ui.end_frame().clone()
}

/// Where the pointer goes to hover row `HOVER_ROW` of A in the sheet.
pub fn hover_point(y_a: f32) -> Vec2 {
    Vec2::new(A_POS.x + 100.0, A_LIST_TOP + ROW_H * HOVER_ROW as f32 - y_a + ROW_H / 2.0)
}

/// The scroll's offset in the sheet: 30 % of the range, to a whole pixel so that the underline of a hovered
/// name (a 1 px line at a fractional height otherwise) falls on one pixel row.
pub fn sheet_offset(max: f32) -> f32 {
    (0.3 * max).round()
}

/// The three cells in four frames: 1, the gutter comes with 2, the scroll is set for 3, the pointer comes
/// over a row for 4. The last picture is the sheet.
pub fn lists_sheet(theme: Theme) -> DrawList {
    let mut ui = Ui::new(theme);
    let mut s = ListsState::default();
    frame(&mut ui, &mut s);
    frame(&mut ui, &mut s);
    s.y_a = sheet_offset(s.max_a);
    frame(&mut ui, &mut s);
    ui.event(&Event::PointerMoved { pos: hover_point(s.y_a) });
    let mut list = frame(&mut ui, &mut s);
    list.cmds.extend(lists_marks(&ui, &s, theme).cmds);
    list
}
