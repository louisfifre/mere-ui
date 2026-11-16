//! Label, button and toggle in the page's looks.

use nui_base::{Cmd, Color, Cursor, Rect, Weight};
use nui_layout::{Align, Dim, Edges, Justify, Margin, Side, Style};

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

/// The launcher's `.toggle button` (lines 204-205) and what it differs by from the preview's.
#[derive(Copy, Clone, Debug)]
pub struct ToggleOpts {
    /// `min-width: 8ch`.
    pub min_w: Dim,
    /// Padding with the border.
    pub pad: Edges,
    /// Disabled: faded to `FADE_DISABLED`, and no click.
    pub disabled: bool,
}

impl Default for ToggleOpts {
    fn default() -> Self {
        ToggleOpts { min_w: Dim::Auto, pad: Edges::vh(FRAME + INSET_HAIR, FRAME + INSET), disabled: false }
    }
}

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

    /// `select.field` (lines 123 and 127), display only: it shows its value and an arrow and reports clicks.
    /// Its menu is `select_menu`. The compact select: `control-compact` tall, the value `inset` from the frame.
    pub fn select(&mut self, key: &str, value: &str) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, false, Cursor::Hand);
        let t = *self.theme();
        let pad = Edges::new(FRAME + INSET_HAIR, FRAME + INSET_TIGHT, FRAME + INSET_HAIR, FRAME + INSET);
        let style = Style { h: Dim::Px(CONTROL_COMPACT), pad, align_items: Align::Center, ..Style::default() };
        self.open(key, id, style, Paint::Box { fill: None, frame: Some(t.edge) });
        self.add_text(value, TextLook::body(Tone::Text), t.text, false, Style::default());
        let chevron = Style {
            w: Dim::Px(8.0),
            h: Dim::Px(5.0),
            shrink: 0.0,
            margin: Margin { left: Side::Auto, ..Margin::default() },
            ..Style::default()
        };
        self.add("chevron", chevron, Paint::Chevron { color: t.text });
        self.close();
        resp
    }

    /// A text leaf that ends in an ellipsis when its box is narrower than the text. `min-width: 0` lets it
    /// shrink in a flex row; a grid fr column already stretches it.
    pub fn ellipsis(&mut self, s: &str, look: TextLook) {
        let color = self.tone_color(look.tone);
        self.add_ellipsis(s, look, color, false);
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
                    t.faded(t.muted, FADE_DISABLED)
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
                    t.faded(t.text, FADE_DISABLED)
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
            (t.faded(t.muted, FADE_DISABLED), t.faded(t.edge, FADE_DISABLED))
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

    /// `.setting button.field`: shows `on_word` or `off_word` (the app's catalog) and flips `on` on a click,
    /// before painting, so the frame already shows the new word. `Resp.clicked` means the value changed.
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
        let mut frame = if resp.hovered || resp.held { t.text } else { t.edge };
        let mut word = t.text;
        if opts.disabled {
            frame = t.faded(frame, FADE_DISABLED);
            word = t.faded(word, FADE_DISABLED);
        }
        let style = Style {
            // The padding counts the frame; the word is right-aligned.
            pad: opts.pad,
            min_w: opts.min_w,
            justify: Justify::End,
            align_items: Align::Start,
            ..Style::default()
        };
        self.open(key, id, style, Paint::Box { fill: None, frame: Some(frame) });
        let look = TextLook::body(Tone::Text);
        self.add_text(if *on { on_word } else { off_word }, look, word, false, Style::default());
        self.close();
        resp
    }
}
