//! Each setting's help: the "?" after its label opens a panel under the row that stays open, shows the label as its
//! title and the hint under it, with a close cross on the title's line. Another "?" moves it, the same one, the cross
//! or Escape closes it. Built as the application builds it, with events fed to the `Ui` between frames; the two
//! pictures go through the offscreen renderer.

use nui_base::{Button, Cmd, Color, Cursor, DrawList, Event, Gfx, Key, Mods, Rect, Vec2};
use nui_ui::{Id, Theme, Ui};
use nui_win32::Renderer;
use preview::settings::SETTINGS;
use preview::shell::help_escape;
use preview::text::{HELP_MARK, SETTING_HINTS, setting_hint};
use preview::{PreviewState, ShellIds, build};

const SIZE: Vec2 = Vec2::new(1400.0, 940.0);

fn index(key: &str) -> usize {
    SETTINGS.iter().position(|s| s.key == key).expect("a setting of that key")
}

/// The shell with no world, built as `PreviewApp::frame` builds it: again while the state moves or a frame asks.
struct Rig {
    ui: Ui,
    s: PreviewState,
    ids: ShellIds,
    list: DrawList,
    size: Vec2,
}

impl Rig {
    fn new(theme: Theme, size: Vec2, s: PreviewState) -> Rig {
        let mut r = Rig { ui: Ui::new(theme), s, ids: ShellIds::default(), list: DrawList::default(), size };
        // The first frame learns where everything is: the pointer is judged against the last one.
        r.frame();
        r.frame();
        r
    }

    fn frame(&mut self) {
        for _ in 0..4 {
            let before = self.s.clone();
            self.ui.begin_frame(self.size);
            self.ids = build(&mut self.ui, &mut self.s, self.size, None, None, None);
            self.list = self.ui.end_frame().clone();
            if !self.ui.want_redraw() && self.s == before {
                break;
            }
        }
    }

    fn rect(&self, id: Id) -> Rect {
        assert_ne!(id, Id::ROOT, "the part is not built");
        self.ui.rect(id).expect("the part was laid out")
    }

    fn centre(&self, id: Id) -> Vec2 {
        let r = self.rect(id);
        Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    fn moved(&mut self, pos: Vec2) {
        self.ui.event(&Event::PointerMoved { pos });
    }

    /// A whole click: the pointer arrives, goes down, a frame, goes up, a frame.
    fn click(&mut self, pos: Vec2) {
        self.moved(pos);
        self.frame();
        self.ui.event(&Event::PointerDown { pos, button: Button::Left, clicks: 1 });
        self.frame();
        self.ui.event(&Event::PointerUp { pos, button: Button::Left });
        self.frame();
    }

    fn click_mark(&mut self, key: &str) {
        let at = self.centre(self.ids.help_marks[index(key)]);
        self.click(at);
    }

    /// The colours the "?" marks are drawn in this frame, in drawing order.
    fn marks(&self) -> Vec<Color> {
        self.list.cmds.iter().filter_map(|c| match c {
            Cmd::Text { text, color, .. } if text == HELP_MARK => Some(*color),
            _ => None,
        }).collect()
    }
}

fn near(a: f32, b: f32, what: &str) {
    assert!((a - b).abs() < 0.01, "{what}: {a} != {b}");
}

#[test]
fn h1_every_setting_has_a_short_hint_of_its_own() {
    assert_eq!(SETTING_HINTS.len(), SETTINGS.len());
    for s in &SETTINGS {
        let hint = setting_hint(s.key);
        assert!(!hint.is_empty(), "{} has no hint", s.key);
        // One or two short sentences, each ended.
        assert!(hint.ends_with('.'), "{}: {hint}", s.key);
        let sentences = hint.matches(". ").count() + 1;
        assert!(sentences <= 2 && hint.len() <= 120, "{} is too long: {hint}", s.key);
    }
    for (i, (k, _)) in SETTING_HINTS.iter().enumerate() {
        assert!(SETTING_HINTS[..i].iter().all(|(o, _)| o != k), "{k} twice");
    }
    assert_eq!(setting_hint("no_such_setting"), "");
}

#[test]
fn h2_the_mark_opens_the_help_under_its_row_and_it_stays() {
    let mut r = Rig::new(Theme::dark(), SIZE, PreviewState::default());
    assert_eq!(r.ids.help, Id::ROOT, "no help at first");
    r.click_mark("vegetation_scale");
    let i = index("vegetation_scale");
    assert_eq!(r.s.help, Some(i));
    // Under its row, as wide, the row above it and the next row under it.
    let (row, panel) = (r.rect(r.ids.rows[i]), r.rect(r.ids.help));
    near(panel.x, row.x, "panel x");
    near(panel.w, row.w, "panel w");
    assert!(panel.y >= row.y + row.h, "the panel is under its row");
    assert!(r.rect(r.ids.rows[i + 1]).y >= panel.y + panel.h, "the next row is under the panel");
    // The pointer leaving and more frames do not close it.
    r.moved(Vec2::new(700.0, 500.0));
    r.frame();
    r.frame();
    assert_eq!(r.s.help, Some(i), "it stays open until closed");
    // A click elsewhere in the page does not close it either.
    r.click(Vec2::new(700.0, 500.0));
    assert_eq!(r.s.help, Some(i));
}

#[test]
fn h3_another_mark_switches_the_same_one_closes() {
    let mut r = Rig::new(Theme::dark(), SIZE, PreviewState::default());
    r.click_mark("temperature_scale");
    assert_eq!(r.s.help, Some(index("temperature_scale")));
    // In another group: the help moves there, one panel only.
    r.click_mark("vertical_scale");
    let v = index("vertical_scale");
    assert_eq!(r.s.help, Some(v));
    let row = r.rect(r.ids.rows[v]);
    assert!(r.rect(r.ids.help).y >= row.y + row.h, "the panel moved under its new row");
    // In the same group.
    r.click_mark("elevation_boost");
    assert_eq!(r.s.help, Some(index("elevation_boost")));
    // The same mark again closes it.
    r.click_mark("elevation_boost");
    assert_eq!(r.s.help, None);
    assert_eq!(r.ids.help, Id::ROOT, "no panel is built once closed");
}

#[test]
fn h4_the_cross_closes_it() {
    let mut r = Rig::new(Theme::light(), SIZE, PreviewState::default());
    r.click_mark("temperature_offset");
    assert!(r.s.help.is_some());
    let at = r.centre(r.ids.help_close);
    r.click(at);
    assert_eq!(r.s.help, None);
    assert_eq!(r.ids.help, Id::ROOT);
}

#[test]
fn h5_escape_closes_it_after_a_field_has_had_it() {
    let mut r = Rig::new(Theme::dark(), SIZE, PreviewState::default());
    let escape = Event::Key { key: Key::Escape, mods: Mods::default() };
    assert!(!help_escape(&mut r.s, &r.ui, &escape), "nothing open, nothing taken");
    r.click_mark("ridge_scale");
    assert!(r.s.help.is_some());
    // A key other than Escape does nothing.
    assert!(!help_escape(&mut r.s, &r.ui, &Event::Key { key: Key::Enter, mods: Mods::default() }));
    // A focused field takes Escape first (it puts its value back and lets go of the keyboard).
    let seed = r.centre(r.ids.seed);
    r.click(seed);
    assert!(r.ui.focused().is_some(), "the seed field has the keyboard");
    assert!(!help_escape(&mut r.s, &r.ui, &escape));
    assert!(r.s.help.is_some());
    // Once nothing has the keyboard, Escape closes the help.
    r.click(Vec2::new(700.0, 500.0));
    assert!(r.ui.focused().is_none());
    assert!(help_escape(&mut r.s, &r.ui, &escape));
    assert_eq!(r.s.help, None);
}

#[test]
fn h6_the_mark_lights_under_the_pointer_and_while_open() {
    let t = Theme::dark();
    let mut r = Rig::new(t, SIZE, PreviewState::default());
    let i = index("temperature_multiplier");
    // At rest no mark shows.
    assert!(r.marks().is_empty(), "hidden at rest");
    // The pointer on a row shows its mark only, muted.
    let row = r.centre(r.ids.rows[i]);
    r.moved(row);
    r.frame();
    assert_eq!(r.marks(), vec![t.muted], "the row's mark shows, muted");
    let at = r.centre(r.ids.help_marks[i]);
    r.moved(at);
    r.frame();
    assert_eq!(r.ui.cursor(), Cursor::Hand, "the hand of what can be clicked");
    assert_eq!(r.marks().iter().filter(|&&c| c == t.text).count(), 1, "the one under the pointer lights");
    // Away from it, the open one stays lit: it says which help is open.
    r.click(at);
    r.moved(Vec2::new(700.0, 500.0));
    r.frame();
    assert_eq!(r.ui.cursor(), Cursor::Arrow);
    assert_eq!(r.marks().iter().filter(|&&c| c == t.text).count(), 1, "the open one is lit");
    // The cross lights and shows the hand too.
    let cross = r.centre(r.ids.help_close);
    r.moved(cross);
    r.frame();
    assert_eq!(r.ui.cursor(), Cursor::Hand);
}

/// The panel's parts: the title on the first line with the cross at its top right, inside the panel's padding, and
/// the hint under them, wrapped within the panel.
fn check_panel(r: &Rig, what: &str) {
    let (panel, title, cross, hint) = (r.rect(r.ids.help), r.rect(r.ids.help_title), r.rect(r.ids.help_close), r.rect(r.ids.help_text));
    // The frame (1) and the inset (8).
    let pad = 9.0;
    near(cross.y, panel.y + pad, &format!("{what}: the cross on the panel's top margin"));
    near(title.y, cross.y, &format!("{what}: the cross on the title's first line"));
    near(cross.x + cross.w, panel.x + panel.w - pad, &format!("{what}: the cross at the right"));
    near(title.x, panel.x + pad, &format!("{what}: the title at the left"));
    assert!(title.x + title.w <= cross.x, "{what}: the title never runs under the cross");
    assert!(hint.y >= title.y + title.h, "{what}: the hint is under the title");
    assert!(hint.x >= panel.x + pad && hint.x + hint.w <= panel.x + panel.w - pad + 0.01, "{what}: the hint stays inside");
    assert!(panel.y + panel.h >= hint.y + hint.h + pad - 0.01, "{what}: the panel holds the hint");
}

#[test]
fn h7_a_long_hint_wraps_and_the_cross_keeps_its_line() {
    // A long hint at the natural width, and a long title in the narrowest stacked panel, where it wraps too.
    let wide = PreviewState { help: Some(index("ultrasmooth")), help_reveal: true, ..PreviewState::default() };
    let r = Rig::new(Theme::dark(), SIZE, wide);
    check_panel(&r, "wide");
    assert!(r.rect(r.ids.help_text).h >= 3.0 * 18.0, "the long hint wraps onto several lines");

    let narrow = PreviewState { help: Some(index("river_lanterns")), help_reveal: true, ..PreviewState::default() };
    let r = Rig::new(Theme::dark(), Vec2::new(preview::MIN_SIZE.0, 800.0), narrow);
    check_panel(&r, "narrow");
    assert!(r.rect(r.ids.help_title).h >= 2.0 * 18.0, "the long title wraps beside the cross");
}

#[test]
fn h8_a_help_low_in_the_list_scrolls_into_sight() {
    let s = PreviewState { help: Some(index("remove_frozen_ocean_ice")), help_reveal: true, ..PreviewState::default() };
    let r = Rig::new(Theme::dark(), SIZE, s);
    assert!(r.s.settings_y > 0.0, "the list scrolled");
    assert!(!r.s.help_reveal, "once");
    let (panel, list) = (r.rect(r.ids.help), r.rect(r.ids.settings_scroll));
    assert!(panel.y + panel.h <= list.y + list.h, "the panel is in sight: {panel:?} in {list:?}");
    assert!(panel.y >= list.y);
}

fn picture(theme: Theme, size: Vec2, key: &str, name: &str) -> (Vec<u8>, Rig) {
    let s = PreviewState { help: Some(index(key)), help_reveal: true, ..PreviewState::default() };
    let r = Rig::new(theme, size, s);
    let (w, h) = (size.x as u32, size.y as u32);
    let mut out = Renderer::offscreen(w, h, 1.0).expect("offscreen renderer");
    out.begin(theme.ground);
    out.draw_list(&r.list);
    out.end();
    let rgba = out.read_rgba().unwrap();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, nui_png::encode(w, h, &rgba)).unwrap();
    println!("wrote {}", path.display());
    (rgba, r)
}

fn px(rgba: &[u8], w: f32, x: f32, y: f32) -> [u8; 3] {
    let i = (y as usize * w as usize + x as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn same(got: [u8; 3], want: Color, what: &str) {
    let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
    assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what}: got {got:?}, want {w:?}");
}

#[test]
fn h9_pictures_in_both_themes() {
    for (theme, tag) in [(Theme::dark(), "dark"), (Theme::light(), "light")] {
        for (size, key, at) in [(SIZE, "ultrasmooth", "wide"), (Vec2::new(preview::MIN_SIZE.0, 800.0), "river_lanterns", "narrow")] {
            let (rgba, r) = picture(theme, size, key, &format!("setting_help_{at}_{tag}.png"));
            let panel = r.rect(r.ids.help);
            // The frame's hairlines, in the edge colour, and the ground inside.
            same(px(&rgba, size.x, panel.x + panel.w / 2.0, panel.y), theme.edge, "top hairline");
            same(px(&rgba, size.x, panel.x, panel.y + panel.h / 2.0), theme.edge, "left hairline");
            same(px(&rgba, size.x, panel.x + 3.0, panel.y + 3.0), theme.ground, "inside");
            // The cross's middle pixel is drawn in the muted colour at rest.
            let c = r.rect(r.ids.help_close);
            same(px(&rgba, size.x, c.x + c.w / 2.0 - 1.0, c.y + c.h / 2.0 - 1.0), theme.muted, "the cross");
        }
    }
}
