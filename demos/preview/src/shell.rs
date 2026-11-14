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

use nui_base::{Cmd, Color, DrawList, Orbit, Rect, Vec2};
use nui_layout::{Align, Dim, Display, Edges, Insets, Justify, Margin, Position, Side, Style, Track, Tracks};
use nui_ui::{
    ButtonLook, ButtonOpts, FieldLook, FieldOpts, Id, ListRow, RowOpts, ScrollOpts, TabLook, TabOpts, TextLook, Theme, Tone, Ui,
    choice_style, details_style, scroll_style, strip_style, wrap_lines,
};

use crate::cut::{self, CUT_IMAGE, CutLook, NAV_H, NAV_IMAGE};
use crate::map::{Carry, Look, frame_cmds, point_under, structure_near, tick_at};
use crate::paint::{View, rows};
use crate::run::{self, Ask, DETAIL_BLOCKS, Follow, Pace, Steps, duration};
use crate::scene;
use crate::section::{ORE_COLOURS, ORE_KINDS, Shown};
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

/// A press the shell recorded and the application acts on after the build (a frame may build up to three times,
/// so the shell never acts itself: recording is idempotent).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Want {
    Generate,
    Stop,
    Export,
    OpenFolder,
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
    /// The cut of the section being looked at (0-based).
    pub cut_at: usize,
    /// How the cut is looked at; `CutLook::FRESH` until its first settling.
    pub cut_look: CutLook,
    /// A press on the cut: where it began and the look then.
    pub cut_carry: Option<(Vec2, CutLook)>,
    /// A press on the strip: how far from the frame's left edge it took hold.
    pub nav_carry: Option<f64>,
    /// The ores chosen in the section's list, by kind name (`chosen.section`): the cut draws these alone.
    pub chosen_ores: BTreeSet<String>,
    /// The scanned area in 16-block points, the page's `samples` (what the area select holds).
    pub samples: u32,
    /// What the map on screen was made from (the page's `shown`): a fact of the world, whatever the form says.
    pub made: Option<Ask>,
    /// The run being followed, if one is under way.
    pub follow: Option<Follow>,
    /// The status line's two sentences that outlive a run: the trouble (in the warning tone) and what was said.
    pub trouble: String,
    pub said: String,
    /// `#steps`: the last run's steps, or an export's notice.
    pub steps: Steps,
    /// The press to act on after this build.
    pub want: Option<Want>,
    /// A copy of the learned pace the shell reads for the detail tabs' waits; the application owns the file.
    pub pace: Pace,
}

impl PreviewState {
    /// The page after `loadWorld`: the form holds the settings of the run that made the map.
    pub fn for_world(world: Option<&World>) -> PreviewState {
        let mut s = PreviewState::default();
        if let Some(ask) = world.and_then(run::ask_of) {
            run::adopt(&mut s, &ask);
            s.made = Some(ask);
        }
        s
    }
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
            cut_at: 0,
            cut_look: CutLook::FRESH,
            cut_carry: None,
            nav_carry: None,
            chosen_ores: BTreeSet::new(),
            samples: 256,
            made: None,
            follow: None,
            trouble: String::new(),
            said: String::new(),
            steps: Steps::Empty,
            want: None,
            pace: Pace::default(),
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
    /// The list's title as shown: "HEIGHTS", or "ORES" in the section.
    pub list_title: String,
    pub pick: Id,
    pub band_scroll: Id,
    pub bands: Vec<Id>,
    /// The cut of the section (a column inside the frame), its strip, the strip's note and the cut at true scale
    /// (canvases): the ids are Id::ROOT unless a cut is shown.
    pub cut: Id,
    pub nav: Id,
    pub nav_note: Id,
    pub cut_view: Id,
    /// The cut view's border box as the look arithmetic takes it (before layout): width and height.
    pub cut_box: (f64, f64),
    /// "Another cut", when the section view has a section.
    pub next_cut: Id,
    /// The ore rows of the section's list, in `ORE_KINDS` order.
    pub ores: Vec<Id>,
    /// The square beside the generate button, while a run is under way (Id::ROOT otherwise).
    pub stop: Id,
    /// `#steps`, its hairline, and the export notice with its two links (Id::ROOT unless shown).
    pub steps: Id,
    pub bar: Id,
    pub saved: Id,
    pub open_folder: Id,
    pub close_notice: Id,
    /// What the status line says, whole ("" when it is not built), and the word on the generate button.
    pub status_text: String,
    pub generate_text: String,
    /// The steps as the line shows them: "✓ label 12 s" for a done one, "› label" for the one under way.
    pub steps_items: Vec<String>,
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
            list_title: String::new(),
            pick: o,
            band_scroll: o,
            bands: Vec::new(),
            cut: o,
            nav: o,
            nav_note: o,
            cut_view: o,
            cut_box: (0.0, 0.0),
            next_cut: o,
            ores: Vec::new(),
            stop: o,
            steps: o,
            bar: o,
            saved: o,
            open_folder: o,
            close_notice: o,
            status_text: String::new(),
            generate_text: String::new(),
            steps_items: Vec::new(),
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
pub fn build(ui: &mut Ui, s: &mut PreviewState, size: Vec2, world: Option<&World>, shown: Option<&Shown>) -> ShellIds {
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
    // The section takes the square's place (line 283): the column's whole width, and its height less `--under`.
    let frame_size = if s.view == 5 { (middle_w.max(0.0), (column_h - UNDER).max(0.0)) } else { (side, side) };

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
            ids.middle = ui.node("middle", column(), |ui| middle_column(ui, s, &mut ids, &t, frame_size, world, shown));
            ids.right = ui.node("right", column(), |ui| right_column(ui, s, &mut ids, &t, &cols, world, shown));
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
            let r = ui.select("area", &format!("{} blocks", text::num(s.samples as f64 * 16.0)));
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

/// What `#status` says (`showButton`, lines 799-802, and `tick`, 2610-2615): the clock while a run is followed, else
/// the trouble, what was said, or that the map already shows these settings.
fn status_words(s: &PreviewState) -> String {
    if let Some(f) = &s.follow {
        let elapsed = duration(f.seconds);
        return match f.expected {
            Some(e) => text::running(&elapsed, &duration(e)),
            None => text::measuring(&elapsed),
        };
    }
    if !s.trouble.is_empty() {
        return s.trouble.clone();
    }
    if !s.said.is_empty() {
        return s.said.clone();
    }
    if made_shows_form(s) { text::UP_TO_DATE.to_string() } else { String::new() }
}

/// `sameAs(shown)`: the map on screen was made from what the form holds.
fn made_shows_form(s: &PreviewState) -> bool {
    s.made.as_ref().is_some_and(|m| m.same_as(&s.seed, s.samples, DETAIL_BLOCKS[s.detail], &s.values))
}

/// `#run` (line 192): a hairline, the status, the details and the generate button with the stop square beside it.
fn run_block(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme) {
    let style = Style { column: true, margin: Margin { top: Side::Auto, ..Margin::default() }, shrink: 0.0, ..Style::default() };
    ids.run = ui.node("run", style, |ui| {
        ui.fill(rule(), t.edge);
        let inner = Style { column: true, pad: Edges::new(12.0, 0.0, 0.0, 0.0), gap: Vec2::new(0.0, 8.0), ..Style::default() };
        ui.node("runBody", inner, |ui| {
            // `#status:empty { display: none }`: nothing to say, no box.
            ids.status_text = status_words(s);
            if !ids.status_text.is_empty() {
                let tone = if s.follow.is_none() && !s.trouble.is_empty() { Tone::Warn } else { Tone::Muted };
                let words = ids.status_text.clone();
                ids.status = ui.node("status", Style::default(), |ui| ui.paragraph(&words, TextLook::status(tone)));
            }
            ids.details = ui.node("details", details_style(), |ui| {
                for (i, word) in text::DETAILS.iter().enumerate() {
                    // The wait this machine took for this detail on this area, under its name (showDetails, 819-830).
                    let wait = s.pace.about(s.samples, DETAIL_BLOCKS[i]).map(duration).unwrap_or_default();
                    // The sixteen-block map of a wide area is out of reach (line 823).
                    let disabled = DETAIL_BLOCKS[i] == 16 && s.samples > 2048;
                    let r = ui.detail_tab(&format!("d{i}"), word, &wait, TabOpts { pressed: i == s.detail, disabled });
                    if r.clicked {
                        s.detail = i;
                    }
                }
            });
            let same = made_shows_form(s);
            let busy = s.follow.as_ref().map(|f| (f.ask.same_as(&s.seed, s.samples, DETAIL_BLOCKS[s.detail], &s.values), f.share(), f.stopping));
            ui.node("go", Style { gap: Vec2::new(6.0, 0.0), ..Style::default() }, |ui| {
                // While a run is under way the button says so; once a setting moved it starts the new run in its
                // place (lines 789-798). The shell only records the press: the application acts on it.
                let (word, look, disabled) = match busy {
                    Some((true, share, stopping)) => (text::GENERATING, ButtonLook::Running { share }, stopping),
                    Some((false, _, stopping)) => (text::GENERATE, ButtonLook::Primary, stopping),
                    // Disabled: the map already shows these settings (line 467).
                    None => (text::GENERATE, ButtonLook::Primary, same),
                };
                ids.generate_text = word.to_string();
                let r = ui.button_with("generate", word, look, ButtonOpts { disabled, grow: 1.0 });
                ids.generate = r.id;
                if r.clicked {
                    s.want = Some(Want::Generate);
                }
                if let Some((_, _, stopping)) = busy {
                    let r = ui.stop_button("stopRun", stopping);
                    ids.stop = r.id;
                    if r.clicked {
                        s.want = Some(Want::Stop);
                    }
                }
            });
        });
    });
}
/// The middle column: the stage holds the map's square and the strip under it (line 226).
fn middle_column(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme, frame_size: (f32, f32), world: Option<&World>, shown: Option<&Shown>) {
    // The square's side in the 2D and 3D views; the section's frame is the column's width by the height left.
    let (side, frame_h) = frame_size;
    let stage = Style { align_items: Align::Center, ..Style { column: true, ..fills() } };
    ids.stage = ui.node("stage", stage, |ui| {
        let frame = Style { position: Position::Relative, w: Dim::Px(side), h: Dim::Px(frame_h), shrink: 0.0, ..Style::default() };
        ids.frame = ui.node("frame", frame, |ui| {
            let view = View::from_index(s.view);
            // The note comes after the veil, which dims the content and not the note (line 238).
            let mut words = None;
            match (world, view) {
                (None, _) => words = Some(text::NO_PREVIEW),
                // The section (view 5): a world without one says so; one whose cut is not read yet shows nothing.
                (Some(w), None) if w.section.is_none() => words = Some(text::NO_SECTION),
                (Some(w), None) => {
                    if let Some(shown) = shown {
                        cut_frame(ui, s, ids, t, w, shown, (side, frame_h));
                    }
                }
                // Before the 3D view: a world without structures has nothing to drape (line 2521).
                (Some(w), Some(View::Structures)) if w.structures.is_none() => words = Some(text::NO_STRUCTURES),
                (Some(w), Some(_)) if s.in3d => ids.relief = relief_canvas(ui, s, side, w),
                (Some(w), Some(view)) => ids.map = map_canvas(ui, s, t, side, w, view),
            }
            // `#frame.busy` (line 238): while a run is under way the map, the 3D view and the cut are at opacity
            // 0.3, which over the ground is the ground at 0.7 laid over them.
            if s.follow.is_some() {
                let inset = Insets { left: Dim::Px(0.0), right: Dim::Px(0.0), top: Dim::Px(0.0), bottom: Dim::Px(0.0) };
                ui.fill(Style { position: Position::Absolute, inset, ..Style::default() }, t.ground.with_alpha(0.7));
            }
            if let Some(words) = words {
                note(ui, words);
            }
            // `#under` (line 260): 10 px below the map, as wide as it.
            let under = Style {
                position: Position::Absolute,
                inset: Insets { left: Dim::Px(0.0), right: Dim::Px(0.0), top: Dim::Px(frame_h + 10.0), ..Insets::default() },
                column: true,
                gap: Vec2::new(0.0, 6.0),
                ..Style::default()
            };
            let probe_text = if view.is_none() { cut_probe_text(ui, s, ids, world, shown) } else { probe_text(ui, s, ids, side, world) };
            ids.probe_text = probe_text.clone();
            // "Another cut" is there in the section view only, when there is a section (line 2527).
            let cuts = if view.is_none() { world.and_then(|w| w.section.as_ref()).map_or(0, |sec| sec.cuts.len()) } else { 0 };
            ids.under = ui.node("under", under, |ui| {
                ui.node("probeRow", Style { gap: Vec2::new(18.0, 0.0), align_items: Align::Baseline, ..Style::default() }, |ui| {
                    let probe = Style { grow: 1.0, basis: Dim::Px(0.0), min_w: Dim::Px(0.0), ..Style::default() };
                    ids.probe = ui.node("probe", probe, |ui| ui.ellipsis(&probe_text, TextLook::body(Tone::Muted)));
                    if cuts > 0 {
                        let r = ui.button("nextCut", text::NEXT_CUT, ButtonLook::Link);
                        ids.next_cut = r.id;
                        if r.clicked {
                            s.cut_at = (s.cut_at + 1) % cuts;
                        }
                    }
                    let r = ui.button("export", text::EXPORT, ButtonLook::Link);
                    ids.export = r.id;
                    if r.clicked {
                        s.want = Some(Want::Export);
                    }
                });
                steps_block(ui, s, ids, t, side);
            });
        });
    });
}

/// `#steps` (lines 263-273, 2563-2581, 2800-2813): a run's steps under a hairline that fills as the wait goes, or the
/// notice of where an export went. While a run is followed they are read from it; after it, from the state.
/// `width` is the frame's: `#under` is as wide.
fn steps_block(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme, width: f32) {
    let shown = match &s.follow {
        Some(f) => Steps::Run { done: f.done.clone(), now: f.current(), share: f.share() },
        None => s.steps.clone(),
    };
    let muted = TextLook::body(Tone::Muted);
    let now_look = TextLook::body(Tone::Text);
    let column = Style { column: true, gap: Vec2::new(0.0, 4.0), ..Style::default() };
    ids.steps = ui.node("steps", column, |ui| match &shown {
        Steps::Empty => {}
        Steps::Run { done, now, share } => {
            let (line, text_colour, share) = (t.line, t.text, *share);
            // `.bar` (line 268): a 1 px hairline of `--line`, filled from the left with `--text`.
            ids.bar = ui
                .canvas("bar", Style { h: Dim::Px(1.0), shrink: 0.0, ..Style::default() }, |_, _| {
                    vec![
                        Cmd::Fill { rect: Rect::new(0.0, 0.0, width, 1.0), color: line },
                        Cmd::Fill { rect: Rect::new(0.0, 0.0, share.clamp(0.0, 1.0) * width, 1.0), color: text_colour },
                    ]
                })
                .id;
            // `.line` (line 269): one line, cut at the edge; the names of the done steps shorten first (`.step`,
            // 270-273), their seconds and the step under way stay whole.
            let row = Style { gap: Vec2::new(14.0, 0.0), min_w: Dim::Px(0.0), overflow: nui_layout::Overflow::Hidden, ..Style::default() };
            ui.node("line", row, |ui| {
                for (i, (label, seconds)) in done.iter().enumerate() {
                    let took = duration(*seconds);
                    ids.steps_items.push(format!("{} {label} {took}", text::DONE_MARK));
                    let step = Style { gap: Vec2::new(6.0, 0.0), min_w: Dim::Px(0.0), ..Style::default() };
                    ui.node(&format!("step{i}"), step, |ui| {
                        ui.text(text::DONE_MARK, muted);
                        ui.ellipsis(label, muted);
                        ui.text(&took, muted);
                    });
                }
                if !now.is_empty() {
                    ids.steps_items.push(format!("{} {now}", text::NOW_MARK));
                    ui.node("now", Style { gap: Vec2::new(6.0, 0.0), shrink: 0.0, ..Style::default() }, |ui| {
                        ui.text(text::NOW_MARK, now_look);
                        ui.text(now, now_look);
                    });
                }
            });
        }
        Steps::Saved { folder } => {
            // `.saved` (lines 277-281): the path takes a whole line, then the two links, wrapping. UNVERIFIED: the
            // label and the path stand side by side here, the page flows the path on after its label.
            let notice = Style { wrap: true, align_items: Align::Baseline, gap: Vec2::new(16.0, 2.0), ..Style::default() };
            ids.saved = ui.node("saved", notice, |ui| {
                let path = Style { basis: Dim::Pct(100.0), align_items: Align::Baseline, ..Style::default() };
                ui.node("path", path, |ui| {
                    let label = Style { shrink: 0.0, margin: Margin { right: Side::Px(6.0), ..Margin::default() }, ..Style::default() };
                    ui.node("to", label, |ui| ui.text(text::EXPORTED_TO, muted));
                    let folder_box = Style { column: true, grow: 1.0, basis: Dim::Px(0.0), min_w: Dim::Px(0.0), ..Style::default() };
                    ui.node("folder", folder_box, |ui| ui.paragraph(folder, now_look));
                });
                let r = ui.button("openFolder", text::OPEN_FOLDER, ButtonLook::Link);
                ids.open_folder = r.id;
                if r.clicked {
                    s.want = Some(Want::OpenFolder);
                }
                let r = ui.button("closeNotice", text::CLOSE_NOTICE, ButtonLook::Link);
                ids.close_notice = r.id;
                if r.clicked {
                    s.steps = Steps::Empty;
                }
            });
        }
    });
}

/// The section in the frame (`#cut`, lines 284-291 and 2205-2332): over the whole frame a column of the strip
/// (the whole cut drawn small, with the frame of the part looked at), its note, and the cut at true scale. The
/// pictures are uploaded by the app (`cut::nav_image`, `cut::sheet`); here they are placed. The cut view's box is
/// arithmetic before layout, as `side` is, because the look is settled against it before anything reads the look:
/// the frame less the strip (56), the note (its wrapped lines of 18 px and 8 px under) and two gaps of 8.
fn cut_frame(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme, world: &World, shown: &Shown, size: (f32, f32)) {
    let section = world.section.as_ref().expect("a cut is shown only for a world with a section");
    let (w, h) = size;
    let length = section.length;
    let ground = &shown.ground;
    let range = cut::nav_range(section, ground);
    let note_text = text::nav_note(&text::times(cut::nav_times(length, w as f64, range)));
    let muted = TextLook::body(Tone::Muted);
    let lines = wrap_lines(&note_text, muted, w).len() as f32;
    let box_h = (h - NAV_H - 8.0 - 18.0 * lines - 8.0 - 8.0).max(0.0);
    let (bw, bh) = (w as f64, box_h as f64);
    s.cut_look = cut::settled(s.cut_look, bw, bh, ground, length);
    ids.cut_box = (bw, bh);
    let (line, text_colour) = (t.line, t.text);

    let column = Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(0.0), right: Dim::Px(0.0), top: Dim::Px(0.0), bottom: Dim::Px(0.0) },
        column: true,
        gap: Vec2::new(0.0, 8.0),
        ..Style::default()
    };
    ids.cut = ui.node("cut", column, |ui| {
        // `#nav` (line 285): 56 px with a hairline under it; its canvas shows in the 55 px above the hairline.
        let nav_style = Style { h: Dim::Px(NAV_H), shrink: 0.0, ..Style::default() };
        ids.nav = ui
            .canvas("nav", nav_style, |ui, resp| {
                if let Some(rect) = resp.rect {
                    let pointer = ui.pointer();
                    // A click brings the window there; a press on the frame (3 px of grace) carries it along (2319-2332).
                    if let (true, Some(p)) = (resp.pressed, pointer) {
                        let x = (p.x - rect.x) as f64;
                        let frame = cut::nav_frame(s.cut_look, bw, bh, length, ground, section, range, bw).rect();
                        let (left, right) = (frame.x as f64, (frame.x + frame.w) as f64);
                        let inside = x >= left - 3.0 && x <= right + 3.0;
                        s.nav_carry = Some(if inside { x - left } else { frame.w as f64 / 2.0 });
                    }
                    if let (Some(grip), Some(p)) = (s.nav_carry, pointer) {
                        let go = cut::nav_go(s.cut_look, grip, (p.x - rect.x) as f64, bw, length);
                        s.cut_look = cut::settled(go, bw, bh, ground, length);
                    }
                    if !resp.held {
                        s.nav_carry = None;
                    }
                }
                let frame = cut::nav_frame(s.cut_look, bw, bh, length, ground, section, range, bw).rect();
                vec![
                    Cmd::Image { image: NAV_IMAGE, rect: Rect::new(0.0, 0.0, w, NAV_H - 1.0) },
                    // `outline: 1px solid var(--text)` inside the frame's box.
                    Cmd::Frame { rect: frame, width_px: 1.0, color: text_colour },
                    Cmd::Fill { rect: Rect::new(0.0, NAV_H - 1.0, w, 1.0), color: line },
                ]
            })
            .id;
        // `#navNote` (line 289): 12 px muted, 8 px under it.
        let note_style = Style { shrink: 0.0, margin: Margin { bottom: Side::Px(8.0), ..Margin::default() }, ..Style::default() };
        ids.nav_note = ui.node("navNote", note_style, |ui| ui.paragraph(&note_text, muted));
        // `#cutView` (line 290): takes what is left; its canvas shows in the 1 px less above the hairline.
        let view_style = Style { grow: 1.0, basis: Dim::Px(0.0), min_h: Dim::Px(0.0), ..Style::default() };
        ids.cut_view = ui
            .canvas("cutView", view_style, |ui, resp| {
                if let Some(rect) = resp.rect {
                    // The wheel comes closer to the point under it (2467-2474).
                    if let Some((notches, p)) = ui.take_wheel(resp.id) {
                        let at = ((p.x - rect.x) as f64, (p.y - rect.y) as f64);
                        s.cut_look = cut::wheel(s.cut_look, at, notches, bw, bh, ground, length);
                    }
                    let pointer = ui.pointer();
                    if let (true, Some(p)) = (resp.pressed, pointer) {
                        s.cut_carry = Some((p, s.cut_look));
                    }
                    // Read until the press is over, not only while held, so the last movement before the release counts.
                    if let (Some((from, start)), Some(p)) = (s.cut_carry, pointer) {
                        s.cut_look = cut::dragged(start, (p.x - from.x) as f64, (p.y - from.y) as f64, bw, bh, ground, length);
                    }
                    if !resp.held {
                        s.cut_carry = None;
                    }
                }
                let mut cmds = vec![Cmd::PushClip { rect: Rect::new(0.0, 0.0, w, box_h - 1.0) }];
                if let Some(sight) = cut::sight(s.cut_look, bw, bh, length, ground.rows) {
                    cmds.push(Cmd::Image { image: CUT_IMAGE, rect: cut::sheet_rect(s.cut_look, sight, bh) });
                }
                cmds.push(Cmd::PopClip);
                cmds.push(Cmd::Fill { rect: Rect::new(0.0, box_h - 1.0, w, 1.0), color: line });
                cmds
            })
            .id;
    });
}

/// What `#probe` says in the section: the block under the pointer when it is over the cut, else the cut's line
/// (`restCut`, 2455-2459). Nothing until a cut is shown. While a drag is held the page freezes the line; here it
/// follows the pointer: UNVERIFIED small difference.
fn cut_probe_text(ui: &Ui, s: &PreviewState, ids: &ShellIds, world: Option<&World>, shown: Option<&Shown>) -> String {
    let (Some(w), Some(shown)) = (world, shown) else { return String::new() };
    let Some(section) = w.section.as_ref() else { return String::new() };
    let cut = &section.cuts[shown.at];
    let rest = || text::cut_rest(shown.at + 1, section.cuts.len(), &text::num(cut.z), &text::num(cut.x), &text::num(cut.x + section.length as f64));
    let (Some(p), Some(rect)) = (ui.pointer(), ui.rect(ids.cut_view)) else { return rest() };
    if !rect.contains(p) {
        return rest();
    }
    cut::probe(w, shown, s.cut_look, ((p.x - rect.x) as f64, (p.y - rect.y) as f64)).unwrap_or_else(rest)
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
    let rest = || text::area_probe(&text::num(w.span()), &text::num(w.span() / 8.0), map_shown && tick_at(w, s.look, s.cut_at).is_some());
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
        frame_cmds(world, view, &s.chosen[view.index()], s.look, side, grid, muted, s.cut_at)
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

/// The right column (lines 429-449): the figures and the history, then the list of height bands. In the section
/// the figures are not built (line 2533: they would describe the map) and the list is the ores of the cut.
fn right_column(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme, cols: &Columns, world: Option<&World>, shown: Option<&Shown>) {
    let cut = s.view == 5;
    if !cut {
        figures_block(ui, s, ids, t, cols);
    }

    // `#listBlock.rule` (lines 295-296, 440): a hairline on top, 9 px of padding, a gap of 6. In the section the
    // rule and the padding go (line 294).
    ids.list_block = ui.node("listBlock", Style { column: true, ..fills() }, |ui| {
        if !cut {
            ui.fill(rule(), t.edge);
        }
        let inner = Style { column: true, pad: Edges::new(if cut { 0.0 } else { 9.0 }, 0.0, 0.0, 0.0), gap: Vec2::new(0.0, 6.0), ..fills() };
        ui.node("listBody", inner, |ui| {
            let title = if cut { text::ORE_LIST[0] } else { text::LIST_HEIGHT[0] }.to_uppercase();
            let section = world.and_then(|w| w.section.as_ref());
            // Names and figures of the ore rows, in `ORE_KINDS` order; empty without a cut.
            let figures: Vec<(usize, String, String)> = match (cut, shown) {
                (true, Some(shown)) => (0..ORE_KINDS.len())
                    .filter_map(|k| {
                        let f = shown.play.per_chunk[k].as_ref()?;
                        let figure = |v: Option<f64>| v.map_or("\u{2013}".to_string(), |v| text::num_digits(v, 1));
                        Some((k, figure(f.low), figure(f.high)))
                    })
                    .collect(),
                _ => Vec::new(),
            };
            ui.node("listHead", column(), |ui| {
                ui.node("title", Style { pad: Edges::new(0.0, 0.0, 2.0, 0.0), ..Style::default() }, |ui| {
                    ui.text(&title, TextLook::heading());
                });
                // `#pick` (line 341): two lines of the small size at most.
                let pick = Style { h: Dim::Px(34.0), overflow: nui_layout::Overflow::Hidden, ..Style::default() };
                let words = match (cut, shown, section) {
                    (false, ..) => text::PICK_HINT.to_string(),
                    (true, Some(_), Some(sec)) if sec.unplayed() > 0 => text::ore_unplayed(sec.unplayed()),
                    (true, Some(_), _) => text::ORE_PICK_HINT.to_string(),
                    (true, None, _) => String::new(),
                };
                ids.pick = ui.node("pick", pick, |ui| ui.paragraph(&words, TextLook::small(Tone::Muted)));
            });
            ids.list_title = title;
            if cut {
                // Line 2370-2372: the legend names the columns; a list without a cut has no legend.
                if shown.is_some() {
                    ui.list_head("legend", text::ORE_LIST[1], text::ORE_COLUMNS[0], text::ORE_COLUMNS[1], cols.narrow);
                }
            } else {
                ui.list_head("legend", text::LIST_HEIGHT[1], text::COL_WIDTH, text::COL_SHARE, cols.narrow);
            }
            let r = ui.scroll("list", &mut s.list_y, scroll_style(), ScrollOpts::LIST, |ui| {
                if cut {
                    for (k, low, high) in &figures {
                        let name = ORE_KINDS[*k];
                        let row = ListRow { swatch: rgb(ORE_COLOURS[*k]), name: text::ORES[*k], width: low, share: high };
                        let opts = RowOpts { pressed: s.chosen_ores.contains(name), choosing: !s.chosen_ores.is_empty(), narrow: cols.narrow, ..RowOpts::default() };
                        let r = ui.list_row(&format!("ore{k}"), &row, opts);
                        ids.ores.push(r.id);
                        // A click picks the ore, or lets it go (2385).
                        if r.clicked && !s.chosen_ores.remove(name) {
                            s.chosen_ores.insert(name.to_string());
                        }
                    }
                    return;
                }
                let names = band_names(SEA, &HEIGHT_STEPS, &SEA_DEPTHS);
                let colours = band_colours(SEA, &HEIGHT_STEPS, &SEA_DEPTHS);
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

/// `#figuresBlock` (lines 430-439): the figures of the map and the history.
fn figures_block(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, t: &Theme, cols: &Columns) {
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
}
/// The shell as a picture: builds until the scrollbar's gutter has settled (it comes one frame late), at most
/// three times, and returns the last draw list with the `Ui` (for the rects) and the ids.
pub fn picture(theme: Theme, size: Vec2) -> (DrawList, Ui, ShellIds) {
    let mut ui = Ui::new(theme);
    let mut s = PreviewState::default();
    let mut last = None;
    for _ in 0..3 {
        ui.begin_frame(size);
        let ids = build(&mut ui, &mut s, size, None, None);
        let list = ui.end_frame().clone();
        last = Some((list, ids));
        if !ui.want_redraw() {
            break;
        }
    }
    let (list, ids) = last.expect("the loop runs at least once");
    (list, ui, ids)
}
