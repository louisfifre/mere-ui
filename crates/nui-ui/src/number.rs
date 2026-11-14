//! `.setting .field` holding a number (preview.html lines 123 and 170).

use nui_layout::{Align, Edges, Justify, Overflow, Style};

use crate::text::{TextLook, Tone};
use crate::ui::{Paint, Resp, Ui};

impl Ui {
    /// A hairline box with the value at its right. No typing yet: editing, arrow-key steps and the focused
    /// frame come with the text field (step 11), which is why the `Resp` is returned (focus on click).
    pub fn number(&mut self, key: &str, v: f64) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, false);
        let t = *self.theme();
        let style = Style {
            // padding 2px 6px plus the 1 px border.
            pad: Edges::vh(3.0, 7.0),
            justify: Justify::End,
            align_items: Align::Start,
            // An input cuts what does not fit.
            overflow: Overflow::Hidden,
            ..Style::default()
        };
        self.open(key, id, style, Paint::Box { fill: None, frame: Some(t.edge) });
        // Rust's shortest round-trip Display gives what JS String(number) gives for the page's values; -0
        // is shown as "0", as JS does.
        let v = if v == 0.0 { 0.0 } else { v };
        self.add_text(&format!("{v}"), TextLook::body(Tone::Text), t.text, false, Style::default());
        self.close();
        resp
    }
}
