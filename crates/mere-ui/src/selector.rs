//! The selector: the small mark that shows whether a choice is taken. A single selector (round) is one choice among
//! several, a multiple selector (square) one of several choices that can be taken together.
//!
//! The mark alone (`selector_mark`) is the atom: it paints a state and takes no press. Beside a label it is the
//! labelled selector (`selector`), which takes the press; in a menu's selection row (`menu_select`) it faces a label
//! and an optional detail line. Every piece that shows a selector paints it with `selector_mark`.
//!
//! A selector asks and its caller decides, whatever its kind: `Resp.clicked` means the person asked for the choice
//! to change, and the caller sets what is selected (this one for a single selector, a flip for a multiple one).

use mere_base::{Cmd, Color, Cursor, Rect, Vec2};
use mere_layout::{Align, Dim, Style};

use crate::text::{TextLook, Tone};
use crate::tokens::{CONTROL_COMPACT, FRAME, GAP, SWATCH};
use crate::ui::{Paint, Resp, Ui};

/// The side of a selector: a swatch's, so that a selection row's columns line up with a list row's.
pub const MARK_SIZE: f32 = SWATCH;
/// The check: 1 px squares inside the frame, the boards' 10 px glyph moved in by the frame.
const CHECK: [(f32, f32); 7] = [(9.0, 3.0), (8.0, 4.0), (7.0, 5.0), (6.0, 6.0), (5.0, 7.0), (4.0, 6.0), (3.0, 5.0)];
/// The selected single selector's dot: a disc this wide, centred in the outline, painted by the same rule as it.
const DOT: f32 = 6.0;

/// The kind of choice a selector stands for.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Selector {
    /// One choice among several: round, a dot when selected.
    Single,
    /// Several choices that can be taken together: square, a check when selected.
    #[default]
    Multiple,
}

/// How the selector alone looks beyond its kind and value: what the piece holding it knows.
#[derive(Copy, Clone, Debug, Default)]
pub struct MarkOpts {
    /// Faded to `FADE_DISABLED`.
    pub disabled: bool,
    /// Under the pointer or held: the line in the text colour.
    pub lit: bool,
    /// On the selection fill of a focused row: the line muted, the edge colour not showing on it.
    pub on_selection: bool,
}

/// A labelled selector.
#[derive(Copy, Clone, Debug, Default)]
pub struct SelectorOpts {
    /// Faded to `FADE_DISABLED`, and no press.
    pub disabled: bool,
}

impl Selector {
    /// The corner radius of the outline: exactly half the side for the single selector, which makes it a circle, and
    /// none for the multiple selector, a square. The kind changes only this and the mark inside: the outline, its
    /// width, its colours and its states are one and the same.
    pub fn radius(self) -> f32 {
        match self {
            Selector::Single => MARK_SIZE / 2.0,
            Selector::Multiple => 0.0,
        }
    }
}

/// Whether the pixel whose centre is `(x, y)` lies in the square at `at` of `side` with corners rounded to `r`.
fn inside(x: f32, y: f32, at: f32, side: f32, r: f32) -> bool {
    let (lo, hi) = (at + r, at + side - r);
    let (dx, dy) = (x - x.clamp(lo, hi), y - y.clamp(lo, hi));
    x >= at && x <= at + side && y >= at && y <= at + side && dx * dx + dy * dy <= r * r
}

/// A shape of whole pixels inside `MARK_SIZE`: each pixel whose centre `keep` accepts, painted in runs along its row.
/// Whole pixels and no antialiasing, so that the round outline is as sharp as the square one beside it.
fn pixels(color: Color, keep: impl Fn(f32, f32) -> bool) -> Vec<Cmd> {
    let n = MARK_SIZE as usize;
    let mut cmds = Vec::new();
    for row in 0..n {
        let y = row as f32 + 0.5;
        let mut run: Option<usize> = None;
        for col in 0..=n {
            let on = col < n && keep(col as f32 + 0.5, y);
            match (on, run) {
                (true, None) => run = Some(col),
                (false, Some(from)) => {
                    cmds.push(Cmd::Fill { rect: Rect::new(from as f32, row as f32, (col - from) as f32, 1.0), color });
                    run = None;
                }
                _ => {}
            }
        }
    }
    cmds
}

/// The outline of a selector: the band `FRAME` wide inside a square of `MARK_SIZE` whose corners are rounded to `r`.
fn outline(r: f32, color: Color) -> Vec<Cmd> {
    let inner = (r - FRAME).max(0.0);
    pixels(color, |x, y| inside(x, y, 0.0, MARK_SIZE, r) && !inside(x, y, FRAME, MARK_SIZE - 2.0 * FRAME, inner))
}

impl Ui {
    /// The selector alone, `MARK_SIZE` square, on whole pixels: a leaf of the tree that takes no press and does not
    /// shrink. Its line is in the text colour when selected or lit, else the edge (muted on the selection fill); the
    /// check or the dot shows when selected, in the text colour.
    pub fn selector_mark(&mut self, kind: Selector, selected: bool, opts: MarkOpts) {
        let t = *self.theme();
        let line = if selected || opts.lit {
            t.text
        } else if opts.on_selection {
            t.muted
        } else {
            t.edge
        };
        let fade = |c| t.disabled_if(c, opts.disabled);
        let (line, inner) = (fade(line), selected.then(|| fade(t.text)));
        let mut cmds = outline(kind.radius(), line);
        if let Some(c) = inner {
            match kind {
                Selector::Multiple => cmds.extend(CHECK.into_iter().map(|(x, y)| Cmd::Fill { rect: Rect::new(x, y, 1.0, 1.0), color: c })),
                Selector::Single => {
                    let at = (MARK_SIZE - DOT) / 2.0;
                    cmds.extend(pixels(c, |x, y| inside(x, y, at, DOT, DOT / 2.0)));
                }
            }
        }
        let style = Style { w: Dim::Px(MARK_SIZE), h: Dim::Px(MARK_SIZE), shrink: 0.0, ..Style::default() };
        self.add("mark", style, Paint::Cmds(cmds));
    }

    /// A selector with its label beside it, `control-compact` tall. `Resp.clicked` means the person asked for the
    /// choice to change (a press on the mark or the label); the caller applies it (selects this one and unselects the
    /// others of its group for a single selector, flips it for a multiple one), and the next frame, which the `Ui`
    /// asks for (`want_redraw`), shows it. Like the toggle it takes no keyboard focus; a menu's selection row does.
    pub fn selector(&mut self, key: &str, label: &str, kind: Selector, selected: bool, opts: SelectorOpts) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, opts.disabled, Cursor::Hand);
        if resp.clicked {
            self.redraw = true;
        }
        let t = *self.theme();
        let style = Style { h: Dim::Px(CONTROL_COMPACT), gap: Vec2::new(GAP, 0.0), align_items: Align::Center, justify_self: Some(Align::Start), ..Style::default() };
        self.open(key, id, style, Paint::None);
        self.selector_mark(kind, selected, MarkOpts { disabled: opts.disabled, lit: resp.hovered || resp.held, on_selection: false });
        if !label.is_empty() {
            let word = t.disabled_if(t.text, opts.disabled);
            self.add_text(label, TextLook::body(Tone::Text), word, false, Style::default());
        }
        self.close();
        resp
    }
}
