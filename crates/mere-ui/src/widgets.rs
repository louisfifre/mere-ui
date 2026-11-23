//! Label, button and toggle in the page's looks.

use mere_base::{Cmd, Color, Cursor, Rect, Vec2, Weight};
use mere_layout::{Align, Dim, Edges, Justify, Style};

use crate::text::{TextLook, Tone};
use crate::tokens::*;
use crate::ui::{Paint, Resp, Ui};

/// The page buttons. The strip, #details and pressed choices are in tab.rs; the stop square is `Ui::stop_button`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ButtonLook {
    /// `#generate`: filled with the text colour, 600 weight.
    Primary,
    /// A `.choice button`: a muted word.
    Plain,
    /// A `.choice button.action`: a muted underlined word.
    Link,
    /// `#generate.running` (lines 215-218): the generate button while a run is under way. Framed in the text
    /// colour, no fill but a `--line` one from the left that follows the wait: `share` of the padding box's width.
    Running { share: f32 },
    /// `#pick button` (lines 342-343): 11 px, the text colour, no padding; a line through the word on hover.
    Pick,
    /// `#pick button.none` (line 344): the same, underlined always. The later rule wins at equal specificity,
    /// so the hover strike does not apply to it.
    PickNone,
    /// The launcher's `.folder button` (lines 211-212): the text colour, underlined, 2 px above and below,
    /// --muted under the pointer.
    Underlined,
}

/// What the toggle's box differs by between applications. The switch itself has one size (`SWITCH_W` by
/// `SWITCH_H`); these measures are the room around it and its word, which is also what a click reaches.
#[derive(Copy, Clone, Debug)]
pub struct ToggleOpts {
    /// Least width of the whole box, so that several toggles or a toggle and a field can line up.
    pub min_w: Dim,
    /// Padding of the box, around the switch and its word.
    pub pad: Edges,
    /// Disabled: faded to `FADE_DISABLED`, and no click.
    pub disabled: bool,
}

impl Default for ToggleOpts {
    fn default() -> Self {
        // 3 above and below the 18 px line of the word make the 24 px of a compact control.
        ToggleOpts { min_w: Dim::Auto, pad: Edges::vh(3.0, 0.0), disabled: false }
    }
}

/// The switch: a square-cornered frame, 28 by 16, whose square knob (8) sits 3 inside the frame's line. 28 and the
/// 8 of the gap leave the word "off" in the 8ch (57.6 px) of a setting's field column.
pub const SWITCH_W: f32 = 28.0;
pub const SWITCH_H: f32 = 16.0;
const KNOB: f32 = 8.0;
#[derive(Copy, Clone, Debug, Default)]
pub struct ButtonOpts {
    pub disabled: bool,
    /// flex-grow of the button's box.
    pub grow: f32,
}

impl Ui {
    pub fn label(&mut self, s: &str, tone: Tone) {
        self.text(s, TextLook::body(tone));
    }

    /// A text leaf, not keyed: text takes no input.
    pub fn text(&mut self, s: &str, look: TextLook) {
        let color = self.tone_color(look.tone);
        self.add_text(s, look, color, false, Style::default());
    }

    /// A text that wraps at its box's width, in the tone's colour: the page's `white-space: normal` text.
    pub fn paragraph(&mut self, s: &str, look: TextLook) {
        let color = self.tone_color(look.tone);
        self.add_paragraph(s, look, color, Style::default());
    }

    /// A muted label and a text that flows on after it and wraps back to the left edge (`.saved .path`).
    pub fn flow(&mut self, lead: &str, lead_tone: Tone, gap: f32, text: &str, look: TextLook) {
        let (lead_color, color) = (self.tone_color(lead_tone), self.tone_color(look.tone));
        self.add_flow(lead, lead_color, gap, text, look, color, Style::default());
    }

    /// An unkeyed box painted in one colour: a one-side border, a rule, a slot.
    pub fn fill(&mut self, style: Style, color: Color) {
        self.add("fill", style, Paint::Box { fill: Some(color), frame: None });
    }

    /// A text leaf that ends in an ellipsis when its box is narrower than the text. `min-width: 0` lets it
    /// shrink in a flex row; a grid fr column already stretches it.
    pub fn ellipsis(&mut self, s: &str, look: TextLook) {
        let color = self.tone_color(look.tone);
        self.add_ellipsis(s, look, color, false);
    }

    /// `ellipsis` cut at the start (`ellipsize_start`): a folder too long for its box keeps its end, which is the part
    /// that tells two folders apart.
    pub fn ellipsis_start(&mut self, s: &str, look: TextLook) {
        let color = self.tone_color(look.tone);
        self.add_ellipsis_start(s, look, color);
    }

    pub fn button(&mut self, key: &str, text: &str, look: ButtonLook) -> Resp {
        self.button_with(key, text, look, ButtonOpts::default())
    }

    pub fn button_with(&mut self, key: &str, text: &str, look: ButtonLook, opts: ButtonOpts) -> Resp {
        let id = self.key(key);
        // `#generate.running` shows the default arrow (line 216).
        let cursor = if matches!(look, ButtonLook::Running { .. }) { Cursor::Arrow } else { Cursor::Hand };
        let resp = self.interact(id, opts.disabled, cursor);
        let t = *self.theme();
        // The page has no :active style, so a held button looks exactly like a hovered one.
        let lit = resp.hovered || resp.held;
        let body = TextLook { size: t.body, line: t.body * 1.5, weight: Weight::Regular, tone: Tone::Text, tracking: 0.0 };
        let (style, paint, word_look, word, underline, strike) = match look {
            ButtonLook::Primary => {
                let style = Style {
                    h: Dim::Px(CONTROL_LARGE),
                    pad: Edges::vh(FRAME, FRAME + INSET_WIDE),
                    align_items: Align::Center,
                    justify: Justify::Start,
                    grow: opts.grow,
                    ..Style::default()
                };
                let (paint, word) = if opts.disabled {
                    (Paint::Box { fill: None, frame: Some(t.edge) }, t.muted)
                } else if lit {
                    let c = t.faded(t.fill, FADE_LIT);
                    (Paint::Box { fill: Some(c), frame: Some(c) }, t.on_fill)
                } else {
                    (Paint::Box { fill: Some(t.fill), frame: Some(t.fill) }, t.on_fill)
                };
                (style, paint, TextLook { weight: Weight::SemiBold, ..body }, word, false, false)
            }
            ButtonLook::Running { share } => {
                let style = Style {
                    h: Dim::Px(CONTROL_LARGE),
                    pad: Edges::vh(FRAME, FRAME + INSET_WIDE),
                    align_items: Align::Center,
                    justify: Justify::Start,
                    grow: opts.grow,
                    ..Style::default()
                };
                // `:disabled` sets the border colour and `.running` does not, so a disabled one keeps --edge; but
                // `.running` sets the colour later in the file than `:disabled`, so its word stays --text. No
                // hover rule applies (`:not(.running)`).
                let frame = if opts.disabled { t.edge } else { t.text };
                (style, Paint::Running { frame, bar: t.line, share }, TextLook { weight: Weight::SemiBold, ..body }, t.text, false, false)
            }
            ButtonLook::Plain | ButtonLook::Link => {
                let link = look == ButtonLook::Link;
                let style = Style {
                    pad: if link { Edges::vh(INSET_HAIR, 0.0) } else { Edges::vh(INSET_HAIR, INSET) },
                    grow: opts.grow,
                    ..Style::default()
                };
                let word = if opts.disabled {
                    t.disabled_if(t.muted, true)
                } else if lit {
                    t.text
                } else {
                    t.muted
                };
                (style, Paint::None, body, word, link, false)
            }
            ButtonLook::Underlined => {
                let style = Style { pad: Edges::vh(INSET_HAIR, 0.0), grow: opts.grow, ..Style::default() };
                let word = if opts.disabled {
                    t.disabled_if(t.text, true)
                } else if lit {
                    t.muted
                } else {
                    t.text
                };
                (style, Paint::None, body, word, true, false)
            }
            ButtonLook::Pick | ButtonLook::PickNone => {
                let none = look == ButtonLook::PickNone;
                let style = Style { grow: opts.grow, ..Style::default() };
                let small = TextLook { size: 11.0, line: 16.5, ..body };
                (style, Paint::None, small, t.text, none, !none && lit)
            }
        };
        self.open(key, id, style, paint);
        if strike {
            self.add_text_struck(text, word_look, word, true, Style::default());
        } else {
            self.add_text(text, word_look, word, underline, Style::default());
        }
        self.close();
        resp
    }

    /// A `control-large` square that does not shrink, framed, with an 8 px filled square centred in it.
    pub fn stop_button(&mut self, key: &str, disabled: bool) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, disabled, Cursor::Hand);
        let t = *self.theme();
        let lit = !disabled && (resp.hovered || resp.held);
        let (colour, frame) = if disabled {
            (t.disabled_if(t.muted, true), t.disabled_if(t.edge, true))
        } else if lit {
            (t.text, t.text)
        } else {
            (t.muted, t.edge)
        };
        const SQUARE: f32 = 8.0;
        let at = (CONTROL_LARGE - SQUARE) / 2.0;
        let style = Style { w: Dim::Px(CONTROL_LARGE), h: Dim::Px(CONTROL_LARGE), shrink: 0.0, ..Style::default() };
        let cmds = vec![
            Cmd::Frame { rect: Rect::new(0.0, 0.0, CONTROL_LARGE, CONTROL_LARGE), width_px: FRAME, color: frame },
            Cmd::Fill { rect: Rect::new(at, at, SQUARE, SQUARE), color: colour },
        ];
        self.open(key, id, style, Paint::Cmds(cmds));
        self.close();
        resp
    }

    /// A switch with its word beside it: off is an outline with an outlined square knob on the left, on is the
    /// frame filled with `--fill` and the knob on the right in `--on-fill`. The word is `on_word` or `off_word`
    /// (the app's catalog), written in capitals as a fold's title is. A click anywhere on the switch or the word
    /// flips `on` before painting, so the frame already shows the new state. `Resp.clicked` means the value
    /// changed. Like the buttons it takes no keyboard focus: only text fields do in this kit.
    pub fn toggle(&mut self, key: &str, on: &mut bool, on_word: &str, off_word: &str) -> Resp {
        self.toggle_with(key, on, on_word, off_word, ToggleOpts::default())
    }

    /// `toggle` with the box's own measures and a disabled state.
    pub fn toggle_with(&mut self, key: &str, on: &mut bool, on_word: &str, off_word: &str, opts: ToggleOpts) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, opts.disabled, Cursor::Hand);
        if resp.clicked {
            *on = !*on;
        }
        let t = *self.theme();
        let lit = resp.hovered || resp.held;
        let on = *on;
        // The lines and the off word follow the pointer like a button's frame; the on fill like the primary's.
        let line = if lit { t.text } else { t.edge };
        let (mut fill, mut frame, mut knob, mut word) = if on {
            let f = if lit { t.faded(t.fill, FADE_LIT) } else { t.fill };
            (Some(f), f, t.on_fill, t.text)
        } else {
            // The off knob is muted at rest, not --edge, which is too faint to read as a part of the control.
            let ink = if lit { t.text } else { t.muted };
            (None, line, ink, ink)
        };
        // The on knob stays `on_fill`: it is read against the faded fill, not the ground.
        let off = |c| t.disabled_if(c, opts.disabled);
        fill = fill.map(off);
        frame = off(frame);
        if !on {
            knob = off(knob);
        }
        word = off(word);
        let style = Style {
            pad: opts.pad,
            min_w: opts.min_w,
            gap: Vec2::new(SPACE_8, 0.0),
            align_items: Align::Center,
            // In a grid cell the box keeps its own width instead of stretching, so a click beside it misses.
            justify_self: Some(Align::Start),
            ..Style::default()
        };
        self.open(key, id, style, Paint::None);
        let mut cmds = Vec::new();
        if let Some(c) = fill {
            cmds.push(Cmd::Fill { rect: Rect::new(0.0, 0.0, SWITCH_W, SWITCH_H), color: c });
        } else {
            cmds.push(Cmd::Frame { rect: Rect::new(0.0, 0.0, SWITCH_W, SWITCH_H), width_px: FRAME, color: frame });
        }
        let at = if on { SWITCH_W - FRAME - 3.0 - KNOB } else { FRAME + 3.0 };
        let knob_rect = Rect::new(at, (SWITCH_H - KNOB) / 2.0, KNOB, KNOB);
        cmds.push(if on { Cmd::Fill { rect: knob_rect, color: knob } } else { Cmd::Frame { rect: knob_rect, width_px: FRAME, color: knob } });
        let sw = Style { w: Dim::Px(SWITCH_W), h: Dim::Px(SWITCH_H), shrink: 0.0, ..Style::default() };
        self.add("switch", sw, Paint::Cmds(cmds));
        let words = if on { on_word } else { off_word }.to_uppercase();
        self.add_text(&words, TextLook::body(Tone::Text), word, false, Style::default());
        self.close();
        resp
    }
}