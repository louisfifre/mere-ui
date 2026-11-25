//! The list line: a line of a list of things that each have a version, an update and a place to act (a profile's
//! mods, the results of a search). Its trailing parts stand in slots of a fixed width, so that lines in different
//! states (up to date, to update, added, with no action) keep their words in the same columns; an empty slot keeps its
//! width. From the left: the name with its version right beside it, muted, then the slots: the new version, an
//! action, an icon action. A line is parted from the next by a separator, as the rows of a group are.
//!
//! The name is cut with an ellipsis when the room runs out and the version stays whole. A line with a `detail` has a
//! second line under the name (a description) and the height of two; a `note` is a last line over the whole width.

use mere_base::{Color, Cursor, Vec2};
use mere_layout::{Align, Dim, Display, Edges, Justify, Style, Track, Tracks};

use crate::chrome::{IconLook, IconOpts};
use crate::icon::Icon;
use crate::id::Id;
use crate::rule::Rule;
use crate::text::{TextLook, Tone};
use crate::tokens::{GAP, ICON, ICON_SQUARE, INSET_TIGHT, SPACE_48};
use crate::ui::{Paint, Resp, Ui};
use crate::widgets::{ActionLook, ActionOpts};

/// The new version's slot: right-aligned, wide enough for an arrow and a version of eight characters in the small look.
pub const LINE_NEW_W: f32 = 72.0;
/// The action's slot: right-aligned, wide enough for "Updating\u{2026}" or a check and "Added".
pub const LINE_ACTION_W: f32 = 64.0;
/// The icon action's slot, as wide as a square: the bare action is pulled to its right edge.
pub const LINE_END_W: f32 = ICON_SQUARE;

/// What a line's action slot holds.
#[derive(Copy, Clone, Debug, Default)]
pub enum LineAction<'a> {
    /// Nothing; the slot keeps its width.
    #[default]
    None,
    /// An underlined word that runs something ("Update", "Add").
    Word(&'a str),
    /// What is already done, no action: a muted icon and a muted word ("Added").
    Done(&'a Icon, &'a str),
}

/// The icon action at the end of a line (a trash, an external link). The app gives it its tooltip.
#[derive(Copy, Clone, Debug)]
pub struct LineEnd<'a> {
    pub icon: &'a Icon,
    pub disabled: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ListLine<'a> {
    pub name: &'a str,
    /// Beside the name, muted and small; empty for none.
    pub version: &'a str,
    /// A second line under the name, muted, cut at its end; empty for none.
    pub detail: &'a str,
    /// A last line over the line's whole width, muted; empty for none.
    pub note: &'a str,
    /// The name is a link: underlined under the pointer, its press reported in `ListLineResp::name`.
    pub link: bool,
    /// The new version, as the app writes it (an arrow and the version); empty for none.
    pub new_version: &'a str,
    pub action: LineAction<'a>,
    /// The action is faded and takes no press (an update under way).
    pub action_disabled: bool,
    pub end: Option<LineEnd<'a>>,
}

/// What a line answers: its parts that can be pressed, each a `Resp` that never fires when the part is absent.
#[derive(Copy, Clone, Debug)]
pub struct ListLineResp {
    pub line: Id,
    pub name: Resp,
    pub action: Resp,
    pub end: Resp,
}

/// The answer of a part that is not there.
fn absent(id: Id) -> Resp {
    Resp { id, rect: None, hovered: false, held: false, clicked: false, pressed: false }
}

/// The least height of a line with a second line, its padding counted.
const TALL: f32 = SPACE_48;

impl Ui {
    /// One line of a list, keyed `key`.
    pub fn list_line(&mut self, key: &str, line: ListLine) -> ListLineResp {
        let t = *self.theme();
        let (body, small) = (TextLook::body(Tone::Text), TextLook::small(Tone::Muted));
        let two = !line.detail.is_empty();
        let cols = [Track::MinMaxFr0(1.0), Track::Px(LINE_NEW_W), Track::Px(LINE_ACTION_W), Track::Px(LINE_END_W)];
        let grid = Style {
            display: Display::Grid,
            cols: Tracks::new(&cols),
            gap: Vec2::new(GAP, 0.0),
            align_items: Align::Center,
            min_h: Dim::Px(if two { TALL } else { ICON_SQUARE }),
            pad: Edges::vh(if two { INSET_TIGHT } else { 0.0 }, 0.0),
            ..Style::default()
        };
        let mut resp = (absent(Id::ROOT), absent(Id::ROOT), absent(Id::ROOT));
        let id = self.node(key, Style { column: true, ..Style::default() }, |ui| {
            ui.node("cells", grid, |ui| {
                // The name's cell: the name and its version on one baseline, the detail under them.
                ui.node("who", Style { column: true, min_w: Dim::Px(0.0), ..Style::default() }, |ui| {
                    ui.node("name", Style { align_items: Align::Baseline, gap: Vec2::new(GAP, 0.0), min_w: Dim::Px(0.0), ..Style::default() }, |ui| {
                        let id = ui.key("link");
                        let r = if line.link { ui.interact(id, false, Cursor::Hand) } else { absent(id) };
                        ui.open("link", id, Style { min_w: Dim::Px(0.0), ..Style::default() }, Paint::None);
                        ui.add_ellipsis(line.name, body, t.text, r.hovered || r.held);
                        ui.close();
                        resp.0 = r;
                        if !line.version.is_empty() {
                            ui.add_text(line.version, small, t.muted, false, Style { shrink: 0.0, ..Style::default() });
                        }
                    });
                    if two {
                        ui.add_ellipsis(line.detail, small, t.muted, false);
                    }
                });
                ui.node("new", slot(LINE_NEW_W), |ui| {
                    if !line.new_version.is_empty() {
                        ui.add_text(line.new_version, TextLook::small(Tone::Text), t.text, false, Style::default());
                    }
                });
                ui.node("action", slot(LINE_ACTION_W), |ui| match line.action {
                    LineAction::None => {}
                    LineAction::Word(word) => resp.1 = ui.action_with("do", word, ActionLook::Underlined, ActionOpts { disabled: line.action_disabled, ..ActionOpts::default() }),
                    LineAction::Done(icon, word) => done(ui, icon, word, t.muted),
                });
                ui.node("end", slot(LINE_END_W), |ui| {
                    if let Some(end) = line.end {
                        resp.2 = ui.icon_action("end", end.icon, IconLook::Bare, IconOpts { disabled: end.disabled, ..IconOpts::default() });
                    }
                });
            });
            if !line.note.is_empty() {
                ui.node("note", Style { pad: Edges::new(0.0, 0.0, INSET_TIGHT, 0.0), ..Style::default() }, |ui| ui.paragraph(line.note, TextLook::small(Tone::Muted)));
            }
            ui.rule(Rule::Separator, Edges::default());
        });
        ListLineResp { line: id, name: resp.0, action: resp.1, end: resp.2 }
    }
}

/// A slot: `w` wide whatever it holds, its content at its right end.
fn slot(w: f32) -> Style {
    Style { w: Dim::Px(w), shrink: 0.0, justify: Justify::End, align_items: Align::Center, ..Style::default() }
}

/// An icon and a word, muted, `gap-tight` apart.
fn done(ui: &mut Ui, icon: &Icon, word: &str, color: Color) {
    ui.node("done", Style { align_items: Align::Center, gap: Vec2::new(INSET_TIGHT, 0.0), ..Style::default() }, |ui| {
        ui.icon("mark", icon, ICON, color, icon.axis_aligned());
        ui.add_text(word, TextLook::body(Tone::Muted), color, false, Style::default());
    });
}
