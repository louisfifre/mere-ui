//! The page's list rows: the biome row and its legend (preview.html lines 318-325, 337-338, 346-354,
//! 360-363) and the history trend row (lines 310-313). A row is its own grid, as the page's `.biome` is, so
//! the width column of one row does not line up with the next one's: only the right edges do.

use nui_base::{Cmd, Color, Cursor, Vec2};
use nui_layout::{Align, Dim, Display, Edges, Justify, Style, Track, Tracks};

use crate::id::Id;
use crate::text::{TextLook, Tone};
use crate::theme::mix;
use crate::ui::{Paint, Resp, Ui};

pub struct ListRow<'a> {
    pub swatch: Color,
    pub name: &'a str,
    /// The typical width, as the app's catalog formats it ("1,250 blocks").
    pub width: &'a str,
    /// The share, as formatted ("12.3%").
    pub share: &'a str,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RowOpts {
    /// `aria-pressed=true`: the row is one of the chosen.
    pub pressed: bool,
    /// The list is choosing: the rows that are not pressed fade (line 349).
    pub choosing: bool,
    /// `.absent` (lines 337-338): muted, half-opaque swatch, and no width or share.
    pub absent: bool,
    /// `i.mark` (line 348): the swatch is a round mark, as a structure's on the map.
    pub mark: bool,
}

/// What a part head of the list needs (`.part`, lines 328-335).
#[derive(Copy, Clone, Debug, Default)]
pub struct PartOpts {
    /// `aria-pressed=true`: the part is the chosen one, so its head takes the text colour.
    pub pressed: bool,
    /// `:first-child`: 2 px above instead of 12.
    pub first: bool,
    /// A button (a part of structures) or a plain note (the absent biomes).
    pub button: bool,
}

/// The swatch column and the gap between a row's columns, and the least widths of the width and share columns (8 ch
/// and 6 ch of 12 px). A panel that holds list rows reads them to work out its natural width.
pub const SWATCH_W: f32 = 12.0;
pub const ROW_GAP: f32 = 8.0;
pub const WIDTH_MIN: f32 = 57.6;
pub const SHARE_MIN: f32 = 43.2;

/// `grid-template-columns: 12px minmax(0, 1fr) auto auto` (line 318).
fn cols() -> Tracks {
    Tracks::new(&[Track::Px(SWATCH_W), Track::MinMaxFr0(1.0), Track::Auto, Track::Auto])
}

fn grid(pad: Edges) -> Style {
    Style { display: Display::Grid, cols: cols(), gap: Vec2::new(ROW_GAP, 0.0), align_items: Align::Center, pad, ..Style::default() }
}

fn end_cell(min_w: f32) -> Style {
    Style { min_w: Dim::Px(min_w), justify: Justify::End, ..Style::default() }
}

impl Ui {
    /// One `.biome` row. `Resp.clicked` is false for an absent row, which the page does not make a button.
    pub fn list_row(&mut self, key: &str, row: &ListRow, opts: RowOpts) -> Resp {
        let id = self.key(key);
        // An absent row is no button: it keeps the arrow.
        let mut resp = self.interact(id, false, if opts.absent { Cursor::Arrow } else { Cursor::Hand });
        if opts.absent {
            resp.clicked = false;
        }
        let t = *self.theme();
        // Line 349: `aria-pressed=false` rows fade while the list is choosing; hovered ones less (line 350).
        let o = if opts.choosing && !opts.pressed {
            if resp.hovered { 0.85 } else { 0.5 }
        } else {
            1.0
        };
        let swatch_o = o * if opts.absent { 0.5 } else { 1.0 };
        let body = TextLook::body(Tone::Text);
        let name_color = t.faded(if opts.absent { t.muted } else { t.text }, o);

        self.open(key, id, grid(Edges::vh(3.0, 0.0)), Paint::None);
        // The page's swatch has `box-shadow: inset 0 0 0 1px rgb(128 128 128 / .35)`: a 1 px frame of that
        // grey over the fill.
        let fill = t.faded(row.swatch, swatch_o);
        let frame = t.faded(mix(Color::hex(0x808080), 0.35, row.swatch), swatch_o);
        let swatch = Style { w: Dim::Px(SWATCH_W), h: Dim::Px(SWATCH_W), ..Style::default() };
        if opts.mark {
            // `border-radius: 50%` on 10 px with a 1 px outside shadow: a disc of radius 5 and a ring spanning 5 to 6.
            let shadow = Color::rgb8(0, 0, 0).with_alpha(0.55 * swatch_o);
            let cmds = vec![
                Cmd::Disc { center: Vec2::new(6.0, 6.0), radius: 5.0, color: fill },
                Cmd::Ring { center: Vec2::new(6.0, 6.0), radius: 5.5, width: 1.0, color: shadow },
            ];
            self.add("swatch", swatch, Paint::Cmds(cmds));
        } else {
            self.add("swatch", swatch, Paint::Box { fill: Some(fill), frame: Some(frame) });
        }
        // Hover underlines the name (line 346).
        self.add_ellipsis(row.name, body, name_color, resp.hovered && !opts.absent);
        if !opts.absent {
            // `.width` and `.share` (lines 323-325): min 8ch and 6ch of 12 px, right-aligned.
            self.cell("width", end_cell(WIDTH_MIN), row.width, body, t.faded(t.muted, o));
            self.cell("share", end_cell(SHARE_MIN), row.share, body, t.faded(t.text, o));
        }
        self.close();
        resp
    }

    /// `.part` (lines 328-335): the line over a part of the list, in the rows' columns, 11 px and muted (the
    /// text colour when pressed). With `button` a hover underlines the name and a click is reported; without
    /// it the head is only a note.
    pub fn list_part(&mut self, key: &str, name: &str, share: &str, opts: PartOpts) -> Resp {
        let id = self.key(key);
        let mut resp = self.interact(id, !opts.button, Cursor::Hand);
        if !opts.button {
            resp.clicked = false;
        }
        let t = *self.theme();
        let look = TextLook { size: 11.0, line: 16.5, ..TextLook::body(Tone::Text) };
        let color = if opts.pressed { t.text } else { t.muted };
        let style = Style {
            align_items: Align::Baseline,
            pad: Edges::new(if opts.first { 2.0 } else { 12.0 }, 0.0, 4.0, 0.0),
            ..grid(Edges::default())
        };
        self.open(key, id, style, Paint::None);
        self.add("pad", Style { w: Dim::Px(12.0), ..Style::default() }, Paint::None);
        self.add_ellipsis(name, look, color, opts.button && resp.hovered);
        self.add("gap", Style::default(), Paint::None);
        self.cell("share", end_cell(39.6), share, look, color);
        self.close();
        resp
    }

    /// `.legend` (lines 352-354): the same columns at 11 px, muted. `min-width` is in 11 px `ch`, so the
    /// cells are narrower than the rows' (52.8 and 39.6 against 57.6 and 43.2); the right edges still meet
    /// because the cells are right-aligned. Returns the ids of the width and share cells, which may carry a title.
    pub fn list_head(&mut self, key: &str, column: &str, width: &str, share: &str) -> [Id; 2] {
        let id = self.key(key);
        let t = *self.theme();
        let look = TextLook { size: 11.0, line: 16.5, ..TextLook::body(Tone::Muted) };
        self.open(key, id, grid(Edges::default()), Paint::None);
        self.add("pad", Style { w: Dim::Px(12.0), ..Style::default() }, Paint::None);
        self.add_text(column, look, t.muted, false, Style::default());
        let w = self.cell("width", end_cell(52.8), width, look, t.muted);
        let s = self.cell("share", end_cell(39.6), share, look, t.muted);
        self.close();
        [w, s]
    }

    /// One `.trend` row of the history (lines 310-313): a name that ends in an ellipsis, the value, and a
    /// 30 px slot for the plot under both. `plot` is drawn in the slot's own coordinates and not clipped (the
    /// svg's `overflow: visible`). Returns the slot's id. The `#history` column (gap 8) is the caller's node.
    pub fn trend_row(&mut self, key: &str, name: &str, value: &str, plot: Vec<Cmd>) -> Id {
        let id = self.key(key);
        let t = *self.theme();
        let style = Style {
            display: Display::Grid,
            cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto]),
            gap: Vec2::new(10.0, 0.0),
            align_items: Align::Baseline,
            ..Style::default()
        };
        self.open(key, id, style, Paint::None);
        let body = TextLook::body(Tone::Text);
        self.add_ellipsis(name, body, t.text, false);
        self.add_text(value, body, t.text, false, Style::default());
        let slot = self.key("plot");
        self.open("plot", slot, Style { col_span_all: true, h: Dim::Px(30.0), ..Style::default() }, Paint::Cmds(plot));
        self.close();
        self.close();
        slot
    }

    /// A box with one word in it, for the cells that align their word to the right.
    fn cell(&mut self, name: &str, style: Style, text: &str, look: TextLook, color: Color) -> Id {
        let id = self.key(name);
        self.open(name, id, style, Paint::None);
        self.add_text(text, look, color, false, Style::default());
        self.close();
        id
    }
}
