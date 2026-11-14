//! Build plan step 8's picture: the page's grid rows and absolute boxes that the shell leaves out, each laid
//! out by `nui-layout` from the numbers of `world-preview/page/preview.html`, in one 718 x 700 sheet with
//! the left column (320 px) and the right column (330 px) of the 1400 px layout side by side.
//!
//! Every number from the page cites its CSS line. A value the page does not fix (figures, durations, the
//! stage height, the cut line's top, the biome names) is marked PLACEHOLDER. Text is IBM Plex Mono: advance
//! 0.6 em (600/1000, all weights), line 1.5 em (body `line-height: 1.5`), baseline 1.125 em (ascent 1.025 +
//! half the line gap, descent 0.275: UNVERIFIED against the font's tables).
//!
//! A padding below is the CSS padding with the 1 px border already counted where the page has one; the
//! numbers are the ones the build brief derived (24 px fields, 22 px inputs), so none is added twice.

use nui_base::Vec2;
use nui_layout::{
    Align, Avail, Dim, Display, Edges, Insets, Margin, Measured, NodeId, Overflow, Position, Side, Style, Track, Tracks, Tree,
};

/// Window size of the sheet: 20 + 320 + 28 + 330 + 20 wide.
pub const GALLERY: (f32, f32) = (718.0, 700.0);

/// A `.setting` row (line 164) and what a test reads in it.
pub struct SettingRow {
    pub row: NodeId,
    pub label: NodeId,
    pub output: NodeId,
    pub field: NodeId,
    pub hint: Option<NodeId>,
}

/// An `.ore-row` (line 179).
pub struct OreRow {
    pub row: NodeId,
    pub swatch: NodeId,
    pub name: NodeId,
    pub inputs: [NodeId; 3],
    pub button: NodeId,
}

/// A `.biome` row (line 319).
pub struct BiomeRow {
    pub row: NodeId,
    pub width: NodeId,
    pub share: NodeId,
}

/// The nodes a test wants to look at by name.
pub struct Gallery {
    pub tree: Tree,
    pub root: NodeId,
    /// The seven cells of the sheet, A to G.
    pub cells: [NodeId; 7],
    pub settings: Vec<SettingRow>,
    pub figures: NodeId,
    /// Name, value and before-value of the first figures row.
    pub figure_row: [NodeId; 3],
    pub rule: NodeId,
    pub ore_head: NodeId,
    pub ore_tries: NodeId,
    pub ore_rows: Vec<OreRow>,
    pub biome_rows: Vec<BiomeRow>,
    pub run: NodeId,
    pub details: NodeId,
    pub choices: [NodeId; 3],
    pub generate: NodeId,
    pub stop_run: NodeId,
    pub icon: NodeId,
    pub stage: NodeId,
    pub frame: NodeId,
    pub flat: NodeId,
    pub cut_line: NodeId,
    pub note: NodeId,
    pub note_text: NodeId,
    pub under: NodeId,
    pub scroll: NodeId,
    pub scroll_rows: Vec<NodeId>,
}

fn add(t: &mut Tree, parent: NodeId, name: &str, style: Style) -> NodeId {
    let n = t.new_node(style);
    t.set_name(n, name);
    t.append(parent, n);
    n
}

/// A text leaf named by its own text. `wrap` breaks greedily by words at the room offered; otherwise the text
/// stays on one line. An empty text is 0 x 0 with no baseline (an empty grid cell).
fn mono(t: &mut Tree, parent: NodeId, text: &str, size: f32, wrap: bool, style: Style) -> NodeId {
    let n = add(t, parent, text, style);
    let text = text.to_string();
    t.set_leaf(
        n,
        Box::new(move |a: Avail| {
            if text.is_empty() {
                return Measured::default();
            }
            let adv = 0.6 * size;
            let cols = match a.w {
                Some(av) if wrap => ((av / adv + 1e-3).floor() as usize).max(1),
                _ => usize::MAX,
            };
            let (mut lines, mut cur, mut longest) = (1usize, 0usize, 0usize);
            for word in text.split(' ') {
                let len = word.chars().count();
                if cur > 0 && cur + 1 + len > cols {
                    lines += 1;
                    cur = len;
                } else {
                    cur += if cur > 0 { 1 } else { 0 } + len;
                }
                longest = longest.max(cur);
            }
            Measured { size: Vec2::new(longest as f32 * adv, lines as f32 * 1.5 * size), baseline: 1.125 * size }
        }),
    );
    n
}

fn grid(cols: &[Track], gap: Vec2, align: Align) -> Style {
    Style { display: Display::Grid, cols: Tracks::new(cols), gap, align_items: align, ..Style::default() }
}

fn column(gap: f32) -> Style {
    Style { column: true, gap: Vec2::new(0.0, gap), ..Style::default() }
}

/// `.setting` (line 164): `grid-template-columns: minmax(0,1fr) auto 8ch; column-gap: 10px;
/// align-items: center; padding: 2px 0`. 8ch = 8 x 7.2 = 57.6. The field (line 123) is 24 high.
fn setting(t: &mut Tree, parent: NodeId, label: &str, output: &str, field: &str, hint: Option<&str>) -> SettingRow {
    let cols = [Track::MinMaxFr0(1.0), Track::Auto, Track::Px(57.6)];
    let row = add(t, parent, "setting", Style { pad: Edges::vh(2.0, 0.0), ..grid(&cols, Vec2::new(10.0, 0.0), Align::Center) });
    let label = mono(t, row, label, 12.0, true, Style::default());
    let output = mono(t, row, output, 12.0, false, Style::default());
    let field = mono(t, row, field, 12.0, false, Style { pad: Edges::new(3.0, 7.0, 3.0, 7.0), ..Style::default() });
    // `small` (line 171): `grid-column: 1 / -1; padding: 2px 0 4px; font-size: 11px`, shown on the focused row.
    let hint = hint.map(|h| mono(t, row, h, 11.0, true, Style { col_span_all: true, pad: Edges::new(2.0, 0.0, 4.0, 0.0), ..Style::default() }));
    SettingRow { row, label, output, field, hint }
}

/// One row of `#figures` (line 297): a name, a value and a before value, in the given sizes.
fn figure_row(t: &mut Tree, parent: NodeId, name: &str, value: &str, before: &str, size: f32) -> [NodeId; 3] {
    [
        mono(t, parent, name, size, true, Style::default()),
        mono(t, parent, value, size, false, Style::default()),
        mono(t, parent, before, 12.0, false, Style::default()),
    ]
}

/// An `.ore-row` (line 179), or the head row (`size` 11: its `ch` is 6.6, so its columns are narrower).
/// `ch` is the width of a character at the row's size.
fn ore_row_style(ch: f32) -> Style {
    let cols = [Track::Px(12.0), Track::MinMaxFr0(1.0), Track::Px(6.0 * ch), Track::Px(5.0 * ch), Track::Px(5.0 * ch), Track::Px(14.0)];
    Style { pad: Edges::vh(2.0, 0.0), ..grid(&cols, Vec2::new(10.0, 0.0), Align::Baseline) }
}

fn ore_row(t: &mut Tree, parent: NodeId, name: &str, v: [&str; 3]) -> OreRow {
    let row = add(t, parent, "ore row", ore_row_style(7.2));
    // The swatch (line 180) is 12 x 12 and centred; the inputs (line 183) have padding 2px 5px.
    let swatch = add(t, row, "swatch", Style { w: Dim::Px(12.0), h: Dim::Px(12.0), align_self: Some(Align::Center), ..Style::default() });
    let name = mono(t, row, name, 12.0, true, Style::default());
    let input = |t: &mut Tree, s: &str| mono(t, row, s, 12.0, false, Style { pad: Edges::new(2.0, 5.0, 2.0, 5.0), ..Style::default() });
    let inputs = [input(t, v[0]), input(t, v[1]), input(t, v[2])];
    // The remove button (line 2424).
    let button = mono(t, row, "\u{d7}", 12.0, false, Style::default());
    OreRow { row, swatch, name, inputs, button }
}

/// A `.biome` row (line 319): `12px minmax(0,1fr) auto auto; gap 8px; align-items center; padding 3px 0`.
/// The width cell is at least 8ch = 57.6 (line 350), the share cell 6ch = 43.2 (line 351).
fn biome_row(t: &mut Tree, parent: NodeId, name: &str, width: &str, share: &str) -> BiomeRow {
    let cols = [Track::Px(12.0), Track::MinMaxFr0(1.0), Track::Auto, Track::Auto];
    let row = add(t, parent, "biome", Style { pad: Edges::vh(3.0, 0.0), ..grid(&cols, Vec2::new(8.0, 0.0), Align::Center) });
    add(t, row, "swatch", Style { w: Dim::Px(12.0), h: Dim::Px(12.0), ..Style::default() });
    mono(t, row, name, 12.0, true, Style::default());
    let width = mono(t, row, width, 12.0, false, Style { min_w: Dim::Px(57.6), ..Style::default() });
    let share = mono(t, row, share, 12.0, false, Style { min_w: Dim::Px(43.2), ..Style::default() });
    BiomeRow { row, width, share }
}

/// A `.choice` button of `#details` (line 147 and 195-199): a column holding the value and its duration, with
/// `margin-left: -1px` so that neighbours share a hairline, padding 4px 7px.
fn detail_choice(t: &mut Tree, parent: NodeId, value: &str, duration: &str) -> NodeId {
    let style = Style { column: true, margin: Margin { left: Side::Px(-1.0), ..Margin::all(0.0) }, pad: Edges::new(4.0, 7.0, 4.0, 7.0), ..Style::default() };
    let b = add(t, parent, value, style);
    mono(t, b, value, 12.0, false, Style::default());
    mono(t, b, duration, 11.0, false, Style::default());
    b
}

pub fn build_gallery() -> Gallery {
    let mut t = Tree::new();
    // The sheet: the page's grid at 1400 px (line 77-80: `320px minmax(0,1fr) 330px`, here the middle is left
    // out) with a row gap of 24 (PLACEHOLDER: the page has one row), padding 20, items at the top of their cell.
    let root = t.new_node(grid(&[Track::Px(320.0), Track::Px(330.0)], Vec2::new(28.0, 24.0), Align::Start));
    t.set_name(root, "sheet");
    t.style_mut(root).pad = Edges::all(20.0);

    // A: the settings group's body (`.group .body`, line 160: a column, padding-top 4) with three `.setting`s.
    // The third is the focused row (line 172), so its hint shows.
    let a = add(&mut t, root, "settings body", Style { column: true, pad: Edges::new(4.0, 0.0, 0.0, 0.0), ..Style::default() });
    let settings = vec![
        setting(&mut t, a, "Vertical scale", "", "1.00", None),
        setting(&mut t, a, "Elevation boost *", "1.00", "1.40", None),
        setting(&mut t, a, "Rolling hills", "", "on", Some("Some plains get very smooth, hilly ground.")),
    ];

    // B: `#figures` (line 297): `minmax(0,1fr) auto auto`, gap 4px 14px, align-items baseline. Goal values are
    // 14 px (line 303), before values 12 px. All the figures are PLACEHOLDER.
    let figures = add(
        &mut t,
        root,
        "figures",
        grid(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Auto], Vec2::new(14.0, 4.0), Align::Baseline),
    );
    let figure_first = figure_row(&mut t, figures, "Flat land", "38 %", "41 %", 14.0);
    figure_row(&mut t, figures, "Typical flat zone", "420", "390", 14.0);
    figure_row(&mut t, figures, "Typical biome", "610", "", 14.0);
    // `.rule` (line 307): spans all columns, 1 px high, margin 4px 0.
    let rule = add(
        &mut t,
        figures,
        "rule",
        Style {
            col_span_all: true,
            h: Dim::Px(1.0),
            margin: Margin { top: Side::Px(4.0), bottom: Side::Px(4.0), ..Margin::all(0.0) },
            ..Style::default()
        },
    );
    figure_row(&mut t, figures, "Sea", "12 %", "", 12.0);
    figure_row(&mut t, figures, "High land", "9 %", "", 12.0);

    // C: ore rows: a head row (11 px, so 6ch = 39.6 and 5ch = 33) and two rows with the JS defaults (line 2410).
    let c = add(&mut t, root, "ore rows", column(0.0));
    let ore_head = add(&mut t, c, "ore head", ore_row_style(6.6));
    let mut tries = None;
    for (i, word) in ["", "", "tries", "from", "to", ""].into_iter().enumerate() {
        let n = mono(&mut t, ore_head, word, 11.0, false, Style::default());
        if i == 2 {
            tries = Some(n);
        }
    }
    let ore_rows = vec![
        ore_row(&mut t, c, "Coal", ["2", "8", "32"]),
        ore_row(&mut t, c, "Iron", ["2", "8", "32"]),
    ];

    // D: biome rows. Names and numbers are PLACEHOLDER.
    let d = add(&mut t, root, "biome rows", column(0.0));
    let biome_rows = vec![
        biome_row(&mut t, d, "Plains", "640", "18 %"),
        biome_row(&mut t, d, "Forest", "420", "11 %"),
        biome_row(&mut t, d, "Snowy taiga", "1,200", "6 %"),
    ];

    // E: `#run` (line 192): a column, gap 8, padding-top 13 (12 + 1 border). `#details` (lines 195-199) also
    // carries `.choice` (line 147): `grid-template-columns: repeat(3, minmax(0,1fr)); gap: 4px;
    // align-items: baseline; padding-left: 1px`. Durations are PLACEHOLDER (format: line 464).
    let run = add(&mut t, root, "run", Style { pad: Edges::new(13.0, 0.0, 0.0, 0.0), ..column(8.0) });
    let details = add(
        &mut t,
        run,
        "details",
        Style { pad: Edges::new(0.0, 0.0, 0.0, 1.0), ..grid(&[Track::MinMaxFr0(1.0); 3], Vec2::new(4.0, 4.0), Align::Baseline) },
    );
    let choices = [
        detail_choice(&mut t, details, "64 blocks", "40 s"),
        detail_choice(&mut t, details, "32 blocks", "1 min 30 s"),
        detail_choice(&mut t, details, "16 blocks", "4 min"),
    ];
    // `.go` (line 209): a row, gap 6. `#generate`: `flex: 1`, 34 high, padding 0 12. `#stopRun` (line 219),
    // shown as while a run is under way: a 34 x 34 grid with place-items center, `flex: none`.
    let go = add(&mut t, run, "go", Style { gap: Vec2::new(6.0, 0.0), ..Style::default() });
    let generate = mono(
        &mut t,
        go,
        "Generate preview",
        12.0,
        false,
        Style { grow: 1.0, basis: Dim::Px(0.0), h: Dim::Px(34.0), pad: Edges::new(0.0, 12.0, 0.0, 12.0), ..Style::default() },
    );
    let stop_run = add(
        &mut t,
        go,
        "stopRun",
        Style {
            display: Display::Grid,
            align_items: Align::Center,
            justify_items: Align::Center,
            w: Dim::Px(34.0),
            h: Dim::Px(34.0),
            shrink: 0.0,
            ..Style::default()
        },
    );
    let icon = add(&mut t, stop_run, "icon", Style { w: Dim::Px(12.0), h: Dim::Px(12.0), ..Style::default() });

    // F: `#stage` (line 226): `display: grid; place-items: start center`. Its height is PLACEHOLDER. The page
    // sets `#frame` to `min(100cqw, 100cqh - var(--under))` with `--under: 64px`: min(330, 260 - 64) = 196.
    let stage_h = 260.0;
    let frame_w = 330.0f32.min(stage_h - 64.0);
    let stage = add(
        &mut t,
        root,
        "stage",
        Style { h: Dim::Px(stage_h), justify_items: Align::Center, ..grid(&[], Vec2::new(0.0, 0.0), Align::Start) },
    );
    let frame = add(
        &mut t,
        stage,
        "frame",
        Style { position: Position::Relative, w: Dim::Px(frame_w), aspect: Some(1.0), ..Style::default() },
    );
    // `#flat` (lines 228, 231): `inset: 0; overflow: hidden`. The map inside is PLACEHOLDER and larger than the
    // box, so that the clip shows.
    let flat = add(
        &mut t,
        frame,
        "flat",
        Style { display: Display::Block, position: Position::Absolute, inset: Insets::all(Dim::Px(0.0)), overflow: Overflow::Hidden, ..Style::default() },
    );
    add(&mut t, flat, "map", Style { w: Dim::Px(230.0), h: Dim::Px(230.0), ..Style::default() });
    // `#cutLine` (line 253): `left: -9px; right: -9px; height: 0`, its top is set by script (PLACEHOLDER 98).
    let cut_line = add(
        &mut t,
        frame,
        "cutLine",
        Style {
            position: Position::Absolute,
            inset: Insets { left: Dim::Px(-9.0), right: Dim::Px(-9.0), top: Dim::Px(98.0), bottom: Dim::Auto },
            h: Dim::Px(0.0),
            ..Style::default()
        },
    );
    // `#note` (line 257): `inset: 0; display: grid; place-items: center; padding: 40px`, text from line 593.
    let note = add(
        &mut t,
        frame,
        "note",
        Style {
            display: Display::Grid,
            position: Position::Absolute,
            inset: Insets::all(Dim::Px(0.0)),
            align_items: Align::Center,
            justify_items: Align::Center,
            pad: Edges::all(40.0),
            ..Style::default()
        },
    );
    let note_text = mono(&mut t, note, "No preview yet. Choose a seed and generate one.", 12.0, true, Style::default());
    // `#under` (line 260): `left: 0; right: 0; top: calc(100% + 10px)`, a column with gap 6.
    let under = add(
        &mut t,
        frame,
        "under",
        Style {
            position: Position::Absolute,
            inset: Insets { left: Dim::Px(0.0), right: Dim::Px(0.0), top: Dim::Px(frame_w + 10.0), bottom: Dim::Auto },
            ..column(6.0)
        },
    );
    // A `.choice` row (line 261): flex, gap 18, align-items baseline. The probe's text is PLACEHOLDER (format:
    // line 596); `Export` has padding 1px 0 and margin-left 6 (line 152).
    let choice = add(&mut t, under, "choice", Style { gap: Vec2::new(18.0, 0.0), align_items: Align::Baseline, ..Style::default() });
    mono(
        &mut t,
        choice,
        "x 120   z -340   height 72   Plains",
        12.0,
        false,
        Style { grow: 1.0, min_w: Dim::Px(0.0), ..Style::default() },
    );
    mono(
        &mut t,
        choice,
        "Export",
        12.0,
        false,
        Style { pad: Edges::vh(1.0, 0.0), margin: Margin { left: Side::Px(6.0), ..Margin::all(0.0) }, ..Style::default() },
    );
    // `#steps` (line 267): a column, gap 4, a caption (line 475) and a 1 px bar (line 268).
    let steps = add(&mut t, under, "steps", column(4.0));
    mono(&mut t, steps, "Map, every 16 blocks", 12.0, false, Style::default());
    add(&mut t, steps, "bar", Style { h: Dim::Px(1.0), ..Style::default() });

    // G: `.scroll` (line 134): `overflow-y: auto; padding: 0 10px 28px 0`; the height is PLACEHOLDER. The page's
    // `margin-right: -10px` is left out.
    let scroll = add(
        &mut t,
        root,
        "scroll",
        Style {
            display: Display::Block,
            overflow: Overflow::ScrollY,
            col_span_all: true,
            h: Dim::Px(90.0),
            pad: Edges::new(0.0, 10.0, 28.0, 0.0),
            ..Style::default()
        },
    );
    let names = [
        "Temperature scale",
        "Vegetation scale",
        "Temperature range",
        "Vegetation range",
        "Temperature offset",
        "Vegetation offset",
        "Vertical scale",
        "Elevation boost",
    ];
    let scroll_rows = names.iter().map(|n| mono(&mut t, scroll, n, 12.0, false, Style::default())).collect();

    t.compute(root, Vec2::new(GALLERY.0, GALLERY.1));
    Gallery {
        tree: t,
        root,
        cells: [a, figures, c, d, run, stage, scroll],
        settings,
        figures,
        figure_row: figure_first,
        rule,
        ore_head,
        ore_tries: tries.unwrap(),
        ore_rows,
        biome_rows,
        run,
        details,
        choices,
        generate,
        stop_run,
        icon,
        stage,
        frame,
        flat,
        cut_line,
        note,
        note_text,
        under,
        scroll,
        scroll_rows,
    }
}
