//! The bar when the window is too narrow for all of it: what it gives up, in which order, and that what it keeps
//! stays whole. Layout is judged on the real rects of a built frame, at 1000, 1280, 1920 and 2560 px wide.
//!
//! The world the tests load (`C:\Games\preview\out`) was stopped at a coarse detail, so Flat, Structures and Section
//! are greyed in it; `None` stands for a world with every view on.

mod common;

use nui_base::{App, Button, Cmd, Event, Rect, Vec2};
use nui_ui::{Id, MENU_ROW, Theme, Ui, tool_button_width};
use nui_win32::Renderer;
use preview::overflow::{self, Form, Inputs};
use preview::world::World;
use preview::{Mode, PreviewApp, PreviewState, ShellIds, picture_with, text};

const WIDTHS: [f32; 4] = [1000.0, 1280.0, 1920.0, 2560.0];
const LONG: &str = "AVeryLongAccountNameThatCannotFit";

fn rect_of(ui: &Ui, id: Id) -> Rect {
    assert_ne!(id, Id::ROOT);
    ui.rect(id).expect("built")
}

fn state(account: Option<&str>, view: usize) -> PreviewState {
    PreviewState { account: account.map(String::from), view, reduce_motion: true, ..PreviewState::default() }
}

fn built(w: f32, st: PreviewState, world: Option<&World>) -> (nui_base::DrawList, Ui, ShellIds) {
    picture_with(Theme::dark(), Vec2::new(w, 800.0), world, st)
}

fn drawn(list: &nui_base::DrawList) -> Vec<String> {
    list.cmds.iter().filter_map(|c| if let Cmd::Text { text, .. } = c { Some(text.clone()) } else { None }).collect()
}

fn inputs(w: f32, account: Option<&str>, pressed: usize, disabled: [bool; 6]) -> Inputs<'_> {
    Inputs { width: w, ends: overflow::ENDS_W, account, preview_open: true, pressed, disabled, shapes: pressed != 5 }
}

/// Every distinct form met while the window narrows from 2560 to 1000, widest first.
fn forms_while_narrowing(account: Option<&str>, pressed: usize, disabled: [bool; 6]) -> Vec<Form> {
    let mut seen: Vec<Form> = Vec::new();
    for w in (1000..=2560).rev() {
        let f = overflow::plan(&inputs(w as f32, account, pressed, disabled));
        if seen.last() != Some(&f) {
            seen.push(f);
        }
    }
    seen
}

fn form(account: bool, word: bool, more: &[usize]) -> Form {
    Form { account, word, more: more.to_vec() }
}

/// The widths the plan counts are the widths the layout gives: the strip, for every form it can take.
#[test]
fn the_measured_strip_is_the_laid_out_strip() {
    let coarse = common::world();
    let mut w = 1000.0;
    let mut checked = 0;
    while w <= 2560.0 {
        for world in [coarse.as_ref(), None] {
            for view in [0, 2, 5] {
                for account in [None, Some(LONG)] {
                    if world.is_some_and(World::coarse) && matches!(view, 1 | 4 | 5) {
                        continue;
                    }
                    let (_, ui, ids) = built(w, state(account, view), world);
                    let disabled: [bool; 6] = std::array::from_fn(|i| world.is_some_and(World::coarse) && matches!(i, 1 | 4 | 5));
                    let predicted = overflow::strip_width(&inputs(w, account, view, disabled), &ids.form.more);
                    let got = rect_of(&ui, ids.strip).w;
                    assert!((predicted - got).abs() < 0.01, "w {w} view {view} {account:?}: predicted {predicted}, laid out {got}");
                    checked += 1;
                }
            }
        }
        w += 53.0;
    }
    assert!(checked > 100, "{checked}");
}

/// The order of giving way: the account, the greyed views, the word, then the others from the strip's end, with
/// the pressed view (Relief here) never among them.
#[test]
fn the_bar_gives_way_in_its_order() {
    let coarse = [false, true, false, false, true, true];
    assert_eq!(
        forms_while_narrowing(Some("Louis"), 2, coarse),
        vec![
            form(true, true, &[]),
            form(false, true, &[]),
            form(false, true, &[1, 4, 5]),
            form(false, false, &[1, 4, 5]),
            form(false, false, &[1, 3, 4, 5]),
            form(false, false, &[0, 1, 3, 4, 5]),
        ][..forms_while_narrowing(Some("Louis"), 2, coarse).len()],
        "greyed world"
    );
    // With every view on there is nothing greyed to move: the word goes, then the views from the end.
    let all_on = forms_while_narrowing(Some("Louis"), 2, [false; 6]);
    let expected = [
        form(true, true, &[]),
        form(false, true, &[]),
        form(false, true, &[5]),
        form(false, true, &[4, 5]),
    ];
    assert_eq!(all_on, expected, "every view on: the word stays, it buys no room next to the window's buttons");
    // The pressed view is never moved: Section pressed, the others go from Structures back.
    let section = forms_while_narrowing(None, 5, [false; 6]);
    assert!(section.iter().all(|f| !f.more.contains(&5)), "{section:?}");
    // Structures pressed: the last view goes first, then the one before the pressed one is passed over.
    let structures = forms_while_narrowing(None, 4, [false; 6]);
    assert!(structures.iter().all(|f| !f.more.contains(&4)), "{structures:?}");
    assert_eq!(structures.iter().find(|f| !f.more.is_empty()).map(|f| f.more.clone()), Some(vec![5]));
}

/// The word comes before the other views when the brand is what is widest: with the window's buttons made narrow
/// (they never are, at 138 px), the account goes, then the word, then the views from the end.
#[test]
fn the_word_goes_before_the_other_views_when_it_buys_room() {
    let mut seen: Vec<Form> = Vec::new();
    for w in (500..=2560).rev() {
        let i = Inputs { ends: 40.0, ..inputs(w as f32, Some("Louis"), 2, [false; 6]) };
        let f = overflow::plan(&i);
        if seen.last() != Some(&f) {
            seen.push(f);
        }
    }
    assert_eq!(&seen[..4], &[form(true, true, &[]), form(false, true, &[]), form(false, false, &[]), form(false, false, &[5])]);
}

/// A form is only taken once it fits: at every width the first form of the order that fits is the one chosen, so
/// the bar never gives up more than it must.
#[test]
fn the_bar_takes_the_first_form_that_fits() {
    let disabled = [false, true, false, false, true, true];
    for w in (1000..=2560).step_by(3) {
        let i = inputs(w as f32, Some("Louis"), 2, disabled);
        let chosen = overflow::plan(&i);
        let order = forms_while_narrowing(Some("Louis"), 2, disabled);
        let at = order.iter().position(|f| *f == chosen).expect("a form of the order");
        assert!(order[..at].iter().all(|f| !overflow::fits(&i, f)), "{w}: a form before {chosen:?} fits");
    }
}

/// At the four widths: nothing overlaps, both tools say their names, and the form is the expected one.
#[test]
fn the_bar_fits_at_every_width_with_the_tools_named() {
    let world = common::world();
    for w in WIDTHS {
        for (name, world) in [("coarse", world.as_ref()), ("all views", None)] {
            for account in [None, Some("Louis"), Some(LONG)] {
                let (list, ui, ids) = built(w, state(account, 2), world);
                let (brand, strip, bar) = (rect_of(&ui, ids.brand), rect_of(&ui, ids.strip), rect_of(&ui, ids.bar));
                assert!(brand.x + brand.w <= strip.x + 0.01, "{name} {w}: the brand overlaps the tools {brand:?} {strip:?}");
                assert!(strip.x + strip.w <= w - 138.0 + 0.01, "{name} {w}: the tools overlap the window's buttons");
                assert!((strip.x + strip.w / 2.0 - w / 2.0).abs() <= 0.5, "{name} {w}: the tools are centred");
                let mut parts = ids.tabs.clone();
                parts.extend(ids.tools);
                if ids.more != Id::ROOT {
                    parts.push(ids.more);
                }
                let mut rects: Vec<Rect> = parts.iter().map(|&p| rect_of(&ui, p)).collect();
                rects.sort_by(|a, b| a.x.total_cmp(&b.x));
                assert!(rects.windows(2).all(|p| p[0].x + p[0].w <= p[1].x + 1.01), "{name} {w}: parts overlap");
                assert!(rects.iter().all(|r| r.x >= strip.x - 0.01 && r.x + r.w <= strip.x + strip.w + 0.01 && r.y >= bar.y && r.y + r.h <= bar.y + bar.h), "{name} {w}: a part is outside");
                // The tools say their names, always, and are as wide as the word and the icon need.
                let words = drawn(&list);
                for (i, tool) in ids.tools.iter().enumerate() {
                    assert!(words.iter().any(|t| t == text::TOOL_NAMES[i]), "{name} {w}: '{}' is not drawn", text::TOOL_NAMES[i]);
                    let r = rect_of(&ui, *tool);
                    assert!((r.w - tool_button_width(text::TOOL_NAMES[i])).abs() < 0.01, "{name} {w}: tool {i} is {} wide", r.w);
                }
                // With no account the brand is the cube and, when the form keeps it, the word: its width says which.
                if account.is_none() {
                    let i = Inputs { width: w, ends: overflow::ENDS_W, account, preview_open: true, pressed: 2, disabled: [false; 6], shapes: true };
                    assert!((rect_of(&ui, ids.brand).w - overflow::brand_need(&i, &ids.form)).abs() < 0.01, "{name} {w}: the brand is not what the form says");
                }
                println!("{name:9} {w:5} {account:?}: {:?}, strip {:.1} wide", ids.form, strip.w);
            }
        }
    }
}

/// The expected outcome at the window's minimum width: the account goes, the greyed views go into "More", and
/// Biomes, Relief and Temperature stay.
#[test]
fn at_the_minimum_width_the_greyed_views_are_in_more() {
    let Some(world) = common::world() else { return };
    assert!(world.coarse(), "the loaded world is a coarse one");
    let (list, ui, ids) = built(1000.0, state(Some("Louis"), 2), Some(&world));
    assert_eq!(ids.form, form(false, true, &[1, 4, 5]));
    assert_eq!(ids.tabs.len(), 3 + 2, "Biomes, Relief, Temperature, then 2D and 3D");
    assert_ne!(ids.more, Id::ROOT);
    let words = drawn(&list);
    for kept in ["Biomes", "Relief", "Temperature", "More", "Home", "Preview", "2D", "3D"] {
        assert!(words.iter().any(|t| t == kept), "{kept} is drawn: {words:?}");
    }
    for gone in ["Flat", "Structures", "Section", "Louis"] {
        assert!(!words.iter().any(|t| t == gone), "{gone} is not drawn");
    }
    // "More" is the last of the views, before the 2D and 3D tabs.
    let (more, relief, d2) = (rect_of(&ui, ids.more), rect_of(&ui, ids.tabs[1]), rect_of(&ui, ids.tabs[3]));
    assert!(relief.x < more.x && more.x + more.w <= d2.x);
}

/// Wide enough, nothing is given up.
#[test]
fn a_wide_window_keeps_everything() {
    let world = common::world();
    let (_, ui, ids) = built(1920.0, state(Some("Louis"), 2), None);
    assert_eq!(ids.form, form(true, true, &[]));
    assert_eq!(ids.more, Id::ROOT);
    assert_eq!(ids.tabs.len(), 8);
    let _ = (ui, world);
}

/// Home has the room: it keeps the account and the word at 1000, while the preview tool, on show, gives them up.
#[test]
fn home_keeps_what_the_preview_tool_gives_up() {
    let mut st = state(Some("Louis"), 2);
    st.mode = Mode::Home;
    let (_, _, ids) = built(1000.0, st, None);
    assert!(ids.form.account && ids.form.word, "{:?}", ids.form);
    let (_, _, ids) = built(1000.0, state(Some("Louis"), 2), None);
    assert!(!ids.form.account, "{:?}", ids.form);
}

/// The pressed view stays among the tabs, at every width and for every view.
#[test]
fn the_pressed_view_is_never_in_more() {
    for view in 0..6 {
        for w in (1000..=1500).step_by(25) {
            let (_, _, ids) = built(w as f32, state(Some("Louis"), view), None);
            assert!(!ids.form.more.contains(&view), "view {view} at {w}: {:?}", ids.form);
            let shapes = if view == 5 { 0 } else { 2 };
            assert_eq!(ids.tabs.len(), 6 - ids.form.more.len() + shapes, "view {view} at {w}");
        }
    }
}

/// An account that fits is whole; one that is a little too long is cut with an ellipsis; one that would be cut
/// to nothing is gone.
#[test]
fn the_account_is_whole_then_cut_then_gone() {
    let mut cut = None;
    let mut gone_at = None;
    for w in (1000..=2000).rev() {
        let (list, _, ids) = built(w as f32, state(Some("Alexandrina"), 2), None);
        let words = drawn(&list);
        let shown = words.iter().find(|t| t.starts_with("Al") && !t.contains(' '));
        match shown {
            Some(t) if t == "Alexandrina" => {}
            Some(t) => {
                assert!(t.ends_with('\u{2026}') && t.chars().count() >= 4, "{t}");
                cut.get_or_insert(w);
            }
            None => {
                assert!(!ids.form.account);
                gone_at.get_or_insert(w);
            }
        }
    }
    let (cut, gone_at) = (cut.expect("a width where it is cut"), gone_at.expect("a width where it goes"));
    assert!(cut > gone_at, "cut at {cut}, gone at {gone_at}");
}

struct Rig {
    app: PreviewApp,
    r: Renderer,
}

impl Rig {
    fn new(w: u32, st: PreviewState, world: Option<&World>) -> Rig {
        let mut r = Renderer::offscreen(w, 700, 1.0).expect("offscreen renderer");
        let mut app = PreviewApp::new(Theme::dark(), true, Vec2::new(w as f32, 700.0), st, world.cloned());
        app.start(&mut r);
        app.event(Event::Resized { width_px: w, height_px: 700, scale: 1.0 });
        app.frame(&mut r, 0.0);
        app.frame(&mut r, 0.0);
        Rig { app, r }
    }

    fn frame(&mut self) {
        self.app.frame(&mut self.r, 0.0);
        self.app.frame(&mut self.r, 0.0);
    }

    fn click(&mut self, p: Vec2) {
        self.app.event(Event::PointerMoved { pos: p });
        self.app.event(Event::PointerDown { pos: p, button: Button::Left, clicks: 1 });
        self.frame();
        self.app.event(Event::PointerUp { pos: p, button: Button::Left });
        self.frame();
    }

    fn more(&self) -> Rect {
        rect_of(self.app.ui(), self.app.ids().more)
    }

    fn row(&self, i: usize) -> Vec2 {
        let b = self.more();
        Vec2::new(b.x + 20.0, b.y + b.h + 1.0 + i as f32 * MENU_ROW + 10.0)
    }

    fn words(&mut self) -> Vec<String> {
        drawn(self.app.list())
    }
}

/// "More" lists the views it holds; choosing one presses it, and it takes its place in the strip.
#[test]
fn choosing_a_view_in_more_presses_it() {
    let mut rig = Rig::new(1000, state(None, 0), None);
    assert_eq!(rig.app.ids().form.more, vec![4, 5], "Structures and Section are in More at 1000");
    let at = rig.more();
    rig.click(Vec2::new(at.x + 10.0, at.y + 10.0));
    let words = rig.words();
    for row in ["Structures", "Section"] {
        assert!(words.iter().any(|t| t == row), "{row} is listed: {words:?}");
    }
    let at = rig.row(1);
    rig.click(at);
    assert_eq!(rig.app.state().view, 5, "Section is pressed");
    assert!(!rig.app.ids().form.more.contains(&5), "and it is in the strip");
    assert_eq!(rig.app.ids().tabs.len(), 6 - rig.app.ids().form.more.len(), "the section has no 2D and 3D, so the strip has room for more views");
}

/// A greyed view is listed, shown disabled, and a press on it presses nothing.
#[test]
fn a_greyed_view_in_more_cannot_be_chosen() {
    let Some(world) = common::world() else { return };
    let mut rig = Rig::new(1000, PreviewState { reduce_motion: true, ..PreviewState::for_world(Some(&world)) }, Some(&world));
    let before = rig.app.state().view;
    let at = rig.more();
    rig.click(Vec2::new(at.x + 10.0, at.y + 10.0));
    let words = rig.words();
    for row in ["Flat", "Structures", "Section"] {
        assert!(words.iter().any(|t| t == row), "{row} is listed");
    }
    let at = rig.row(0);
    rig.click(at);
    assert_eq!(rig.app.state().view, before, "Flat did nothing");
    assert!(rig.words().iter().any(|t| t == "Flat"), "the list stays open: Flat is in it and in no tab");
}
