//! The launcher's `#titlebar` (launcher/index.html lines 75-129 and 221-245), built by the preview as the
//! window's host: the game's name on the left, the tools in the middle with the preview's own tabs after
//! the preview tool, the window's buttons on the right. Step 21 moves this to the launcher; until then the
//! preview is the only application, so it hosts its own bar.
//!
//! The icon paths below come from the launcher page (the line numbers are its), which draws them after
//! Pixelarticons (MIT): see THIRD-PARTY.md.
//!
//! The bar has no breakpoint. The launcher's `condense()` turns the views' names into icons when the bar is
//! too narrow, which is the page's own breakpoint and is not built: the window's minimum width holds the
//! words, and only the two free spaces beside the tools grow or shrink.

use std::sync::OnceLock;

use nui_base::{Cmd, Color, Rect, Vec2};
use nui_layout::{Align, Dim, Display, Edges, Position, Style, Track, Tracks};
use nui_ui::{Icon, IconLook, IconOpts, TabLook, TabOpts, TextLook, Ui, strip_style};

use crate::shell::{BAR_H, Mode, PreviewState, RunEnd, ShellIds, Want};
use crate::text;
use crate::world::World;

/// Line 229.
pub const HOME: &str = "M4 20h16v2H4zm16-10h2v10h-2zM2 10h2v10H2zm2-2h2v2H4zm2-2h2v2H6zm2-2h2v2H8zm2-2h4v2h-4zm4 2h2v2h-2zm2 2h2v2h-2zm2 2h2v2h-2zM8 14h2v6H8zm2-2h4v2h-4zm4 2h2v6h-2z";
/// Line 234.
pub const PREVIEW: &str = "M4 20h2v2H2V6h2v14Zm12 0h2v2h-4v-2h-2v-2h2V8h-2V6h4v14Zm-8 0H6v-2h2v2Zm12 0h-2v-2h2v2ZM10 4h2v2h-2v10h2v2H8V4H6V2h4v2Zm12 14h-2V4h-2V2h4v16ZM6 6H4V4h2v2Zm12 0h-2V4h2v2Z";
/// Line 240.
pub const MINIMIZE: &str = "M5 11h14v2H5z";
/// Line 735.
pub const MAXIMIZE: &str = "M4 4h16v2H4zm0 14h16v2H4zM4 6h2v12H4zm14 0h2v12h-2z";
/// Line 736.
pub const RESTORE: &str = "M8 4h12v2H8zm10 2h2v8h-2zm-2 6h2v2h-2zM8 6h2v2H8zM4 8h12v2H4zm0 10h12v2H4zM4 10h2v8H4zm10 0h2v8h-2z";
/// Line 242.
pub const CLOSE: &str = "M5 5h2v2H5zm2 2h2v2H7zm2 2h2v2H9zm2 2h2v2h-2zm2-2h2v2h-2zm2-2h2v2h-2zm2-2h2v2h-2zM9 13h2v2H9zm-2 2h2v2H7zm-2 2h2v2H5zm8-4h2v2h-2zm2 2h2v2h-2zm2 2h2v2h-2z";
/// Lines 427-432: the views, by key (biomes, flat, height, temperature, structures, section).
pub const VIEW_ICONS: [(&str, &str); 6] = [
    ("biomes", "M8 2h8v2H8zM6 4h12v8H6zM8 12h8v2H8zM11 14h2v6h-2zM5 20h14v2H5z"),
    ("flat", "M2 12h20v2H2zM2 16h20v2H2zM2 20h20v2H2zM8 6h8v2H8z"),
    ("height", "M10 4h4v2h-4zM8 6h2v2H8zm6 0h2v2h-2zM6 8h2v4H6zm10 0h2v4h-2zM4 12h2v4H4zm14 0h2v4h-2zM2 16h2v4H2zm18 0h2v4h-2zM2 20h20v2H2z"),
    ("temperature", "M10 2h4v2h-4zM8 4h2v10H8zm6 0h2v10h-2zM6 14h2v6H6zm10 0h2v6h-2zM8 20h8v2H8zM11 8h2v8h-2zM9 16h6v2H9z"),
    ("structures", "M4 2h4v2h2V2h4v2h2V2h4v6H4zM6 8h2v12H6zm10 0h2v12h-2zM10 14h4v6h-4zM2 20h20v2H2z"),
    ("section", "M2 4h20v2H2zM2 18h20v2H2zM2 6h2v12H2zm18 0h2v12h-2zM4 12h4v2H4zm4-2h4v2H8zm4 2h4v2h-4zm4-4h4v2h-4z"),
];
/// Line 418.
pub const PLAY: &str = "M15 11h-2V9h2zm0 4h-2v-2h2zm-2 2h-2v-2h2zm0-8h-2V7h2zm-2-2H9V5h2zM9 21H7V3h2zm6-8h2v-2h-2zm-6 4h2v2H9z";
/// Line 419.
pub const STOP: &str = "M6 6h12v12H6z";
/// Line 420.
pub const SETTINGS: &str = "M8 14H7v6H5v-6H2v-2h6v2Zm5 6h-2V10h2v10Zm9-2h-3v2h-2v-2h-1v-2h6v2Zm-3-4h-2V4h2v10ZM7 10H5V4h2v6Zm6-4h2v2H9V6h2V4h2v2Z";
/// Line 223: the pack's cube, in a 32 unit grid, each face in its own colour (`fill`).
pub const PACK: [(&str, u32); 3] = [("M16 2l14 7-14 7L2 9z", 0x7fb850), ("M2 9l14 7v14L2 23z", 0x8b6344), ("M30 9l-14 7v14l14-7z", 0x6b4b33)];

/// The icons, parsed once.
pub struct Icons {
    pub home: Icon,
    pub preview: Icon,
    pub minimize: Icon,
    pub maximize: Icon,
    pub restore: Icon,
    pub close: Icon,
    pub views: Vec<Icon>,
    pub play: Icon,
    pub stop: Icon,
    pub settings: Icon,
    /// The three faces of the pack's cube with their colours.
    pub pack: Vec<(Icon, Color)>,
}

/// The path constants are the launcher's own and parse, which a test checks; a failure here would be a typo made
/// in copying, so it is a logged empty icon rather than a panic.
fn parse(d: &str, grid: f32) -> Icon {
    Icon::parse(d, grid).unwrap_or_else(|e| {
        nui_base::observe!(Error, "icon path '{d}': {e}");
        Icon { grid, figures: Vec::new() }
    })
}

pub fn icons() -> &'static Icons {
    static ICONS: OnceLock<Icons> = OnceLock::new();
    ICONS.get_or_init(|| Icons {
        home: parse(HOME, 24.0),
        preview: parse(PREVIEW, 24.0),
        minimize: parse(MINIMIZE, 24.0),
        maximize: parse(MAXIMIZE, 24.0),
        restore: parse(RESTORE, 24.0),
        close: parse(CLOSE, 24.0),
        views: VIEW_ICONS.iter().map(|(_, d)| parse(d, 24.0)).collect(),
        play: parse(PLAY, 24.0),
        stop: parse(STOP, 24.0),
        settings: parse(SETTINGS, 24.0),
        pack: PACK.iter().map(|(d, c)| (parse(d, 32.0), Color::hex(*c))).collect(),
    })
}

/// Which marker the preview tool carries (renderModes, lines 784-787): running while a run is under way, else the
/// way the last one ended while the person has not looked at the preview since.
pub fn marker(s: &PreviewState) -> Option<usize> {
    if s.follow.is_some() {
        Some(0)
    } else if s.preview_seen {
        None
    } else {
        match s.ended {
            RunEnd::Ready => Some(1),
            RunEnd::Failed => Some(2),
            RunEnd::Idle => None,
        }
    }
}

/// What the window's title says (line 794).
pub fn window_title(s: &PreviewState) -> String {
    match marker(s) {
        Some(m) => text::title_with(text::PREVIEW_STATE[m]),
        None => text::BAR_TITLE.to_string(),
    }
}

/// Builds the bar and fills `ids` with where its parts are.
pub fn build(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, world: Option<&World>) {
    let t = *ui.theme();
    let icons = icons();
    let marker = marker(s);
    // The bar is a grid of three columns with a free space on each side of the tools, so the tools stay in the
    // middle whatever the width (line 76). The launcher page has a 1 px border-bottom on #titlebar (line 77); the
    // native bar drops it by the maintainer's choice (2026-11-15: the header should float, no separating line), a
    // deliberate departure from the page (the wry page in the pack's repository is not changed). The 1 px bottom padding
    // is kept so the parts stay where the page centres them above its border: removing it would move them 0.5 px.
    let bar = Style {
        display: Display::Grid,
        cols: Tracks::new(&[Track::MinMaxFr0(1.0), Track::Auto, Track::MinMaxFr0(1.0)]),
        align_items: Align::Center,
        h: Dim::Px(BAR_H),
        shrink: 0.0,
        pad: Edges::new(0.0, 0.0, 1.0, 0.0),
        position: Position::Relative,
        ..Style::default()
    };
    ids.bar = ui.node("bar", bar, |ui| {
        // `.brand` (lines 81-82): the cube and the name in capitals, 12 px in from the left, 10 px apart.
        // Its own width, not the column's: the page's stretches it over the free space, which nothing can see.
        let brand = Style { align_items: Align::Center, gap: Vec2::new(10.0, 0.0), pad: Edges::new(0.0, 0.0, 0.0, 12.0), min_w: Dim::Px(0.0), justify_self: Some(Align::Start), ..Style::default() };
        ids.brand = ui.node("brand", brand, |ui| {
            let cube = Style { w: Dim::Px(16.0), h: Dim::Px(16.0), shrink: 0.0, ..Style::default() };
            ui.canvas("pack", cube, |_, _| {
                icons.pack.iter().map(|(i, c)| Cmd::Shape { rect: Rect::new(0.0, 0.0, 16.0, 16.0), grid: i.grid, figures: i.figures.clone(), color: *c, crisp: false }).collect()
            });
            ui.text(&text::BAR_TITLE.to_uppercase(), TextLook::heading());
        });

        // `.strip` (line 109): the tools, one box wide each, then the preview's tabs after its own tool.
        ids.strip = ui.node("tools", strip_style(), |ui| {
            let tool = |ui: &mut Ui, ids: &mut ShellIds, s: &mut PreviewState, which: usize| {
                let (mode, icon) = if which == 0 { (Mode::Home, &icons.home) } else { (Mode::Preview, &icons.preview) };
                let word = if which == 1 { marker.map(|m| text::PREVIEW_STATE[m]) } else { None };
                let dot = word.map(|_| if marker == Some(2) { t.warn } else { t.text });
                let r = ui.icon_button(&format!("tool{which}"), icon, IconLook::Tool, IconOpts { pressed: s.mode == mode, disabled: false, joined: false, marker: dot });
                ids.tools[which] = r.id;
                ids.bar_buttons.push(r.id);
                let name = text::MODES[which];
                ui.title(r.id, &word.map_or(name.to_string(), |w| text::mode_with(name, w)));
                // `showMode` (line 797): the same mode does nothing; the preview, once shown, has been seen.
                if r.clicked && s.mode != mode {
                    s.mode = mode;
                    if mode == Mode::Preview {
                        s.preview_seen = true;
                    }
                }
            };
            ui.node("home", Style::default(), |ui| tool(ui, ids, s, 0));
            ui.strip_gap();
            ui.node("preview", Style::default(), |ui| {
                tool(ui, ids, s, 1);
                // A closed tool's choices are 0 wide and not built.
                if s.mode == Mode::Preview {
                    ui.node("options", Style { pad: Edges::new(0.0, 0.0, 0.0, 1.0), ..Style::default() }, |ui| tabs(ui, s, ids, world));
                }
            });
        });

        // `.ends` (line 89): the window's buttons, as tall as the bar and flush with its corner.
        let ends = Style { justify_self: Some(Align::End), align_self: Some(Align::Stretch), ..Style::default() };
        ui.node("ends", ends, |ui| {
            let r = ui.icon_button("minimize", &icons.minimize, IconLook::Caption, IconOpts::default());
            ids.ends[0] = r.id;
            ui.title(r.id, text::MINIMIZE);
            if r.clicked {
                s.want = Some(Want::Minimize);
            }
            let (icon, word) = if s.maximized { (&icons.restore, text::RESTORE) } else { (&icons.maximize, text::MAXIMIZE) };
            let r = ui.icon_button("maximize", icon, IconLook::Caption, IconOpts::default());
            ids.ends[1] = r.id;
            ui.title(r.id, word);
            if r.clicked {
                s.want = Some(Want::Maximize);
            }
            let r = ui.icon_button("close", &icons.close, IconLook::Close, IconOpts::default());
            ids.ends[2] = r.id;
            ui.title(r.id, text::CLOSE_WINDOW);
            if r.clicked {
                s.want = Some(Want::Close);
            }
            ids.bar_buttons.extend(ids.ends);
        });
    });
}

/// The preview's tabs as the page sends them to its host (lines 749-754): the six views, then, outside the section,
/// a gap and 2D and 3D. A disabled one carries the page's `needsFull` as its title, an enabled one its label.
fn tabs(ui: &mut Ui, s: &mut PreviewState, ids: &mut ShellIds, world: Option<&World>) {
    let views = text::VIEWS.iter().copied().chain([text::SECTION]);
    // Flat, Structures and Section come after the full map only (line 2506).
    let coarse = world.is_some_and(World::coarse);
    for (i, word) in views.enumerate() {
        let disabled = coarse && matches!(i, 1 | 4 | 5);
        let r = ui.tab(&format!("view{i}"), word, TabLook::Strip, TabOpts { pressed: i == s.view, disabled });
        ids.tabs.push(r.id);
        ids.bar_buttons.push(r.id);
        ui.title(r.id, if disabled { text::NEEDS_FULL } else { word });
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
            ids.bar_buttons.push(r.id);
            ui.title(r.id, word);
            if r.clicked {
                s.in3d = i == 1;
            }
        }
    }
}
