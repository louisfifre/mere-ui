//! The text field: `input[type=text]` (preview.html line 143), the profile name's compact one (line 154)
//! and, through `number.rs`, the number field (lines 123 and 170). One call builds the box, reads the pointer
//! and the keys for it, and paints the frame, the selection, the text and the caret.
//!
//! At most one field has the keyboard, so the editing state lives in `Ui::focus`, not in a map: it is made
//! when a field gains focus and dropped when it loses it. Focus follows the pointer within the frame, before
//! the keys are read, so a press on one field and the typing after it land on the right one whichever field
//! was built first.

use mere_base::{Color, Cursor, DrawList, Ime, Key, Mods, Rect, Vec2, observe};
use mere_layout::{Dim, Edges, Measured, Style};

use crate::compose::{self, Compose};
use crate::edit::{Accept, Edit, Outcome};
use crate::number::{self, NumberSpec, js_number};
use crate::text::{ASCENT_EM, DESCENT_EM, TextLook, Tone, baseline, cells, cells_of, half_leading};
use crate::tokens::{FADE_DISABLED, FRAME, INSET, INSET_HAIR, INSET_TIGHT};
use crate::ui::{Paint, Resp, Ui};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FieldLook {
    /// `input[type=text]`, line 143: padding 3px 6px.
    Text,
    /// `#profiles input`, line 154: padding 1px 6px.
    Compact,
}

#[derive(Copy, Clone, Debug)]
pub struct FieldOpts<'a> {
    pub look: FieldLook,
    /// Shown, muted, whenever the text is empty, focused or not.
    pub placeholder: &'a str,
    /// `maxlength`: counted in chars (a browser counts UTF-16 units). 0 means no limit.
    pub max_chars: usize,
    /// The border-box width. `Auto` is 20 chars, the browser's `size=20` (an interpretation: every use on
    /// the page sets a width or stretches).
    pub width: Dim,
    /// Takes the keyboard with all the text selected when it is built and does not already have it. Pass it
    /// on the frames that follow the thing the page focuses on (a profile just created), not on every frame:
    /// a field that keeps asking takes the focus back whenever another field has it. The field that lost it
    /// is not told.
    pub focus: bool,
    /// `:disabled` (launcher line 207): no focus, no I-beam, the frame and the text at half opacity.
    pub disabled: bool,
    /// The border-box height; `Auto` is one line. `Px(52)` is the launcher's textarea stand-in (line 195): one
    /// line at the top of a taller box.
    pub height: Dim,
}

impl Default for FieldOpts<'_> {
    fn default() -> Self {
        FieldOpts { look: FieldLook::Text, placeholder: "", max_chars: 0, width: Dim::Auto, focus: false, disabled: false, height: Dim::Auto }
    }
}

/// What the person did to a field this frame.
#[derive(Copy, Clone, Debug)]
pub struct FieldResp {
    pub resp: Resp,
    /// The field has the keyboard after this frame's events.
    pub focused: bool,
    /// The value changed this frame (typed, pasted, stepped, undone, or committed).
    pub changed: bool,
    /// Enter was pressed.
    pub submitted: bool,
    /// Escape was pressed.
    pub cancelled: bool,
    /// The field lost the keyboard: a press elsewhere, Tab, Enter or Escape.
    pub blurred: bool,
    /// An input method composition is showing in the field after this frame's events.
    pub composing: bool,
}

pub(crate) enum Typed {
    Key(Key, Mods),
    Char(char),
    Ime(Ime),
}

/// What a focused field is doing. Dropped with the focus.
pub(crate) struct FieldState {
    pub edit: Edit,
    /// How far the text is pushed left to keep the caret visible, in dip.
    pub scroll: f32,
    /// A number's value when it gained focus: what Escape and an unreadable buffer go back to.
    pub at_focus: f64,
    /// The left button went down inside the field and is still the drag that extends the selection.
    pub drag: bool,
    /// The input method's composition, while there is one: shown at the caret, never part of the value.
    pub compose: Option<Compose>,
}

pub(crate) struct Focus {
    pub id: crate::id::Id,
    /// None until the field is built with the focus: the state needs the field's own text.
    pub state: Option<FieldState>,
    /// Select all the text when the state is made (Tab, `FieldOpts::focus`).
    pub select_all: bool,
}

/// Everything `paint_field` needs, copied out of the frame so painting reads nothing else.
pub(crate) struct FieldPaint {
    pub text: String,
    pub placeholder: String,
    pub right: bool,
    pub scroll: f32,
    pub frame: Color,
    pub text_color: Color,
    pub muted: Color,
    pub select: Color,
    pub select_text: Color,
    /// The caret's cell column, when focused with nothing selected: the browser hides it over a selection.
    pub caret: Option<usize>,
    /// The selection's cell columns, when focused.
    pub selection: Option<(usize, usize)>,
    /// The composition's cells (start, end), underlined once.
    pub underline: Option<(usize, usize)>,
    /// The cell where the input method may write, for a focused text field: the composition's start while
    /// composing, else the caret.
    pub spot: Option<usize>,
}

/// What `field` needs besides its target: the box's size and how it behaves.
#[derive(Copy, Clone)]
pub(crate) struct Extra {
    pub width: Dim,
    pub height: Dim,
    pub want_focus: bool,
    pub disabled: bool,
    /// The padding's vertical size (border included), where a field is not the shape's own.
    pub pad_v: Option<f32>,
    /// Text hugs the right edge; None is the shape's own.
    pub right: Option<bool>,
}

impl Extra {
    pub(crate) fn new(width: Dim) -> Extra {
        Extra { width, height: Dim::Auto, want_focus: false, disabled: false, pad_v: None, right: None }
    }
}

/// What a field edits.
pub(crate) enum Target<'a> {
    Text { s: &'a mut String, max_chars: usize },
    Number { v: &'a mut f64, spec: NumberSpec },
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Shape {
    Text,
    Compact,
    Number,
}

impl Shape {
    /// Padding plus the 1 px border.
    fn pad(self) -> Edges {
        match self {
            Shape::Text => Edges::vh(FRAME + INSET_TIGHT, FRAME + INSET),
            Shape::Compact => Edges::vh(FRAME + INSET_HAIR, FRAME + INSET),
            Shape::Number => Edges::vh(FRAME + INSET_HAIR, FRAME + INSET),
        }
    }
    /// `.setting .field` is right-aligned, line 170.
    fn right(self) -> bool {
        self == Shape::Number
    }
}

fn body() -> TextLook {
    TextLook::body(Tone::Text)
}

/// The box inside the pad.
fn inner_of(r: Rect, pad: Edges) -> Rect {
    Rect::new(r.x + pad.left, r.y + pad.top, r.w - pad.left - pad.right, r.h - pad.top - pad.bottom)
}

/// Where the text starts on x: right-aligned text hugs the right edge until it no longer fits.
fn text_x(inner: Rect, text_w: f32, right: bool, scroll: f32) -> f32 {
    let align = if right { (inner.w - text_w).max(0.0) } else { 0.0 };
    inner.x + align - scroll
}

impl Ui {
    /// A text field editing `s`. `s` changes as the person types, so the application always holds the
    /// current text; `FieldResp::changed` says when it did.
    pub fn text_field(&mut self, key: &str, s: &mut String, opts: FieldOpts) -> FieldResp {
        let shape = if opts.look == FieldLook::Compact { Shape::Compact } else { Shape::Text };
        let extra = Extra { height: opts.height, want_focus: opts.focus, disabled: opts.disabled, ..Extra::new(opts.width) };
        self.field(key, Target::Text { s, max_chars: opts.max_chars }, shape, opts.placeholder, extra)
    }

    pub(crate) fn field(&mut self, key: &str, mut target: Target, shape: Shape, placeholder: &str, extra: Extra) -> FieldResp {
        let Extra { width, height, want_focus, disabled, pad_v, right } = extra;
        let id = self.key(key);
        let resp = self.interact(id, disabled, Cursor::IBeam);
        // A disabled field is not in Tab's walk.
        if !disabled {
            self.order.push(id);
        }
        let t = *self.theme();
        let mut pad = shape.pad();
        if let Some(v) = pad_v {
            pad = Edges::vh(v, pad.left);
        }
        let right = right.unwrap_or(shape.right());
        let step = body().step();
        let rect = resp.rect;
        let (accept, max_chars) = match &target {
            Target::Text { max_chars, .. } => (Accept::Any, *max_chars),
            Target::Number { .. } => (Accept::Number, 0),
        };
        let (mut changed, mut submitted, mut cancelled, mut blurred) = (false, false, false, false);

        // ---- focus by pointer: judged on where the press went, so it does not depend on build order ----
        let press = self.input.press_pos;
        let over = !disabled && matches!((rect, press), (Some(r), Some(p)) if r.contains(p));
        let mut mine = !disabled && self.focus.as_ref().is_some_and(|f| f.id == id);
        // The state of a focus that ended this frame, for a number to commit what was typed.
        let mut ended: Option<FieldState> = None;
        if mine && press.is_some() && !over {
            ended = self.focus.take().and_then(|f| f.state);
            mine = false;
            blurred = true;
            observe!(Debug, "blur {:x}: pressed elsewhere", id.0);
        } else if !mine && over {
            // The field that had it may be built later this frame: it is told then.
            self.lost = self.focus.replace(Focus { id, state: None, select_all: false });
            mine = true;
            observe!(Debug, "focus {:x}: pressed", id.0);
        }
        if self.lost.as_ref().is_some_and(|l| l.id == id) {
            ended = self.lost.take().and_then(|l| l.state);
            blurred = true;
            observe!(Debug, "blur {:x}: another field was pressed", id.0);
        }
        if let (Some(st), Target::Number { v, spec }) = (&ended, &mut target) {
            changed |= number::commit(&st.edit.text, v, *spec, st.at_focus);
        }
        if want_focus && !mine && !disabled {
            self.focus = Some(Focus { id, state: None, select_all: true });
            mine = true;
            observe!(Debug, "focus {:x}: asked for", id.0);
        }

        if mine {
            self.start_state(&target);
            if let Some(r) = rect {
                self.pointer_in_field(r, pad, right, step, over.then(|| press.unwrap()), resp.held);
            }
            // ---- keys ----
            let typed = std::mem::take(&mut self.typed);
            for item in &typed {
                let focus = self.focus.as_mut().expect("still focused: the loop stops at a blur");
                let st = focus.state.as_mut().expect("the state was made above");
                // While a composition shows, the input method owns the keys: it eats them, and one that slipped
                // through is dropped rather than edit the value under the preedit.
                if matches!(item, Typed::Key(..) | Typed::Char(_)) && st.compose.as_ref().is_some_and(Compose::shown) {
                    observe!(Debug, "{:x}: key dropped while composing", id.0);
                    continue;
                }
                let outcome = match item {
                    Typed::Ime(i) => match &target {
                        Target::Text { .. } => compose::apply(st, i, max_chars),
                        Target::Number { .. } => {
                            observe!(Debug, "{:x}: input method ignored in a number field", id.0);
                            Outcome::None
                        }
                    },
                    Typed::Key(k, m) => st.edit.key(*k, *m, &mut *self.clipboard, max_chars, accept),
                    Typed::Char(c) => {
                        if st.edit.insert(c.encode_utf8(&mut [0; 4]), max_chars, accept) {
                            Outcome::Edited
                        } else {
                            Outcome::None
                        }
                    }
                };
                match outcome {
                    Outcome::None | Outcome::Moved => {}
                    Outcome::Edited => match &mut target {
                        Target::Text { s, .. } => {
                            **s = st.edit.text.clone();
                            changed = true;
                        }
                        // Live, like the page's `input` event: unclamped, and a half-typed "1e" leaves the
                        // value alone until it parses.
                        Target::Number { v, .. } => changed |= number::live(&st.edit.text, v),
                    },
                    Outcome::StepUp | Outcome::StepDown => {
                        if let Target::Number { v, spec } = &mut target {
                            changed |= number::step(v, *spec, outcome == Outcome::StepUp, &mut st.edit);
                        }
                    }
                    Outcome::Submit | Outcome::Cancel | Outcome::Tab { .. } => {
                        let st = self.focus.take().and_then(|f| f.state).expect("the state was made above");
                        match (&mut target, outcome) {
                            (Target::Number { v, .. }, Outcome::Cancel) => {
                                // Escape gives the value back; a text field keeps what was typed, as the
                                // browser does.
                                changed |= **v != st.at_focus;
                                **v = st.at_focus;
                            }
                            (Target::Number { v, spec }, _) => changed |= number::commit(&st.edit.text, v, *spec, st.at_focus),
                            _ => {}
                        }
                        submitted = outcome == Outcome::Submit;
                        cancelled = outcome == Outcome::Cancel;
                        if let Outcome::Tab { back } = outcome {
                            self.tab = Some((id, back));
                        }
                        blurred = true;
                        mine = false;
                        observe!(Debug, "blur {:x}: {outcome:?}", id.0);
                        break;
                    }
                }
            }
        }

        // ---- scrolling: keep the caret in view; a field that is not focused shows its start ----
        let focused = mine && self.focus.as_ref().is_some_and(|f| f.id == id);
        let mut shown = match (&target, focused) {
            (Target::Text { s, .. }, _) => (**s).clone(),
            (Target::Number { .. }, true) => self.state().edit.text.clone(),
            (Target::Number { v, .. }, false) => js_number(**v),
        };
        // A composition replaces the selection in the display (the commit will) and sits at its start:
        // (start cell, end cell, caret cell).
        let mut comp: Option<(usize, usize, usize)> = None;
        if let (Target::Text { .. }, true) = (&target, focused) {
            let st = self.state();
            if let Some(c) = st.compose.as_ref().filter(|c| c.shown()) {
                let (lo, hi) = st.edit.selection().unwrap_or((st.edit.cursor, st.edit.cursor));
                shown = format!("{}{}{}", &st.edit.text[..lo], c.text, &st.edit.text[hi..]);
                let a = st.edit.cells_before(lo);
                let before_caret: String = c.text.chars().take(c.cursor).collect();
                comp = Some((a, a + cells_of(&c.text), a + cells_of(&before_caret)));
            }
        }
        let text_w = cells_of(&shown) as f32 * step;
        let mut scroll = 0.0;
        let (mut caret, mut selection, mut spot) = (None, None, None);
        if focused {
            let st = self.state();
            let (cur, sel) = match comp {
                Some((_, _, c)) => (c, None),
                None => (st.edit.cells_before(st.edit.cursor), st.edit.selection().map(|(a, b)| (st.edit.cells_before(a), st.edit.cells_before(b)))),
            };
            if shape != Shape::Number {
                spot = Some(comp.map_or_else(|| st.edit.cells_before(st.edit.cursor), |(a, _, _)| a));
            }
            if let Some(r) = rect {
                let inner_w = inner_of(r, pad).w;
                let caret_px = cur as f32 * step;
                let mut sc = st.scroll;
                if text_w <= inner_w {
                    sc = 0.0;
                } else {
                    if caret_px - sc > inner_w {
                        sc = caret_px - inner_w;
                    }
                    if caret_px - sc < 0.0 {
                        sc = caret_px;
                    }
                    sc = sc.clamp(0.0, text_w - inner_w);
                }
                st.scroll = sc;
                scroll = sc;
            }
            selection = sel;
            caret = sel.is_none().then_some(cur);
        }

        // ---- the box ----
        let paint = FieldPaint {
            text: shown,
            placeholder: placeholder.to_string(),
            right,
            scroll,
            frame: if disabled { t.faded(t.edge, FADE_DISABLED) } else if focused { t.text } else { t.edge },
            text_color: if disabled { t.faded(t.text, FADE_DISABLED) } else { t.text },
            muted: if disabled { t.faded(t.muted, FADE_DISABLED) } else { t.muted },
            select: t.select,
            select_text: t.select_text,
            caret,
            selection,
            underline: comp.map(|(a, b, _)| (a, b)),
            spot,
        };
        // Number fields are as wide as their text; the grid cell they sit in stretches them.
        let content_w = if shape == Shape::Number { text_w } else { 20.0 * step };
        let measured = Measured { size: Vec2::new(content_w, body().line), baseline: baseline(body()) };
        self.add_keyed_leaf("field", id, Style { w: width, h: height, pad, ..Style::default() }, Paint::Field(Box::new(paint)), measured);
        FieldResp { resp, focused, changed, submitted, cancelled, blurred, composing: comp.is_some() }
    }

    /// The focused field's state; callers know there is one.
    fn state(&mut self) -> &mut FieldState {
        self.focus.as_mut().and_then(|f| f.state.as_mut()).expect("a focused field has its state")
    }

    /// Makes the editing state of a field that just got the keyboard, from its own value; and, when the
    /// application changed the text under a focused field, takes the application's text.
    fn start_state(&mut self, target: &Target) {
        let focus = self.focus.as_mut().expect("the caller checked");
        let st = focus.state.get_or_insert_with(|| match target {
            Target::Text { s, .. } => FieldState { edit: Edit::new(s), scroll: 0.0, at_focus: 0.0, drag: false, compose: None },
            Target::Number { v, .. } => FieldState { edit: Edit::new(&js_number(**v)), scroll: 0.0, at_focus: **v, drag: false, compose: None },
        });
        if std::mem::take(&mut focus.select_all) {
            st.edit.select_all();
        }
        // A number ignores an external change while it is being typed: its buffer is not the value.
        if let Target::Text { s, .. } = target {
            if **s != st.edit.text {
                st.edit = Edit::new(s);
            }
        }
    }

    /// A press inside the field puts the caret where it landed (or selects a word or everything, by the click
    /// count); a held left button that started inside drags the selection's end along.
    fn pointer_in_field(&mut self, r: Rect, pad: Edges, right: bool, step: f32, press: Option<Vec2>, held: bool) {
        let (clicks, pos) = (self.input.clicks, self.input.pos);
        let st = self.state();
        let inner = inner_of(r, pad);
        let x0 = text_x(inner, cells_of(&st.edit.text) as f32 * step, right, st.scroll);
        // The cell position of a pointer x: the caret takes the nearest gap between two chars, a word click
        // the char under the pointer.
        let cell_at = |x: f32| (x - x0) / step;
        if let Some(p) = press {
            match clicks {
                0 | 1 => {
                    let i = st.edit.byte_near_cell(cell_at(p.x));
                    st.edit.set_caret(i, false);
                    st.drag = true;
                }
                2 => {
                    let i = st.edit.byte_under_cell(cell_at(p.x));
                    st.edit.select_word_at(i);
                    st.drag = false;
                }
                _ => {
                    st.edit.select_all();
                    st.drag = false;
                }
            }
        } else if held && st.drag {
            if let Some(p) = pos {
                let i = st.edit.byte_near_cell(cell_at(p.x));
                st.edit.set_caret(i, true);
            }
        }
    }
}

/// Paints a field in `r` (absolute) with padding `pad`: the frame, then, clipped to the content box (one
/// pixel wider on the right so a caret at the very edge shows), the selection, the text, the text again in
/// the selection's colour inside the selection, and the caret.
pub(crate) fn paint_field(list: &mut DrawList, f: &FieldPaint, r: Rect, pad: Edges) -> Option<Rect> {
    let look = body();
    let step = look.step();
    let inner = inner_of(r, pad);
    list.frame(r, 1.0, f.frame);
    list.push_clip(Rect::new(inner.x, inner.y, inner.w + 1.0, inner.h));
    // The glyph box is ascent plus descent tall and the CSS line puts half the leading above it.
    let top = inner.y + half_leading(look);
    let glyph_h = (ASCENT_EM + DESCENT_EM) * look.size;
    let chars = f.text.chars().count();
    let x0 = text_x(inner, cells_of(&f.text) as f32 * step, f.right, f.scroll);
    if let Some((a, b)) = f.selection {
        list.fill(Rect::new(x0 + a as f32 * step, top, (b - a) as f32 * step, glyph_h), f.select);
    }
    if chars == 0 {
        if !f.placeholder.is_empty() {
            let pw = f.placeholder.chars().count() as f32 * step;
            list.text(Vec2::new(text_x(inner, pw, f.right, 0.0), top), f.placeholder.clone(), look.size, f.muted);
        }
    } else {
        draw_cells(list, &f.text, x0, top, look, f.text_color);
        if let Some((a, b)) = f.selection {
            list.push_clip(Rect::new(x0 + a as f32 * step, top, (b - a) as f32 * step, glyph_h));
            draw_cells(list, &f.text, x0, top, look, f.select_text);
            list.pop_clip();
        }
    }
    if let Some(c) = f.caret {
        // One dip wide: exact at scale 1, and it lands on the pixel grid by rounding.
        list.fill(Rect::new((x0 + c as f32 * step).round(), top, 1.0, glyph_h), f.text_color);
    }
    if let Some((a, b)) = f.underline {
        // One pixel under the whole composition, in the text colour: the composition's attributes (clauses,
        // the target clause) are not read.
        list.fill(Rect::new(x0 + a as f32 * step, (top + ASCENT_EM * look.size + 1.5).round(), (b - a) as f32 * step, 1.0), f.text_color);
    }
    list.pop_clip();
    f.spot.map(|c| Rect::new((x0 + c as f32 * step).clamp(inner.x, inner.x + inner.w), top, 1.0, glyph_h))
}

/// Draws `text` from `x0` cell by cell. A maximal run of one-cell chars is one command at its cell, so plain
/// text is one command as before. A two-cell char is its own command, centred in its two cells: the offset
/// assumes the fallback font's glyph advances 1 em (an INTERPRETATION; IBM Plex Mono has no CJK glyphs, so
/// DirectWrite falls back to a system font).
fn draw_cells(list: &mut DrawList, text: &str, x0: f32, top: f32, look: TextLook, color: Color) {
    let step = look.step();
    let (mut cell, mut run, mut run_cell) = (0usize, String::new(), 0usize);
    for c in text.chars() {
        if cells(c) == 1 {
            if run.is_empty() {
                run_cell = cell;
            }
            run.push(c);
            cell += 1;
        } else {
            if !run.is_empty() {
                list.text(Vec2::new(x0 + run_cell as f32 * step, top), std::mem::take(&mut run), look.size, color);
            }
            list.text(Vec2::new(x0 + cell as f32 * step + (2.0 * step - look.size) / 2.0, top), c.to_string(), look.size, color);
            cell += 2;
        }
    }
    if !run.is_empty() {
        list.text(Vec2::new(x0 + run_cell as f32 * step, top), run, look.size, color);
    }
}