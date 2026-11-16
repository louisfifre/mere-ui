//! The Design System's cards: each component alone in its variants, the same picture as the Design System's
//! preview of that component (`components/<Name>/preview.html`), so the code's pictures and the design match.
//!
//! A card has no caption, no label and no title. It is a list of blocks (`NUI.matrix`, `NUI.block`) stacked
//! `SPACE_96` apart inside a `SPACE_64` padding; a matrix is a grid whose columns are as wide as their widest
//! cell, `SPACE_48` apart in both directions, each cell centred vertically and starting at its column's left
//! edge. The picture is exactly the content plus the padding.
//!
//! A card module declares its blocks:
//!
//! ```ignore
//! pub fn blocks() -> Vec<Block> {
//!     vec![
//!         Block::matrix(vec![vec![Cell::new(State::Rest, |ui, state| { /* build one widget, return its Id */ })]]),
//!         Block::single(Cell::new(State::Rest, |ui, _| { /* ... */ })),
//!     ]
//! }
//! pub fn card(theme: Theme) -> Card { render(theme, blocks()) }
//! ```
//!
//! One pointer can hover one widget, so each cell is built in its own `Ui`: the harness lays the cell out,
//! puts the pointer on the centre of the widget the builder returned (hover; held adds the button press) and
//! builds the cell again. The builder is told its state so that it can focus a field (`State::Focus`) or
//! disable itself; the pointer is the harness's business.

use nui_base::{Button, DrawList, Event, Vec2};
use nui_layout::{Dim, Insets, Position, Style};
use nui_ui::tokens::{SPACE_48, SPACE_64, SPACE_96};
use nui_ui::{Id, Theme, Ui};

pub mod button;
pub mod cover;
pub mod detail_tabs;
pub mod fold;
pub mod hover_box;
pub mod icon_button;
pub mod list_heads;
pub mod list_row;
pub mod number_field;
pub mod scroll_area;
pub mod select;
pub mod stop_button;
pub mod tab_menu;
pub mod tabs;
pub mod text;
pub mod text_field;
pub mod toggle;
pub mod tool_button;
pub mod tooltip;
pub mod trend_row;

/// A card as a draw list and its size in pixels.
pub struct Card {
    pub list: DrawList,
    pub size: (u32, u32),
}

/// What the pointer (and the focus) does to a cell's widget.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    Rest,
    Hover,
    Held,
    /// The builder focuses its field; the harness does nothing more.
    Focus,
}

type Build = Box<dyn Fn(&mut Ui, State) -> Id>;

/// One cell: a builder that makes one widget (or a small group) and answers the id the pointer rests on.
pub struct Cell {
    pub state: State,
    build: Build,
}

impl Cell {
    pub fn new(state: State, build: impl Fn(&mut Ui, State) -> Id + 'static) -> Cell {
        Cell { state, build: Box::new(build) }
    }
}

/// `NUI.matrix` (or `NUI.block`, a matrix of one cell): rows of cells, `width` a column's fixed width when
/// the preview gives one (`o.col`), else each column is as wide as its widest cell.
pub struct Block {
    rows: Vec<Vec<Option<Cell>>>,
    pub width: Option<f32>,
}

impl Block {
    pub fn matrix(rows: Vec<Vec<Cell>>) -> Block {
        Block { rows: rows.into_iter().map(|r| r.into_iter().map(Some).collect()).collect(), width: None }
    }
    /// A matrix whose cells may be left empty (`null` in the preview).
    pub fn sparse(rows: Vec<Vec<Option<Cell>>>) -> Block {
        Block { rows, width: None }
    }
    pub fn single(cell: Cell) -> Block {
        Block::matrix(vec![vec![cell]])
    }
    pub fn col_width(mut self, w: f32) -> Block {
        self.width = Some(w);
        self
    }
}

const FRAME: Vec2 = Vec2 { x: 4096.0, y: 4096.0 };

fn at(p: Vec2, w: Option<f32>) -> Style {
    Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(p.x), top: Dim::Px(p.y), right: Dim::Auto, bottom: Dim::Auto },
        w: w.map_or(Dim::Auto, Dim::Px),
        ..Style::default()
    }
}

/// One frame of a cell at `p`: the cell's own box and the widget's id.
fn build(ui: &mut Ui, cell: &Cell, p: Vec2, w: Option<f32>) -> (Id, Id) {
    ui.begin_frame(FRAME);
    let mut target = None;
    let root = ui.node("cell", at(p, w), |ui| target = Some((cell.build)(ui, cell.state)));
    ui.end_frame();
    (root, target.expect("a cell builds one widget"))
}

fn size_of(theme: Theme, cell: &Cell, w: Option<f32>) -> Vec2 {
    let mut ui = Ui::new(theme);
    // Two frames: a widget's size can depend on the last frame (a scrollbar's gutter).
    build(&mut ui, cell, Vec2::new(0.0, 0.0), w);
    let (root, _) = build(&mut ui, cell, Vec2::new(0.0, 0.0), w);
    let r = ui.rect(root).expect("the cell was laid out");
    Vec2::new(r.w, r.h)
}

fn paint(theme: Theme, cell: &Cell, p: Vec2, w: Option<f32>, out: &mut DrawList) {
    let mut ui = Ui::new(theme);
    let (_, target) = build(&mut ui, cell, p, w);
    if cell.state != State::Focus {
        build(&mut ui, cell, p, w);
    }
    let r = ui.rect(target).expect("the widget was laid out");
    let c = Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0);
    match cell.state {
        State::Rest | State::Focus => {}
        State::Hover => {
            ui.event(&Event::PointerMoved { pos: c });
        }
        State::Held => {
            ui.event(&Event::PointerMoved { pos: c });
            ui.event(&Event::PointerDown { pos: c, button: Button::Left, clicks: 1 });
        }
    }
    ui.begin_frame(FRAME);
    ui.node("cell", at(p, w), |ui| {
        (cell.build)(ui, cell.state);
    });
    out.cmds.extend(ui.end_frame().cmds.iter().cloned());
}

/// Lays the blocks out and draws them. Sizes are rounded up to whole pixels so that columns and rows start
/// on the pixel grid.
pub fn render(theme: Theme, blocks: Vec<Block>) -> Card {
    let mut sized = Vec::new();
    let mut inner_w: f32 = 0.0;
    let mut inner_h: f32 = 0.0;
    for (bi, b) in blocks.iter().enumerate() {
        let n_cols = b.rows.iter().map(Vec::len).max().unwrap_or(0);
        let mut cols = vec![0.0f32; n_cols];
        let mut rows = vec![0.0f32; b.rows.len()];
        let mut sizes = Vec::new();
        for (ri, row) in b.rows.iter().enumerate() {
            for (ci, c) in row.iter().enumerate() {
                let s = c.as_ref().map_or(Vec2::new(0.0, 0.0), |c| size_of(theme, c, b.width));
                cols[ci] = cols[ci].max(b.width.unwrap_or(s.x).ceil());
                rows[ri] = rows[ri].max(s.y.ceil());
                sizes.push(s);
            }
        }
        let w: f32 = cols.iter().sum::<f32>() + SPACE_48 * n_cols.saturating_sub(1) as f32;
        let h: f32 = rows.iter().sum::<f32>() + SPACE_48 * rows.len().saturating_sub(1) as f32;
        inner_w = inner_w.max(w);
        inner_h += h + if bi > 0 { SPACE_96 } else { 0.0 };
        sized.push((cols, rows, sizes));
    }
    let size = ((inner_w + 2.0 * SPACE_64).ceil() as u32, (inner_h + 2.0 * SPACE_64).ceil() as u32);
    let mut list = DrawList::default();
    let mut y = SPACE_64;
    for (b, (cols, rows, sizes)) in blocks.iter().zip(&sized) {
        let mut k = 0;
        let mut ry = y;
        for (ri, row) in b.rows.iter().enumerate() {
            let mut cx = SPACE_64;
            for (ci, c) in row.iter().enumerate() {
                if let Some(c) = c {
                    let dy = ((rows[ri] - sizes[k].y) / 2.0).floor();
                    paint(theme, c, Vec2::new(cx, ry + dy), b.width, &mut list);
                }
                k += 1;
                cx += cols[ci] + SPACE_48;
            }
            ry += rows[ri] + SPACE_48;
        }
        y += rows.iter().sum::<f32>() + SPACE_48 * rows.len().saturating_sub(1) as f32 + SPACE_96;
    }
    Card { list, size }
}

/// Every card, by its kebab-case name.
pub const CARDS: &[(&str, fn(Theme) -> Card)] = &[
    ("button", button::card),
    ("cover", cover::card),
    ("detail-tabs", detail_tabs::card),
    ("fold", fold::card),
    ("hover-box", hover_box::card),
    ("icon-button", icon_button::card),
    ("list-heads", list_heads::card),
    ("list-row", list_row::card),
    ("number-field", number_field::card),
    ("scroll-area", scroll_area::card),
    ("select", select::card),
    ("stop-button", stop_button::card),
    ("tab-menu", tab_menu::card),
    ("tabs", tabs::card),
    ("text", text::card),
    ("text-field", text_field::card),
    ("toggle", toggle::card),
    ("tool-button", tool_button::card),
    ("tooltip", tooltip::card),
    ("trend-row", trend_row::card),
];

pub fn find(name: &str) -> Option<fn(Theme) -> Card> {
    CARDS.iter().find(|(n, _)| *n == name).map(|(_, f)| *f)
}
