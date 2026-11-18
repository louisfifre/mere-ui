//! The help of a row: the mark after the label, the panel over the row, its cross, and where the panel goes.

use mere_base::{Button, Cmd, Color, DrawList, Event, Rect, Vec2};
use mere_layout::{Align, Dim, Edges, Position, Style};
use mere_ui::tokens::{CONTROL, FRAME, INSET};
use mere_ui::{HELP_MARK_W, HelpResp, HelpText, Id, TextLook, Theme, Tone, Ui, help_anchor, help_label};

const SIZE: Vec2 = Vec2::new(400.0, 300.0);
const W: f32 = 240.0;
const MARK: &str = "?";
const HINT: &str = "What this setting shapes, said in a sentence long enough to wrap onto a second line of the panel.";

/// A column of rows `top` below the window's top, each with its label and mark, and the open one's help.
struct Rig {
    ui: Ui,
    open: Option<usize>,
    top: f32,
    facts: &'static str,
    rows: Vec<Id>,
    marks: Vec<Id>,
    help: Option<HelpResp>,
    list: DrawList,
}

impl Rig {
    fn new(top: f32) -> Rig {
        let mut r = Rig { ui: Ui::new(Theme::dark()), open: None, top, facts: "", rows: Vec::new(), marks: Vec::new(), help: None, list: DrawList::default() };
        r.frame();
        r.frame();
        r
    }

    fn frame(&mut self) {
        let Rig { ui, open, top, facts, rows, marks, help, list } = self;
        rows.clear();
        marks.clear();
        *help = None;
        ui.begin_frame(SIZE);
        ui.node("col", Style { column: true, w: Dim::Px(W), pad: Edges::new(*top, 0.0, 0.0, 20.0), ..Style::default() }, |ui| {
            for i in 0..3 {
                ui.node(&format!("place{i}"), help_anchor(), |ui| {
                    let row = Style { h: Dim::Px(CONTROL), align_items: Align::Center, ..Style::default() };
                    let id = ui.hover_area("row", row, |ui, over| {
                        let (label, at) = help_label("Label", W);
                        ui.node("name", Style { position: Position::Relative, ..Style::default() }, |ui| {
                            ui.node("label", label, |ui| ui.paragraph("Label", TextLook::body(Tone::Text)));
                            let r = ui.help_mark(MARK, at, over.is_some(), *open == Some(i));
                            marks.push(r.id);
                            if r.clicked {
                                *open = if *open == Some(i) { None } else { Some(i) };
                            }
                        });
                    });
                    rows.push(id);
                    if *open == Some(i) {
                        let r = ui.help_panel("help", id, HelpText { title: "Label", hint: HINT, facts });
                        if r.closed {
                            *open = None;
                        }
                        *help = Some(r);
                    }
                });
            }
        });
        *list = ui.end_frame().clone();
    }

    fn rect(&self, id: Id) -> Rect {
        self.ui.rect(id).expect("laid out")
    }

    fn centre(&self, id: Id) -> Vec2 {
        let r = self.rect(id);
        Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    fn moved(&mut self, pos: Vec2) {
        self.ui.event(&Event::PointerMoved { pos });
        self.frame();
    }

    fn click(&mut self, pos: Vec2) {
        self.moved(pos);
        self.ui.event(&Event::PointerDown { pos, button: Button::Left, clicks: 1 });
        self.frame();
        self.ui.event(&Event::PointerUp { pos, button: Button::Left });
        self.frame();
        self.frame();
    }

    /// The colours the marks are drawn in this frame.
    fn drawn(&self) -> Vec<Color> {
        self.list.cmds.iter().filter_map(|c| match c {
            Cmd::Text { text, color, .. } if text == MARK => Some(*color),
            _ => None,
        }).collect()
    }
}

#[test]
fn the_mark_shows_on_its_row_lights_under_the_pointer_and_stays_lit_while_open() {
    let t = Theme::dark();
    let mut r = Rig::new(20.0);
    assert!(r.drawn().is_empty(), "hidden at rest");
    let row = r.rect(r.rows[1]);
    r.moved(Vec2::new(row.x + row.w - 4.0, row.y + row.h / 2.0));
    assert_eq!(r.drawn(), vec![t.muted], "the row's own mark, muted");
    let at = r.centre(r.marks[1]);
    r.moved(at);
    assert_eq!(r.drawn(), vec![t.text], "in the text colour under the pointer");
    // The mark follows the label, a tight gap after its last word.
    let mark = r.rect(r.marks[1]);
    assert_eq!(mark.w, HELP_MARK_W);
    assert!(mark.x > row.x && mark.x < row.x + 80.0, "after the label, {mark:?}");
    r.click(at);
    assert_eq!(r.open, Some(1));
    r.moved(Vec2::new(390.0, 290.0));
    assert_eq!(r.drawn(), vec![t.text], "the open one stays lit");
    // The panel covers its own row: the mark under it takes no click, the cross is what closes.
    r.click(at);
    assert_eq!(r.open, Some(1));
}

#[test]
fn the_panel_starts_at_its_row_s_top_moves_nothing_and_its_cross_closes_it() {
    let mut r = Rig::new(20.0);
    let before: Vec<Rect> = r.rows.iter().map(|&id| r.rect(id)).collect();
    let at = r.centre(r.marks[0]);
    r.click(at);
    let h = r.help.expect("the help is built");
    let (row, panel, cross, title, hint) = (r.rect(r.rows[0]), r.rect(h.panel), r.rect(h.close), r.rect(h.title), r.rect(h.hint));
    assert_eq!((panel.x, panel.y, panel.w), (row.x, row.y, row.w), "over its row, from the row's top, as wide");
    assert!(panel.h > row.h, "and covers what follows");
    for (k, was) in before.iter().enumerate() {
        assert_eq!(r.rect(r.rows[k]), *was, "row {k} did not move");
    }
    // The cross on the title's first line, at the right, inside the frame and the inset; the hint under them.
    let pad = FRAME + INSET;
    assert_eq!((cross.y, title.y, title.x), (panel.y + pad, panel.y + pad, panel.x + pad));
    assert_eq!(cross.x + cross.w, panel.x + panel.w - pad);
    assert!(cross.y >= row.y && cross.y + cross.h <= row.y + row.h, "the cross is on the row's line");
    assert!(hint.y >= title.y + title.h && hint.h >= 2.0 * 18.0, "the hint wraps under the title");
    assert!(h.facts.is_none(), "no facts, no line for them");
    assert_eq!(panel.y + panel.h, hint.y + hint.h + pad);
    // What the panel covers takes no click.
    let under = r.centre(r.marks[1]);
    assert!(panel.contains(under));
    r.click(under);
    assert_eq!(r.open, Some(0));
    r.click(r.centre(h.close));
    assert_eq!(r.open, None);
    assert!(r.help.is_none());
}

#[test]
fn the_facts_end_the_panel() {
    let mut r = Rig::new(20.0);
    r.facts = "Default 1, from 0 to 4.";
    r.open = Some(0);
    r.frame();
    r.frame();
    let h = r.help.unwrap();
    let (panel, hint, facts) = (r.rect(h.panel), r.rect(h.hint), r.rect(h.facts.expect("facts")));
    assert!(facts.y >= hint.y + hint.h);
    assert_eq!(panel.y + panel.h, facts.y + facts.h + FRAME + INSET);
}

#[test]
fn a_panel_that_would_pass_the_window_s_bottom_is_raised_to_end_on_it() {
    // The last row's top is 20 px above the window's bottom: its help has no room under the row's top.
    let mut r = Rig::new(SIZE.y - 2.0 * CONTROL - 20.0);
    let at = r.centre(r.marks[2]);
    r.click(at);
    let (row, panel) = (r.rect(r.rows[2]), r.rect(r.help.unwrap().panel));
    assert!(panel.y + panel.h <= SIZE.y && panel.y + panel.h > SIZE.y - 1.0, "it ends on the window's bottom, {panel:?}");
    assert!(panel.y < row.y, "raised above its row's top");
}
