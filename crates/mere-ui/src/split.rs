//! The split action: one primary button in two parts. The word at the left does the action; a narrow chevron part at
//! the right, parted from it by a 1 px line, opens a menu of choices (`Trigger::Split` of the menu, which is where
//! the list, the keyboard and the closing are). What is chosen in the list is shown or remembered by the caller.
//!
//! The two parts are one action and look alike: both filled, or both outlined and faded when either is disabled (a
//! word that cannot act with a chevron that still opens the list is shown as a whole faded button, its chevron
//! lighting to the text colour under the pointer and framed in it under the keyboard, so that it can still be found).
//! Each part has its own hover, held and focus looks of the primary action (`ActionLook::Primary`). While a run is
//! under way the word takes the running look (`ActionLook::Running`) and the chevron the same frame, with nothing
//! filled. The focus on a filled part is a 1 px frame inside it in the colour of its word, the one that shows against
//! its fill.
//!
//! The chevron points up when the list opens above the button (`SplitOpts::up`), down when it opens under it. The list
//! is as wide as the button, or as the rect `SplitOpts::list_over` gives (a row of several actions).
//!
//! Keyboard: Enter or Space on the word acts; Down, Alt+Down, Enter or Space on the chevron opens the list.
//!
//! ```ignore
//! let r = ui.split_action("generate", "Generate", ActionLook::Primary, &mut open, SplitOpts::default(), |ui| {
//!     ui.group("details", |ui| { /* immediate rows: the chosen one is the caller's */ });
//! });
//! if r.word.is_some_and(|w| w.clicked) { /* act */ }
//! ```

use mere_base::Rect;
use mere_layout::{Align, Dim, Edges, Margin, Side, Style};

use crate::id::Id;
use crate::menu::{MenuOpts, MenuResp, Trigger};
use crate::text::{TextLook, Tone};
use mere_base::Weight;
use crate::tokens::{CONTROL_LARGE, FADE_LIT, FRAME, INSET_WIDE};
use crate::ui::{Paint, Ui};
use crate::widgets::ActionLook;

/// What differs between split actions.
#[derive(Copy, Clone, Debug, Default)]
pub struct SplitOpts {
    /// The word cannot act; the chevron still opens the list.
    pub word_disabled: bool,
    /// The chevron opens nothing, and an open list closes.
    pub chevron_disabled: bool,
    /// The list opens above the button (a control at the bottom of a panel) instead of under it.
    pub up: bool,
    /// The list takes this rect's left edge and width, as the caller's last frame laid it out (`Ui::rect`), instead of
    /// the button's.
    pub list_over: Option<Rect>,
}

/// What the pointer and the keyboard did to the parts, as the last frame left them.
#[derive(Copy, Clone)]
pub(crate) struct SplitState {
    pub word_lit: bool,
    pub chevron_lit: bool,
    pub word_focus: bool,
    pub chevron_focus: bool,
}

impl Ui {
    /// A split action: `look` is `ActionLook::Primary` or `ActionLook::Running`. The answer is the menu's: `word` is the
    /// word's press, `line` the chevron's, and `open` is the caller's, as for `menu`. A press on the word closes an open
    /// list.
    pub fn split_action(&mut self, key: &str, word: &str, look: ActionLook, open: &mut bool, opts: SplitOpts, rows: impl FnOnce(&mut Ui)) -> MenuResp {
        debug_assert!(matches!(look, ActionLook::Primary | ActionLook::Running { .. }), "a split action is primary or running");
        let menu = MenuOpts { up: opts.up, disabled: opts.chevron_disabled, over: opts.list_over, ..MenuOpts::default() };
        self.menu(key, Trigger::Split { word, look, word_disabled: opts.word_disabled }, open, menu, rows)
    }

    /// The three boxes of the button. When both parts are filled the 1 px line between them is the ground showing
    /// through; otherwise (a part outlined or running) the two frames share their line, overlapped by `FRAME`.
    pub(crate) fn split_parts(&mut self, word_id: Id, chevron_id: Id, text: &str, look: ActionLook, word_off: bool, chevron_off: bool, up: bool, st: SplitState) {
        let t = *self.theme();
        let running = match look {
            ActionLook::Running { share } => Some(share),
            _ => None,
        };
        // One look for both parts: faded as soon as one cannot act, though a usable chevron of a faded split still
        // answers the pointer and the keyboard (text colour), so that it is not lost.
        let faded = word_off || chevron_off;
        // (paint, colour of what is drawn in the part) of a primary part; the focus is a frame in that colour.
        let primary = |lit: bool, focus: bool| {
            let (fill, frame, ink) = if faded {
                (None, if focus { t.text } else { t.edge }, if lit || focus { t.text } else { t.muted })
            } else if lit {
                let c = t.faded(t.fill, FADE_LIT);
                (Some(c), c, t.on_fill)
            } else {
                (Some(t.fill), t.fill, t.on_fill)
            };
            let frame = if focus && !faded { t.on_fill } else { frame };
            (Paint::Box { fill, frame: Some(frame) }, ink)
        };
        let filled = running.is_none() && !faded;
        let row = Style { w: Dim::Pct(100.0), h: Dim::Px(CONTROL_LARGE), ..Style::default() };
        self.node("split", row, |ui| {
            let word_style = Style { h: Dim::Px(CONTROL_LARGE), pad: Edges::vh(FRAME, FRAME + INSET_WIDE), align_items: Align::Center, grow: 1.0, min_w: Dim::Px(0.0), ..Style::default() };
            let look_word = TextLook { weight: Weight::SemiBold, ..TextLook::body(Tone::Text) };
            let (paint, ink) = match running {
                Some(share) => (Paint::Running { frame: if faded { t.edge } else { t.text }, bar: t.line, share }, t.text),
                None => primary(st.word_lit, st.word_focus),
            };
            ui.open("word", word_id, word_style, paint);
            ui.add_text(text, look_word, ink, false, Style::default());
            ui.close();
            if filled {
                ui.add("rule", Style { w: Dim::Px(1.0), h: Dim::Px(CONTROL_LARGE), shrink: 0.0, ..Style::default() }, Paint::Box { fill: Some(t.ground), frame: None });
            }
            let overlap = if filled { Margin::default() } else { Margin { left: Side::Px(-FRAME), ..Margin::default() } };
            let chevron_style = Style { w: Dim::Px(CONTROL_LARGE), h: Dim::Px(CONTROL_LARGE), shrink: 0.0, align_items: Align::Center, margin: overlap, ..Style::default() };
            let (paint, ink) = match running {
                Some(_) => (Paint::Box { fill: None, frame: Some(if faded { t.edge } else { t.text }) }, t.text),
                None => primary(st.chevron_lit, st.chevron_focus),
            };
            ui.open("line", chevron_id, chevron_style, paint);
            ui.add("chevron", Style { w: Dim::Px(crate::ui::CHEVRON.x), h: Dim::Px(crate::ui::CHEVRON.y), shrink: 0.0, margin: Margin { left: Side::Auto, right: Side::Auto, ..Margin::default() }, ..Style::default() }, Paint::Chevron { color: ink, up });
            ui.close();
        });
    }
}
