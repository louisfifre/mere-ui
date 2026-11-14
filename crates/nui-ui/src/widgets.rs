//! Label, button and toggle in the page's looks.

use nui_base::Weight;
use nui_layout::{Align, Dim, Edges, Justify, Style};

use crate::text::{TextLook, Tone};
use crate::ui::{Paint, Resp, Ui};

/// The three page buttons this step needs. The strip, #details and pressed choices are in tab.rs, the
/// stop button is an icon button (step 17).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ButtonLook {
    /// `#generate`: filled with the text colour, 600 weight.
    Primary,
    /// A `.choice button`: a muted word.
    Plain,
    /// A `.choice button.action`: a muted underlined word.
    Link,
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
        let resp = self.interact(id, opts.disabled);
        let t = *self.theme();
        // The page has no :active style, so a held button looks exactly like a hovered one.
        let lit = resp.hovered || resp.held;
        let body = TextLook { size: t.body, line: t.body * 1.5, weight: Weight::Regular, tone: Tone::Text, tracking: 0.0 };
        let (style, paint, word_look, word, underline) = match look {
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
                (style, paint, TextLook { weight: Weight::SemiBold, ..body }, word, false)
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
                (style, Paint::None, body, word, link)
            }
        };
        self.open(key, id, style, paint);
        self.add_text(text, word_look, word, underline, Style::default());
        self.close();
        resp
    }

    /// `.setting button.field`: shows `on_word` or `off_word` (the app's catalog) and flips `on` on a click,
    /// before painting, so the frame already shows the new word. `Resp.clicked` means the value changed.
    pub fn toggle(&mut self, key: &str, on: &mut bool, on_word: &str, off_word: &str) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, false);
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
