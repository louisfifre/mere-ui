//! The page's pressed-state buttons: the toolbar's strip, the choices drawn in reverse, and `#details`.
//! The app decides which tab is pressed (the page's strip lights one tab in each of two groups); a click is
//! reported in `Resp.clicked` and the app changes its selection, which the next frame shows.

use mere_base::{Color, Cursor};
use mere_layout::{Align, Dim, Display, Edges, Margin, NodeId, Position, Side, Style, Track, Tracks};
use mere_base::Vec2;

use crate::text::{TextLook, Tone};
use crate::theme::mix;
use crate::tokens::*;
use crate::ui::{Paint, Resp, Ui};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TabLook {
    /// `.strip button` (lines 95-99): the toolbar's boxes sharing their hairlines.
    Strip,
    /// `.choice button` with aria-pressed (lines 147-150, 202): words, the current one in reverse.
    Choice,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TabOpts {
    pub pressed: bool,
    pub disabled: bool,
}

/// `.strip` (line 95).
pub fn strip_style() -> Style {
    Style { align_items: Align::Stretch, pad: Edges::new(0.0, 0.0, 0.0, 1.0), ..Style::default() }
}

/// `.choice` (line 147).
pub fn choice_style() -> Style {
    Style { gap: Vec2::new(GAP_TIGHT, 0.0), align_items: Align::Baseline, ..Style::default() }
}

/// `#details` (line 195) plus the `.choice` class it also carries (line 394, the step 8 observation).
pub fn details_style() -> Style {
    Style {
        display: Display::Grid,
        cols: Tracks::new(&[Track::MinMaxFr0(1.0); 3]),
        pad: Edges::new(0.0, 0.0, 0.0, 1.0),
        gap: Vec2::new(GAP_TIGHT, GAP_TIGHT),
        // Every option as tall as the tallest, so one with a second line does not stand out of the row.
        align_items: Align::Stretch,
        ..Style::default()
    }
}

fn overlap() -> Margin {
    Margin { left: Side::Px(-1.0), ..Margin::default() }
}

impl Ui {
    /// A tab of the strip or a choice. Two adjacent tabs both count as hovered on their shared 1 px seam
    /// (the browser gives it to the later one): accepted.
    pub fn tab(&mut self, key: &str, text: &str, look: TabLook, opts: TabOpts) -> Resp {
        let (resp, _, _) = self.tab_open(key, text, look, opts);
        self.close();
        resp
    }

    /// A tab left open: the caller adds what else the box holds and closes it. Answers the box's node and the
    /// colour its word took.
    pub(crate) fn tab_open(&mut self, key: &str, text: &str, look: TabLook, opts: TabOpts) -> (Resp, NodeId, Color) {
        let id = self.key(key);
        let resp = self.interact(id, opts.disabled, Cursor::Hand);
        let t = *self.theme();
        let lit = resp.hovered || resp.held;
        // Disabled fades the whole box: every colour, fill included.
        let fade = |c: Color| if opts.disabled { t.faded(c, FADE_DISABLED) } else { c };
        // DELIBERATE DIFFERENCE: in the page's cascade `.strip button:hover:not(:disabled)` (0,3,1) beats
        // `[aria-pressed=true]` (0,2,1), so hovering the pressed strip tab paints its word in --text on a
        // --text ground and the word vanishes. We keep the pressed word on hover.
        let word = if opts.pressed {
            t.on_fill
        } else if lit {
            t.text
        } else {
            t.muted
        };
        let fill = opts.pressed.then_some(fade(t.fill));
        let (style, frame) = match look {
            TabLook::Strip => (
                Style {
                    h: Dim::Px(CONTROL),
                    pad: Edges::vh(FRAME, FRAME + INSET_WIDE),
                    margin: overlap(),
                    align_items: Align::Center,
                    position: if opts.pressed { Position::Relative } else { Position::Static },
                    ..Style::default()
                },
                Some(fade(if opts.pressed { t.fill } else { t.edge })),
            ),
            TabLook::Choice => (Style { pad: Edges::vh(INSET_HAIR, INSET), ..Style::default() }, None),
        };
        let node = self.open(key, id, style, Paint::Box { fill, frame });
        self.add_text(text, TextLook::body(Tone::Text), fade(word), false, Style::default());
        (resp, node, fade(word))
    }

    /// `#details button` (lines 195-200): a word and its wait on a second, smaller line. No hover rule
    /// exists for it.
    pub fn detail_tab(&mut self, key: &str, text: &str, sub: &str, opts: TabOpts) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, opts.disabled, Cursor::Hand);
        let t = *self.theme();
        let fade = |c: Color| if opts.disabled { t.faded(c, FADE_DISABLED) } else { c };
        let style = Style {
            column: true,
            pad: Edges::vh(FRAME + INSET_TIGHT, FRAME + INSET),
            margin: overlap(),
            align_items: Align::Start,
            position: if opts.pressed { Position::Relative } else { Position::Static },
            ..Style::default()
        };
        let fill = opts.pressed.then_some(fade(t.fill));
        let frame = fade(if opts.pressed { t.fill } else { t.edge });
        self.open(key, id, style, Paint::Box { fill, frame: Some(frame) });
        let word = if opts.pressed { t.on_fill } else { t.text };
        self.add_text(text, TextLook::body(Tone::Text), fade(word), false, Style::default());
        // `small` inherits the unitless 1.5 line height; pressed, colour inherit at opacity .7 over the fill.
        let small = TextLook { size: 11.0, line: 16.5, ..TextLook::body(Tone::Text) };
        let sub_color = if opts.pressed { mix(t.on_fill, 0.7, t.fill) } else { t.muted };
        // An empty small block is 0 high, so the button has one line then: no leaf at all.
        if !sub.is_empty() {
            self.add_text(sub, small, fade(sub_color), false, Style::default());
        }
        self.close();
        resp
    }

    /// The `gap-wide` between two groups of the strip. Not keyed.
    pub fn strip_gap(&mut self) {
        self.add("sep", Style { w: Dim::Px(GAP_WIDE), shrink: 0.0, ..Style::default() }, Paint::None);
    }
}
