//! Label, button and toggle in the page's looks.

use nui_base::{Cmd, Color, Cursor, Rect, Weight};
use nui_layout::{Align, Dim, Edges, Justify, Margin, Side, Style};

use crate::text::{TextLook, Tone};
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
    /// Its menu comes with the step that needs it. UNVERIFIED: the value's place (7 px from the left, centred
    /// vertically); the reference capture's select was empty. The height and the arrow were measured in Chrome.
    pub fn select(&mut self, key: &str, value: &str) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, false, Cursor::Hand);
        let t = *self.theme();
        let style = Style { h: Dim::Px(23.0), pad: Edges::new(3.0, 5.0, 3.0, 7.0), align_items: Align::Center, ..Style::default() };
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
                    h: Dim::Px(34.0),
                    pad: Edges::new(1.0, 13.0, 1.0, 13.0),
                    align_items: Align::Center,
                    justify: Justify::Start,
                    grow: opts.grow,
                    ..Style::default()
                };
                let (paint, word) = if opts.disabled {
                    (Paint::Box { fill: None, frame: Some(t.edge) }, t.muted)
                } else if lit {
                    // opacity .85 over the ground leaves the ground-coloured word unchanged.
                    let c = t.faded(t.text, 0.85);
                    (Paint::Box { fill: Some(c), frame: Some(c) }, t.ground)
                } else {
                    (Paint::Box { fill: Some(t.text), frame: Some(t.text) }, t.ground)
                };
                (style, paint, TextLook { weight: Weight::SemiBold, ..body }, word, false, false)
            }
            ButtonLook::Running { share } => {
                let style = Style {
                    h: Dim::Px(34.0),
                    pad: Edges::new(1.0, 13.0, 1.0, 13.0),
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
                    pad: if link { Edges::vh(1.0, 0.0) } else { Edges::vh(1.0, 6.0) },
                    grow: opts.grow,
                    ..Style::default()
                };
                let word = if opts.disabled {
                    t.faded(t.muted, 0.35)
                } else if lit {
                    t.text
                } else {
                    t.muted
                };
                (style, Paint::None, body, word, link, false)
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

    /// `#stopRun` (lines 219-222): a 34 px square that does not shrink, framed, with a filled square as its icon.
    /// The icon is the svg's path `M5 5h14v14H5z` of a 24 unit box shown at 12 px: 7 px, 13.5 px from the corner
    /// (1 border + 10 of centring + 2.5). UNVERIFIED: the half pixel's look (Chrome smooths it, Direct2D draws what
    /// it draws).
    pub fn stop_button(&mut self, key: &str, disabled: bool) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, disabled, Cursor::Hand);
        let t = *self.theme();
        let lit = !disabled && (resp.hovered || resp.held);
        let (colour, frame) = if disabled {
            (t.faded(t.muted, 0.4), t.faded(t.edge, 0.4))
        } else if lit {
            (t.text, t.text)
        } else {
            (t.muted, t.edge)
        };
        let style = Style { w: Dim::Px(34.0), h: Dim::Px(34.0), shrink: 0.0, ..Style::default() };
        let cmds = vec![
            Cmd::Frame { rect: Rect::new(0.0, 0.0, 34.0, 34.0), width_px: 1.0, color: frame },
            Cmd::Fill { rect: Rect::new(13.5, 13.5, 7.0, 7.0), color: colour },
        ];
        self.open(key, id, style, Paint::Cmds(cmds));
        self.close();
        resp
    }

    /// `.setting button.field`: shows `on_word` or `off_word` (the app's catalog) and flips `on` on a click,
    /// before painting, so the frame already shows the new word. `Resp.clicked` means the value changed.
    pub fn toggle(&mut self, key: &str, on: &mut bool, on_word: &str, off_word: &str) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, false, Cursor::Hand);
        if resp.clicked {
            *on = !*on;
        }
        let t = *self.theme();
        let frame = if resp.hovered || resp.held { t.text } else { t.edge };
        let style = Style {
            // padding 2px 6px plus the 1 px border; text-align: right, line 170.
            pad: Edges::vh(3.0, 7.0),
            justify: Justify::End,
            align_items: Align::Start,
            ..Style::default()
        };
        self.open(key, id, style, Paint::Box { fill: None, frame: Some(frame) });
        let look = TextLook::body(Tone::Text);
        self.add_text(if *on { on_word } else { off_word }, look, t.text, false, Style::default());
        self.close();
        resp
    }
}
