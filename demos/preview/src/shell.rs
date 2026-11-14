//! The world preview's shell (build plan step 13): the toolbar and the three columns of `#page`
//! (preview.html lines 63-88, 370-449), built with `nui-ui` and filled with the page's real settings
//! sections and static values.
//!
//! What is a PLACEHOLDER is marked so: the map slot (step 14 paints the map there), the figures, and the
//! band widths and shares (step 18 computes them), and the section's slot (step 16). Step 15 puts the 3D view in
//! the frame: a canvas that takes the orbit's drag and wheel, the scene itself is drawn by the app under the
//! interface list. Not built yet: changed marks, hints and tooltips, the select's menu, the ores group, the
//! filter, the tabs that switch the list, the section, runs and export; in 3D, the structure marks, the right-drag
//! pan, keys and touch, and the no3d note.

use std::collections::BTreeSet;

use nui_base::{Color, DrawList, Orbit, Vec2};
use nui_layout::{Align, Dim, Display, Edges, Insets, Justify, Margin, Position, Side, Style, Track, Tracks};
use nui_ui::{
    ButtonLook, ButtonOpts, FieldLook, FieldOpts, Id, ListRow, RowOpts, ScrollOpts, TabLook, TabOpts, TextLook, Theme, Tone, Ui,
    choice_style, details_style, scroll_style, strip_style,
};

use crate::map::{Carry, Look, frame_cmds, point_under, structure_near, tick_at};
use crate::paint::{View, rows};
use crate::scene;
use crate::world::World;
use crate::bands::{BAND_FIGURES, HEIGHT_STEPS, SEA, SEA_DEPTHS, band_colours, band_names};
use crate::settings::{Kind, SETTINGS, grouped, pack_values};
use crate::text;

/// Height of the toolbar: 10 px of padding and the 28 px strip (line 93).
const TOOLBAR_H: f32 = 38.0;
/// `--under` (line 227): what the stage keeps free under the map for `#under`.
const UNDER: f32 = 64.0;

/// The three layouts of `#page`, one per media query.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Columns {
    pub left: f32,
    pub right: f32,
    pub gap: f32,
    pub pad: Edges,
    /// The gap between the columns of the figures grid.
    pub fig_gap: f32,
    /// The font size of the three goal figures.
    pub goal: f32,
    /// The 1100 px column: the list's cells are narrower.
    pub narrow: bool,
}

impl Columns {
    /// `@media (max-width: 1100px)` and `(max-width: 1250px)` both match the width inclusively, and the narrower
    /// one comes later in the file so it wins. The table is shell-boxes' own, copied so that the two do not depend
    /// on each other.
    pub fn for_width(w: f32) -> Columns {
        if w <= 1100.0 {
            // Lines 87 and 356-363.
            Columns { left: 272.0, right: 262.0, gap: 20.0, pad: Edges::new(14.0, 16.0, 18.0, 16.0), fig_gap: 10.0, goal: 13.0, narrow: true }
        } else if w <= 1250.0 {
            // Lines 84 and 356-359.
            Columns { left: 296.0, right: 280.0, gap: 24.0, pad: Edges::new(16.0, 22.0, 20.0, 22.0), fig_gap: 10.0, goal: 13.0, narrow: false }
        } else {
            // Lines 77-80, 297 and 303.
            Columns { left: 320.0, right: 330.0, gap: 28.0, pad: Edges::new(18.0, 28.0, 22.0, 28.0), fig_gap: 14.0, goal: 14.0, narrow: false }
        }
    }
}

/// What the application owns; the widgets read it and a click or a keystroke changes it.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewState {
    /// The strip's pressed view: 0 to 4 are `text::VIEWS`, 5 is the section.
    pub view: usize,
    pub in3d: bool,
    pub seed: String,
    pub world_open: bool,
    pub settings_open: bool,
    pub figures_open: bool,
    pub history_open: bool,
    /// The settings' values, in `SETTINGS` order.
    pub values: [f64; 43],
    /// The pressed detail: an index in `text::DETAILS`.
    pub detail: usize,
    pub settings_y: f32,
    pub list_y: f32,
    /// How the 2D map is looked at, and the press that may become a carry or a click.
    pub look: Look,
    pub carry: Option<Carry>,
    /// What is chosen in each of the five views (by id): what the map shows alone.
    pub chosen: [BTreeSet<String>; 5],
    /// The 3D camera: made when the 3D view is first built, and kept across view and shape switches (the page
    /// resets it only when the area's width changes, line 1873).
    pub orbit: Option<Orbit>,
    /// Where the pointer was at the last frame of a press that began on the 3D view.
    pub turn_from: Option<Vec2>,
}

impl Default for PreviewState {
    /// The page after loading a preview in the relief view, in a fresh browser (the seed is `preview.json`'s).
    fn default() -> PreviewState {
        PreviewState {
            view: 2,
            in3d: false,
            seed: "94427382".to_string(),
            world_open: true,
            settings_open: true,
            figures_open: true,
            history_open: false,
            values: pack_values(),
            detail: 2,
            settings_y: 0.0,
            list_y: 0.0,
            look: Look::REST,
            carry: None,
            chosen: Default::default(),
            orbit: None,
            turn_from: None,
        }
    }
}

/// Where the shell's parts are, for tests and for the steps that draw into them. Filled while building.
#[derive(Clone, Debug)]
pub struct ShellIds {
    pub page: Id,
    pub left: Id,
    pub middle: Id,
    pub right: Id,
    pub strip: Id,
    pub tabs: Vec<Id>,
    pub world_head: Id,
    pub settings_head: Id,
    pub figures_head: Id,
    pub history_head: Id,
    pub seed: Id,
    pub select: Id,
    pub profiles: Id,
    pub settings_scroll: Id,
    pub group_heads: Vec<Id>,
    /// In `SETTINGS` order.
    pub rows: Vec<Id>,
    pub reset: Id,
    pub run: Id,
    pub status: Id,
    pub details: Id,
    pub generate: Id,
    pub stage: Id,
    pub frame: Id,
    /// The map itself (a canvas), when the frame holds one.
    pub map: Id,
    /// The 3D view's square (a canvas), when the frame holds one; the app draws the scene into its rect.
    pub relief: Id,
    pub under: Id,
    pub probe: Id,
    /// What the probe line says, whole (the line itself is cut with an ellipsis when it does not fit).
    pub probe_text: String,
    pub export: Id,
    pub figures_grid: Id,
    pub goal_values: Vec<Id>,
    pub list_block: Id,
    pub pick: Id,
    pub band_scroll: Id,
    pub bands: Vec<Id>,
}

impl Default for ShellIds {
    fn default() -> ShellIds {
        let o = Id::ROOT;
        ShellIds {
            page: o,
            left: o,
            middle: o,
            right: o,
            strip: o,
            tabs: Vec::new(),
            world_head: o,
            settings_head: o,
            figures_head: o,
            history_head: o,
            seed: o,
            select: o,
            profiles: o,
            settings_scroll: o,
            group_heads: Vec::new(),
            rows: vec![o; SETTINGS.len()],
            reset: o,
            run: o,
            status: o,
            details: o,
            generate: o,
            stage: o,
            frame: o,
            map: o,
            relief: o,
            under: o,
            probe: o,
            probe_text: String::new(),
            export: o,
            figures_grid: o,
            goal_values: Vec::new(),
            list_block: o,
            pick: o,
            band_scroll: o,
            bands: Vec::new(),
        }
    }
}

/// `.column` (line 106): a flex column that may shrink below its content.
fn column() -> Style {
    Style { column: true, min_h: Dim::Px(0.0), ..Style::default() }
}

/// `flex: 1; min-height: 0`.
fn fills() -> Style {
    Style { grow: 1.0, shrink: 1.0, basis: Dim::Px(0.0), min_h: Dim::Px(0.0), ..Style::default() }
}

/// A horizontal rule of one pixel: the `border-top` or `border-bottom` of a block.
fn rule() -> Style {
    Style { h: Dim::Px(1.0), shrink: 0.0, ..Style::default() }
}

/// `.row` (line 204): a label column of 9em and the control.
fn world_row() -> Style {
    Style {
        display: Display::Grid,
        cols: Tracks::new(&[Track::Px(108.0), Track::MinMaxFr0(1.0)]),
        gap: Vec2::new(10.0, 0.0),
        align_items: Align::Center,
        ..Style::default()
    }
}

/// `.setting` (line 164): the label, the empty output, and the field in 8ch.
fn setting_row() -> Style {
    Style {
        display: Display::Grid,
        cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Px(57.6)]),
        gap: Vec2::new(10.0, 0.0),
        align_items: Align::Center,
        pad: Edges::vh(2.0, 0.0),
        ..Style::default()
    }
}

fn rgb(c: [u8; 3]) -> Color {
    Color::hex(((c[0] as u32) << 16) | ((c[1] as u32) << 8) | c[2] as u32)
}

/// Builds the shell into the frame the caller began (`begin_frame(size)` before, `end_frame` after). `size` is
/// the window's: the map's side is arithmetic from it, because the square needs the column's height before
/// layout. `world` is the preview on the map, if there is one.
pub fn build(ui: &mut Ui, s: &mut PreviewState, size: Vec2, world: Option<&World>) -> ShellIds {
    let t = *ui.theme();
    let cols = Columns::for_width(size.x);
    let mut ids = ShellIds::default();
    // A map stopped at a coarser detail cannot give these views (line 2509): the page goes back to the biomes.
    if world.is_some_and(World::coarse) && matches!(s.view, 1 | 4 | 5) {
        s.view = 0;
    }

    // Line 227: `side = min(the column's width, its height - --under)`.
    let middle_w = size.x.min(1480.0) - cols.pad.left - cols.pad.right - cols.left - cols.right - 2.0 * cols.gap;
    let column_h = size.y - TOOLBAR_H - cols.pad.top - cols.pad.bottom;
    let side = middle_w.min(column_h - UNDER).max(0.0);

    // body (lines 63-72): a column filling the window.
    ui.node("body", Style { column: true, ..fills() }, |ui| {
        toolbar(ui, s, &mut ids, world);
        let page = Style {
            display: Display::Grid,
            cols: Tracks::new(&[Track::Px(cols.left), Track::MinMaxFr0(1.0), Track::Px(cols.right)]),
            gap: Vec2::new(cols.gap, cols.gap),
            pad: cols.pad,
            w: Dim::Pct(100.0),
            max_w: Dim::Px(1480.0),
            margin: Margin::horizontal_auto(),
            ..fills()
        };
        ids.page = ui.node("page", page, |ui| {
            ids.left = ui.node("left", column(), |ui| left_column(ui, s, &mut ids, &t));
            ids.middle = ui.node("middle", column(), |ui| middle_column(ui, s, &mut ids, &t, side, world));
            ids.right = ui.node("right", column(), |ui| right_column(ui, s, &mut ids, &t, &cols));
        });
    });
    ids
}

/// `#toolbar` (line 93) and the strip of lines 2543-2555: six views, a gap, then 2D and 3D.
fn toolbar(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, world: Option<&World>) {
    let style = Style {
        justify: Justify::Center,
        pad: Edges::new(10.0, 16.0, 0.0, 16.0),
        shrink: 0.0,
        align_items: Align::Start,
        ..Style::default()
    };
    ui.node("toolbar", style, |ui| {
        ids.strip = ui.node("strip", strip_style(), |ui| {
            let views = text::VIEWS.iter().copied().chain([text::SECTION]);
            // Flat, Structures and Section come after the full map only (line 2506).
            let coarse = world.is_some_and(World::coarse);
            for (i, word) in views.enumerate() {
                let disabled = coarse && matches!(i, 1 | 4 | 5);
                let r = ui.tab(&format!("view{i}"), word, TabLook::Strip, TabOpts { pressed: i == s.view, disabled });
                ids.tabs.push(r.id);
                if r.clicked {
                    s.view = i;
                }
            }
            // The section has no 2D or 3D (line 2540): the tabs and the gap before them are not built.
            if s.view != 5 {
                ui.strip_gap();
                for (i, word) in text::SHAPES.iter().enumerate() {
                    let r = ui.tab(&format!("shape{i}"), word, TabLook::Strip, TabOpts { pressed: (i == 1) == s.in3d, disabled: false });
                    ids.tabs.push(r.id);
                    if r.clicked {
                        s.in3d = i == 1;
                    }
                }
            }
        });
    });
}

/// The left column (lines 370-400): the world, the settings, and the run block under them.
fn left_column(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme) {
    let muted = TextLook::body(Tone::Muted);

    let r = ui.fold("world", text::WORLD, &mut s.world_open, |ui| {
        ui.node("seedRow", world_row(), |ui| {
            ui.text(text::SEED, muted);
            let opts = FieldOpts { look: FieldLook::Text, placeholder: text::SEED_RANDOM, max_chars: 64, width: Dim::Pct(100.0), focus: false };
            ids.seed = ui.text_field("seed", &mut s.seed, opts).resp.id;
        });
        ui.node("areaRow", world_row(), |ui| {
            ui.text(text::SPAN, muted);
            // 256 samples x 16 blocks (line 459): the scanned area as the page opens.
            let r = ui.select("area", "4,096 blocks");
            ids.select = r.id;
            if r.clicked {
                nui_base::observe!(Info, "the select's menu comes with step 14");
            }
        });
    });
    ids.world_head = r.id;

    let r = ui.fold_with("settings", text::PROFILES, &mut s.settings_open, nui_ui::FoldOpts { grow: true }, |ui| {
        // Line 153: `#profiles` pulls its first word 6 px left, under the padding of its button.
        let style = Style { wrap: true, margin: Margin { left: Side::Px(-6.0), ..Margin::default() }, ..choice_style() };
        ids.profiles = ui.node("profiles", style, |ui| {
            let at_pack = s.values == pack_values();
            let r = ui.tab("pack", text::PACK, TabLook::Choice, TabOpts { pressed: at_pack, disabled: false });
            if r.clicked {
                s.values = pack_values();
            }
        });
        settings_scroll(ui, &mut s.settings_y, &mut s.values, ids, t);
    });
    ids.settings_head = r.id;

    run_block(ui, s, ids, t);
}

/// `#settings` (lines 157-164): five groups, each a title over its rows, then the reset hint.
fn settings_scroll(ui: &mut Ui, y: &mut f32, values: &mut [f64; 43], ids: &mut ShellIds, t: &Theme) {
    let groups = grouped();
    let body = TextLook::body(Tone::Text);
    let r = ui.scroll("settings", y, scroll_style(), ScrollOpts::LIST, |ui| {
        for (g, members) in groups.iter().enumerate() {
            ui.node(&format!("g{g}"), Style { column: true, shrink: 0.0, ..Style::default() }, |ui| {
                // The group title (lines 105, 157): 4 px above (14 px between groups), 4 px under, then the
                // hairline of `--line`.
                let head = Style { column: true, ..Style::default() };
                ids.group_heads.push(ui.node("head", head, |ui| {
                    let row = Style {
                        align_items: Align::Baseline,
                        gap: Vec2::new(10.0, 0.0),
                        pad: Edges::new(if g == 0 { 4.0 } else { 14.0 }, 0.0, 4.0, 0.0),
                        ..Style::default()
                    };
                    ui.node("title", row, |ui| ui.text(&text::GROUPS[g].to_uppercase(), TextLook::label()));
                    ui.fill(rule(), t.line);
                }));
                ui.node("body", Style { column: true, pad: Edges::new(4.0, 0.0, 0.0, 0.0), ..Style::default() }, |ui| {
                    for &i in members {
                        let set = &SETTINGS[i];
                        ids.rows[i] = ui.node(set.key, setting_row(), |ui| {
                            ui.ellipsis(set.label, body);
                            // The output, empty until a value differs from the pack's.
                            ui.text("", TextLook::body(Tone::Muted));
                            match set.kind {
                                Kind::Number(spec) => {
                                    ui.number("field", &mut values[i], spec);
                                }
                                Kind::Switch => {
                                    let mut on = values[i] != 0.0;
                                    if ui.toggle("field", &mut on, text::ON, text::OFF).clicked {
                                        values[i] = if on { 1.0 } else { 0.0 };
                                    }
                                }
                            }
                        });
                    }
                });
            });
        }
        // Line 161.
        ids.reset = ui.node("reset", Style { column: true, shrink: 0.0, pad: Edges::new(12.0, 0.0, 0.0, 0.0), ..Style::default() }, |ui| {
            ui.paragraph(text::RESET_HINT, TextLook::small(Tone::Muted));
        });
    });
    ids.settings_scroll = r.id;
}

/// `#run` (line 192): a hairline, the status, the details and the generate button.
fn run_block(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme) {
    let style = Style { column: true, margin: Margin { top: Side::Auto, ..Margin::default() }, shrink: 0.0, ..Style::default() };
    ids.run = ui.node("run", style, |ui| {
        ui.fill(rule(), t.edge);
        let inner = Style { column: true, pad: Edges::new(12.0, 0.0, 0.0, 0.0), gap: Vec2::new(0.0, 8.0), ..Style::default() };
        ui.node("runBody", inner, |ui| {
            ids.status = ui.node("status", Style::default(), |ui| ui.paragraph(text::UP_TO_DATE, TextLook::status(Tone::Muted)));
            ids.details = ui.node("details", details_style(), |ui| {
                for (i, word) in text::DETAILS.iter().enumerate() {
                    // No wait is known without timings, so the small line is left out.
                    let r = ui.detail_tab(&format!("d{i}"), word, "", TabOpts { pressed: i == s.detail, disabled: false });
                    if r.clicked {
                        s.detail = i;
                    }
                }
            });
            ui.node("go", Style { gap: Vec2::new(6.0, 0.0), ..Style::default() }, |ui| {
                // Disabled: the map already shows these settings (line 467).
                ids.generate = ui.button_with("generate", text::GENERATE, ButtonLook::Primary, ButtonOpts { disabled: true, grow: 1.0 }).id;
            });
        });
    });
}

/// The middle column: the stage holds the map's square and the strip under it (line 226).
fn middle_column(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme, side: f32, world: Option<&World>) {
    let stage = Style { align_items: Align::Center, ..Style { column: true, ..fills() } };
    ids.stage = ui.node("stage", stage, |ui| {
        let frame = Style { position: Position::Relative, w: Dim::Px(side), h: Dim::Px(side), shrink: 0.0, ..Style::default() };
        ids.frame = ui.node("frame", frame, |ui| {
            let view = View::from_index(s.view);
            let slot = Style { w: Dim::Px(side), h: Dim::Px(side), shrink: 0.0, ..Style::default() };
            match (world, view) {
                (None, _) => note(ui, text::NO_PREVIEW),
                // PLACEHOLDER: the section draws here in step 16.
                (Some(_), None) => ui.fill(slot, t.line),
                // Before the 3D view: a world without structures has nothing to drape (line 2521).
                (Some(w), Some(View::Structures)) if w.structures.is_none() => note(ui, text::NO_STRUCTURES),
                (Some(w), Some(_)) if s.in3d => ids.relief = relief_canvas(ui, s, side, w),
                (Some(w), Some(view)) => ids.map = map_canvas(ui, s, t, side, w, view),
            }
            // `#under` (line 260): 10 px below the map, as wide as it.
            let under = Style {
                position: Position::Absolute,
                inset: Insets { left: Dim::Px(0.0), right: Dim::Px(0.0), top: Dim::Px(side + 10.0), ..Insets::default() },
                column: true,
                gap: Vec2::new(0.0, 6.0),
                ..Style::default()
            };
            let probe_text = probe_text(ui, s, ids, side, world);
            ids.probe_text = probe_text.clone();
            ids.under = ui.node("under", under, |ui| {
                ui.node("probeRow", Style { gap: Vec2::new(18.0, 0.0), align_items: Align::Baseline, ..Style::default() }, |ui| {
                    let probe = Style { grow: 1.0, basis: Dim::Px(0.0), min_w: Dim::Px(0.0), ..Style::default() };
                    ids.probe = ui.node("probe", probe, |ui| ui.ellipsis(&probe_text, TextLook::body(Tone::Muted)));
                    let r = ui.button("export", text::EXPORT, ButtonLook::Link);
                    ids.export = r.id;
                    if r.clicked {
                        nui_base::observe!(Info, "export comes with step 18");
                    }
                });
                // `#steps` is empty until a section is cut.
                ui.node("steps", column(), |_| {});
            });
        });
    });
}

/// `#note` (line 257): over the whole frame, its words centred in 40 px of padding, in the muted colour. The
/// lines of a wrapped note are left-aligned in a block that is centred, not centred one by one: UNVERIFIED
/// against the page, which centres each line.
fn note(ui: &mut Ui, words: &str) {
    let style = Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(0.0), right: Dim::Px(0.0), top: Dim::Px(0.0), bottom: Dim::Px(0.0) },
        pad: Edges::all(40.0),
        align_items: Align::Center,
        justify: Justify::Center,
        ..Style::default()
    };
    ui.node("note", style, |ui| ui.paragraph(words, TextLook::body(Tone::Muted)));
}

/// What `#probe` says: the point or structure under the pointer when it is over the map, else the area
/// (`restProbe`, 1761-1765, and the `mousemove` of 1828-1835). The pointer is judged against the last frame.
fn probe_text(ui: &Ui, s: &PreviewState, ids: &ShellIds, side: f32, world: Option<&World>) -> String {
    let Some(w) = world else { return text::area_probe("4,096", "512", false) };
    let map_shown = ids.map != Id::ROOT;
    let rest = || text::area_probe(&text::num(w.span()), &text::num(w.span() / 8.0), map_shown && tick_at(w, s.look).is_some());
    let (Some(p), Some(rect), Some(view)) = (ui.pointer(), ui.rect(ids.map), View::from_index(s.view)) else { return rest() };
    if !map_shown || !rect.contains(p) {
        return rest();
    }
    let local = Vec2::new(p.x - rect.x, p.y - rect.y);
    if view == View::Structures {
        if let Some((kind, x, z)) = structure_near(w, &s.chosen[4], s.look, side, local) {
            let name = text::pretty(&w.structures.as_ref().expect("a near structure needs structures").ids[kind]);
            return text::structure_probe(&text::num(x), &text::num(z), &name);
        }
    }
    let (col, row, i) = point_under(w, s.look, side, local);
    text::probe(
        &text::num(w.origin_x + col as f64 * w.step),
        &text::num(w.origin_z + row as f64 * w.step),
        &text::num(w.heights[i] as f64),
        &text::pretty(&w.biomes[w.biome_at[i] as usize].id),
    )
}

/// The 2D map: a canvas that takes the wheel, the carry and the click, and draws the picture, the marks, the
/// grid, the cross and the ticks (`map::frame_cmds`). Returns its id.
fn map_canvas(ui: &mut Ui, s: &mut PreviewState, t: &Theme, side: f32, world: &World, view: View) -> Id {
    let style = Style { w: Dim::Px(side), h: Dim::Px(side), shrink: 0.0, ..Style::default() };
    let (grid, muted) = (t.grid, t.muted);
    let resp = ui.canvas("map", style, |ui, resp| {
        if let Some(rect) = resp.rect {
            // The wheel comes closer to the point under it (lines 1796-1803).
            if let Some((notches, p)) = ui.take_wheel(resp.id) {
                s.look = s.look.wheel(Vec2::new((p.x - rect.x) / rect.w, (p.y - rect.y) / rect.h), notches);
            }
            let pointer = ui.pointer();
            if resp.pressed {
                if let Some(from) = pointer {
                    s.carry = Some(Carry { from, look: s.look, carried: false });
                }
            }
            // A press that travels carries the map. Read until the press is over, not only while held, so the
            // last movement before the release is not lost.
            if let (Some(c), Some(p)) = (s.carry.as_mut(), pointer) {
                let d = Vec2::new(p.x - c.from.x, p.y - c.from.y);
                // A hand is never still: a few pixels are not yet a move.
                if !c.carried && d.x.hypot(d.y) >= 4.0 {
                    c.carried = true;
                }
                if c.carried {
                    s.look = Look::dragged(c.look, d, side);
                }
            }
            // A press that stays in place is the click that chooses (lines 1821-1827); the structures are
            // chosen in the list only.
            if resp.clicked && view != View::Structures && s.carry.is_some_and(|c| !c.carried) {
                if let (Some(p), Some((at, ids))) = (pointer, rows(world, view)) {
                    let (_, _, i) = point_under(world, s.look, side, Vec2::new(p.x - rect.x, p.y - rect.y));
                    let id = ids[at[i] as usize].clone();
                    let set = &mut s.chosen[view.index()];
                    if !set.remove(&id) {
                        set.insert(id);
                    }
                }
            }
            if !resp.held {
                s.carry = None;
            }
        }
        frame_cmds(world, view, &s.chosen[view.index()], s.look, side, grid, muted)
    });
    resp.id
}
/// The 3D view: a canvas that takes the orbit's drag and wheel. It draws nothing itself, because the scene is a mesh
/// the app draws into this rect before the interface list (the page's #relief, absolutely over the frame).
/// Returns its id.
fn relief_canvas(ui: &mut Ui, s: &mut PreviewState, side: f32, world: &World) -> Id {
    let style = Style { w: Dim::Px(side), h: Dim::Px(side), shrink: 0.0, ..Style::default() };
    let resp = ui.canvas("relief", style, |ui, resp| {
        let orbit = s.orbit.get_or_insert_with(|| scene::orbit(world, None));
        if let Some(rect) = resp.rect {
            if let Some((notches, _)) = ui.take_wheel(resp.id) {
                orbit.zoom(notches);
            }
            let pointer = ui.pointer();
            if resp.pressed {
                s.turn_from = pointer;
            }
            // Read until the press is over, not only while held, so the last movement before the release counts
            // (as the map does). A drag that leaves the square keeps turning: OrbitControls captures the pointer.
            if let (Some(from), Some(p)) = (s.turn_from, pointer) {
                // OrbitControls turns a full circle per clientHeight of its canvas: the square's side.
                orbit.drag(p.x - from.x, p.y - from.y, rect.h);
                s.turn_from = Some(p);
            }
            if !resp.held {
                s.turn_from = None;
            }
        }
        Vec::new()
    });
    resp.id
}

/// The right column (lines 429-449): the figures and the history, then the list of height bands.
fn right_column(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme, cols: &Columns) {
    let body = TextLook::body(Tone::Text);
    let muted = TextLook::body(Tone::Muted);
    ui.node("figuresBlock", column(), |ui| {
        let r = ui.fold("figures", text::FIGURES, &mut s.figures_open, |ui| {
            // Line 297: name, value and change; the rule before "Sea" spans all three.
            let grid = Style {
                display: Display::Grid,
                cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Auto]),
                gap: Vec2::new(cols.fig_gap, 4.0),
                align_items: Align::Baseline,
                ..Style::default()
            };
            // PLACEHOLDER: the figures of a run. "25 of 146" is `preview.json`'s 25 biomes and 146 possible.
            const VALUES: [&str; 7] = ["38.2%", "1,250 blocks", "900 blocks", "41.5%", "6.3%", "height 287", "25 of 146"];
            ids.figures_grid = ui.node("grid", grid, |ui| {
                for (i, name) in text::FIGURE_NAMES.iter().enumerate() {
                    if i == 3 {
                        let span = Style { col_span_all: true, pad: Edges::vh(4.0, 0.0), ..Style::default() };
                        ui.node("rule", span, |ui| ui.fill(Style { h: Dim::Px(1.0), grow: 1.0, shrink: 0.0, ..Style::default() }, t.line));
                    }
                    ui.node(&format!("name{i}"), Style { min_w: Dim::Px(0.0), ..Style::default() }, |ui| ui.paragraph(name, body));
                    // Line 303: the three figures of the goal are larger and semibold.
                    let look = if i < 3 {
                        TextLook { size: cols.goal, line: cols.goal * 1.5, weight: nui_base::Weight::SemiBold, ..body }
                    } else {
                        body
                    };
                    let v = ui.node(&format!("value{i}"), Style { justify: Justify::End, ..Style::default() }, |ui| ui.text(VALUES[i], look));
                    if i < 3 {
                        ids.goal_values.push(v);
                    }
                    ui.text("", muted);
                }
            });
        });
        ids.figures_head = r.id;
        // Closed: the body would be the empty `#history`.
        let r = ui.fold("history", text::HISTORY, &mut s.history_open, |ui| ui.text("", body));
        ids.history_head = r.id;
    });

    // `#listBlock.rule` (lines 295-296, 440): a hairline on top, 9 px of padding, a gap of 6.
    ids.list_block = ui.node("listBlock", Style { column: true, ..fills() }, |ui| {
        ui.fill(rule(), t.edge);
        let inner = Style { column: true, pad: Edges::new(9.0, 0.0, 0.0, 0.0), gap: Vec2::new(0.0, 6.0), ..fills() };
        ui.node("listBody", inner, |ui| {
            ui.node("listHead", column(), |ui| {
                ui.node("title", Style { pad: Edges::new(0.0, 0.0, 2.0, 0.0), ..Style::default() }, |ui| {
                    ui.text(&text::LIST_HEIGHT[0].to_uppercase(), TextLook::heading());
                });
                // `#pick` (line 341): two lines of the small size at most.
                let pick = Style { h: Dim::Px(34.0), overflow: nui_layout::Overflow::Hidden, ..Style::default() };
                ids.pick = ui.node("pick", pick, |ui| ui.paragraph(text::PICK_HINT, TextLook::small(Tone::Muted)));
            });
            ui.list_head("legend", text::LIST_HEIGHT[1], text::COL_WIDTH, text::COL_SHARE, cols.narrow);
            let names = band_names(SEA, &HEIGHT_STEPS, &SEA_DEPTHS);
            let colours = band_colours(SEA, &HEIGHT_STEPS, &SEA_DEPTHS);
            let r = ui.scroll("list", &mut s.list_y, scroll_style(), ScrollOpts::LIST, |ui| {
                for i in 0..names.len() {
                    let row = ListRow { swatch: rgb(colours[i]), name: &names[i], width: BAND_FIGURES[i].0, share: BAND_FIGURES[i].1 };
                    let r = ui.list_row(&format!("band{i}"), &row, RowOpts { narrow: cols.narrow, ..RowOpts::default() });
                    ids.bands.push(r.id);
                }
            });
            ids.band_scroll = r.id;
        });
    });
}

/// The shell as a picture: builds until the scrollbar's gutter has settled (it comes one frame late), at most
/// three times, and returns the last draw list with the `Ui` (for the rects) and the ids.
pub fn picture(theme: Theme, size: Vec2) -> (DrawList, Ui, ShellIds) {
    let mut ui = Ui::new(theme);
    let mut s = PreviewState::default();
    let mut last = None;
    for _ in 0..3 {
        ui.begin_frame(size);
        let ids = build(&mut ui, &mut s, size, None);
        let list = ui.end_frame().clone();
        last = Some((list, ids));
        if !ui.want_redraw() {
            break;
        }
    }
    let (list, ids) = last.expect("the loop runs at least once");
    (list, ui, ids)
}
