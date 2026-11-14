//! The single-line text-editing model: the text, a caret and an anchor, one level of undo, and what each key
//! does to them. It knows nothing of pixels, focus or the clipboard's system: the widget in `field.rs` feeds
//! it keys and draws what it holds.
//!
//! Positions are byte indexes that always sit on char boundaries, and every step is by Rust `char`, not by
//! grapheme cluster: a combining mark or a ZWJ emoji moves one code point at a time. That is a documented limit.

use nui_base::{Clipboard, Key, Mods};

/// What typed or pasted text may contain.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Accept {
    Any,
    /// Only `0-9 . - + e E`: what an `input type=number` lets through.
    Number,
}

impl Accept {
    fn allows(self, c: char) -> bool {
        match self {
            Accept::Any => true,
            Accept::Number => c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'),
        }
    }
}

/// What a key did, for the widget to act on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    None,
    /// The caret or the selection moved: nothing to store, a frame to draw.
    Moved,
    Edited,
    Submit,
    Cancel,
    Tab { back: bool },
    /// Up and Down in a number field: the widget knows the step, the model does not.
    StepUp,
    StepDown,
}

/// What the last edit was, to decide whether the next one starts a new undo step.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Run {
    /// Anything that always makes its own step: a paste, a cut, an edit over a selection; also "no edit yet".
    None,
    Typing,
    Deleting,
}

/// A clipboard kept in memory: the default of a Ui, and what tests use.
#[derive(Default)]
pub struct MemoryClipboard(pub Option<String>);

impl Clipboard for MemoryClipboard {
    fn get(&mut self) -> Option<String> {
        self.0.clone()
    }
    fn set(&mut self, text: &str) {
        self.0 = Some(text.to_string());
    }
}

pub(crate) struct Edit {
    pub text: String,
    /// The caret: where the next typed char goes. A byte index.
    pub cursor: usize,
    /// The other end of the selection; equal to `cursor` when nothing is selected.
    pub anchor: usize,
    /// One level: the text and caret before the last step. A second Ctrl+Z swaps back (a redo).
    undo: Option<(String, usize, usize)>,
    run: Run,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Edit {
    pub fn new(text: &str) -> Edit {
        Edit { text: text.to_string(), cursor: text.len(), anchor: text.len(), undo: None, run: Run::None }
    }

    /// The selected range as (min, max), or None when the caret is alone.
    pub fn selection(&self) -> Option<(usize, usize)> {
        (self.cursor != self.anchor).then(|| (self.cursor.min(self.anchor), self.cursor.max(self.anchor)))
    }

    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.cursor = self.text.len();
        self.run = Run::None;
    }

    /// Puts the caret at `i`; the anchor follows unless the selection is being extended.
    pub fn set_caret(&mut self, i: usize, extend: bool) {
        self.cursor = i;
        if !extend {
            self.anchor = i;
        }
        self.run = Run::None;
    }

    /// Selects the maximal run of chars of the same class (word or not) around the char at `i`.
    pub fn select_word_at(&mut self, i: usize) {
        if self.text.is_empty() {
            return;
        }
        let i = if i >= self.text.len() { self.prev_char(self.text.len()) } else { i };
        let class = is_word(self.text[i..].chars().next().expect("i is inside the text"));
        let (mut lo, mut hi) = (i, self.next_char(i));
        while lo > 0 && is_word(self.text[self.prev_char(lo)..].chars().next().unwrap()) == class {
            lo = self.prev_char(lo);
        }
        while hi < self.text.len() && is_word(self.text[hi..].chars().next().unwrap()) == class {
            hi = self.next_char(hi);
        }
        self.anchor = lo;
        self.cursor = hi;
        self.run = Run::None;
    }

    pub fn prev_char(&self, i: usize) -> usize {
        self.text[..i].char_indices().next_back().map_or(0, |(b, _)| b)
    }

    pub fn next_char(&self, i: usize) -> usize {
        i + self.text[i..].chars().next().map_or(0, char::len_utf8)
    }

    /// How many chars come before byte `i`.
    pub fn col(&self, i: usize) -> usize {
        self.text[..i].chars().count()
    }

    /// The byte index of char number `c`, or the end of the text.
    pub fn byte_at_col(&self, c: usize) -> usize {
        self.text.char_indices().nth(c).map_or(self.text.len(), |(b, _)| b)
    }

    /// As a desktop text box does it: over the word's chars, then over what separates it from the next word.
    pub fn next_word(&self, mut i: usize) -> usize {
        while i < self.text.len() && is_word(self.text[i..].chars().next().unwrap()) {
            i = self.next_char(i);
        }
        while i < self.text.len() && !is_word(self.text[i..].chars().next().unwrap()) {
            i = self.next_char(i);
        }
        i
    }

    /// Back over the separators, then back over the word before them.
    pub fn prev_word(&self, mut i: usize) -> usize {
        while i > 0 && !is_word(self.text[self.prev_char(i)..].chars().next().unwrap()) {
            i = self.prev_char(i);
        }
        while i > 0 && is_word(self.text[self.prev_char(i)..].chars().next().unwrap()) {
            i = self.prev_char(i);
        }
        i
    }

    fn move_to(&mut self, i: usize, extend: bool) -> Outcome {
        self.set_caret(i, extend);
        Outcome::Moved
    }

    /// Remembers the text before an edit of kind `kind`, unless it continues the run it follows. Called
    /// only once the edit is certain to change something, so a refused key leaves no empty step.
    fn snapshot(&mut self, kind: Run) {
        if kind == Run::None || kind != self.run {
            self.undo = Some((self.text.clone(), self.cursor, self.anchor));
        }
        self.run = kind;
    }

    fn remove(&mut self, lo: usize, hi: usize, kind: Run) -> bool {
        if lo == hi {
            return false;
        }
        self.snapshot(kind);
        self.text.replace_range(lo..hi, "");
        self.cursor = lo;
        self.anchor = lo;
        true
    }

    /// Types `s` over the selection. Control characters are dropped (a pasted line break or tab disappears:
    /// UNVERIFIED against Chrome), then what `accept` refuses, then what `max_chars` (0 = no limit) has no
    /// room for, counted in chars where a browser counts UTF-16 units. True when the text changed.
    pub fn insert(&mut self, s: &str, max_chars: usize, accept: Accept) -> bool {
        self.insert_as(s, max_chars, accept, Run::Typing)
    }

    fn insert_as(&mut self, s: &str, max_chars: usize, accept: Accept, kind: Run) -> bool {
        let mut room = usize::MAX;
        let (lo, hi) = self.selection().unwrap_or((self.cursor, self.cursor));
        if max_chars > 0 {
            room = max_chars.saturating_sub(self.text.chars().count() - self.col(hi) + self.col(lo));
        }
        let ins: String = s.chars().filter(|c| *c >= ' ' && *c != '\x7f' && accept.allows(*c)).take(room).collect();
        if ins.is_empty() {
            return false;
        }
        // Typing over a selection is one step of its own, like a paste.
        let kind = if lo != hi { Run::None } else { kind };
        self.snapshot(kind);
        self.text.replace_range(lo..hi, &ins);
        self.cursor = lo + ins.len();
        self.anchor = self.cursor;
        true
    }

    pub fn key(&mut self, key: Key, mods: Mods, clip: &mut dyn Clipboard, max_chars: usize, accept: Accept) -> Outcome {
        let (ext, ctrl) = (mods.shift, mods.ctrl);
        let sel = self.selection();
        match key {
            Key::Left => {
                let to = match sel {
                    _ if ctrl => self.prev_word(self.cursor),
                    Some((lo, _)) if !ext => lo,
                    _ => self.prev_char(self.cursor),
                };
                self.move_to(to, ext)
            }
            Key::Right => {
                let to = match sel {
                    _ if ctrl => self.next_word(self.cursor),
                    Some((_, hi)) if !ext => hi,
                    _ => self.next_char(self.cursor),
                };
                self.move_to(to, ext)
            }
            Key::Home => self.move_to(0, ext),
            Key::End => self.move_to(self.text.len(), ext),
            Key::Up => match accept {
                Accept::Any => self.move_to(0, ext),
                Accept::Number => Outcome::StepUp,
            },
            Key::Down => match accept {
                Accept::Any => self.move_to(self.text.len(), ext),
                Accept::Number => Outcome::StepDown,
            },
            Key::Backspace | Key::Delete => {
                let back = key == Key::Backspace;
                let (lo, hi) = match (sel, ctrl, back) {
                    (Some(s), _, _) => s,
                    (None, true, true) => (self.prev_word(self.cursor), self.cursor),
                    (None, true, false) => (self.cursor, self.next_word(self.cursor)),
                    (None, false, true) => (self.prev_char(self.cursor), self.cursor),
                    (None, false, false) => (self.cursor, self.next_char(self.cursor)),
                };
                let kind = if sel.is_some() { Run::None } else { Run::Deleting };
                if self.remove(lo, hi, kind) { Outcome::Edited } else { Outcome::None }
            }
            Key::Enter => Outcome::Submit,
            Key::Escape => Outcome::Cancel,
            Key::Tab => Outcome::Tab { back: ext },
            Key::A if ctrl => {
                self.select_all();
                Outcome::Moved
            }
            Key::C if ctrl => {
                if let Some((lo, hi)) = sel {
                    clip.set(&self.text[lo..hi]);
                }
                Outcome::None
            }
            Key::X if ctrl => match sel {
                Some((lo, hi)) => {
                    clip.set(&self.text[lo..hi]);
                    self.remove(lo, hi, Run::None);
                    Outcome::Edited
                }
                None => Outcome::None,
            },
            Key::V if ctrl => match clip.get() {
                Some(s) if self.insert_as(&s, max_chars, accept, Run::None) => Outcome::Edited,
                _ => Outcome::None,
            },
            Key::Z if ctrl => match self.undo.take() {
                Some((text, cursor, anchor)) => {
                    self.undo = Some((std::mem::replace(&mut self.text, text), self.cursor, self.anchor));
                    self.cursor = cursor;
                    self.anchor = anchor;
                    self.run = Run::None;
                    Outcome::Edited
                }
                None => Outcome::None,
            },
            // A letter without Ctrl is typing, which arrives as a char.
            Key::A | Key::C | Key::V | Key::X | Key::Z => Outcome::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Mods = Mods { ctrl: false, shift: false };
    const CTRL: Mods = Mods { ctrl: true, shift: false };
    const SHIFT: Mods = Mods { ctrl: false, shift: true };

    fn press(e: &mut Edit, k: Key, m: Mods) -> Outcome {
        e.key(k, m, &mut MemoryClipboard::default(), 0, Accept::Any)
    }

    fn typed(e: &mut Edit, s: &str) {
        for c in s.chars() {
            e.insert(&c.to_string(), 0, Accept::Any);
        }
    }

    #[test]
    fn moves_never_split_a_char() {
        // "é" is 2 bytes, the emoji 4, "a" 1: boundaries at 0, 2, 6, 7.
        let mut e = Edit::new("é😀a");
        assert_eq!(e.cursor, 7);
        for want in [6, 2, 0, 0] {
            press(&mut e, Key::Left, NONE);
            assert_eq!(e.cursor, want);
        }
        for want in [2, 6, 7, 7] {
            press(&mut e, Key::Right, NONE);
            assert_eq!(e.cursor, want);
        }
        assert_eq!((e.col(6), e.byte_at_col(2), e.byte_at_col(9)), (2, 6, 7));
    }

    #[test]
    fn word_moves_on_a_phrase() {
        // Indexes: "deep" 0..4, space 4, "ocean_2" 5..12, space 12, "x" 13, length 14.
        // next_word from 0: over "deep" to 4, over the space to 5. From 5: over "ocean_2" (5..12) to 12, over
        // the space to 13. From 13: over "x" to 14, the end.
        let e = Edit::new("deep ocean_2 x");
        assert_eq!((e.next_word(0), e.next_word(5), e.next_word(13)), (5, 13, 14));
        // prev_word from 14: no separator, back over "x" to 13. From 13: over the space to 12, back over
        // "ocean_2" to 5. From 5: over the space to 4, back over "deep" to 0.
        assert_eq!((e.prev_word(14), e.prev_word(13), e.prev_word(5)), (13, 5, 0));
    }

    #[test]
    fn collapse_goes_to_the_near_end() {
        let mut e = Edit::new("abcdef");
        e.set_caret(4, false);
        e.set_caret(1, true);
        assert_eq!(e.selection(), Some((1, 4)));
        press(&mut e, Key::Left, NONE);
        assert_eq!((e.cursor, e.anchor), (1, 1));
        e.set_caret(4, true);
        press(&mut e, Key::Right, NONE);
        assert_eq!((e.cursor, e.anchor), (4, 4));
        // With Shift the selection extends from the anchor instead.
        press(&mut e, Key::Left, SHIFT);
        press(&mut e, Key::Left, SHIFT);
        assert_eq!(e.selection(), Some((2, 4)));
    }

    #[test]
    fn ctrl_backspace_removes_the_last_word() {
        let mut e = Edit::new("the lazy dog");
        assert_eq!(press(&mut e, Key::Backspace, CTRL), Outcome::Edited);
        assert_eq!(e.text, "the lazy ");
    }

    #[test]
    fn max_chars_cuts_typing_and_paste() {
        let mut e = Edit::new("");
        for c in "1234567".chars() {
            e.insert(&c.to_string(), 5, Accept::Any);
        }
        assert_eq!(e.text, "12345");
        let mut e = Edit::new("ab");
        assert!(e.insert("cdefgh", 5, Accept::Any));
        assert_eq!(e.text, "abcde");
        // A selection frees its room: replacing "bcd" of "abcde" by 5 chars keeps 3.
        e.set_caret(1, false);
        e.set_caret(4, true);
        e.insert("vwxyz", 5, Accept::Any);
        assert_eq!(e.text, "avwxe");
    }

    #[test]
    fn paste_drops_line_breaks_and_tabs() {
        let mut clip = MemoryClipboard(Some("a\r\nb\tc".into()));
        let mut e = Edit::new("");
        assert_eq!(e.key(Key::V, CTRL, &mut clip, 0, Accept::Any), Outcome::Edited);
        assert_eq!(e.text, "abc");
    }

    #[test]
    fn number_accepts_only_its_characters() {
        let mut e = Edit::new("");
        for c in "a1e-".chars() {
            e.insert(&c.to_string(), 0, Accept::Number);
        }
        assert_eq!(e.text, "1e-");
    }

    #[test]
    fn undo_is_one_coalesced_step_and_redo() {
        let mut e = Edit::new("");
        typed(&mut e, "abc");
        press(&mut e, Key::Z, CTRL);
        assert_eq!(e.text, "");
        press(&mut e, Key::Z, CTRL);
        assert_eq!((e.text.as_str(), e.cursor), ("abc", 3));

        let mut e = Edit::new("");
        typed(&mut e, "ab");
        press(&mut e, Key::Backspace, NONE);
        assert_eq!(e.text, "a");
        press(&mut e, Key::Z, CTRL);
        assert_eq!(e.text, "ab");
    }

    #[test]
    fn copy_cut_paste_and_select_all() {
        let mut clip = MemoryClipboard::default();
        let mut e = Edit::new("hello");
        e.key(Key::A, CTRL, &mut clip, 0, Accept::Any);
        assert_eq!(e.selection(), Some((0, 5)));
        e.key(Key::X, CTRL, &mut clip, 0, Accept::Any);
        assert_eq!((e.text.as_str(), clip.0.as_deref()), ("", Some("hello")));
        e.key(Key::V, CTRL, &mut clip, 0, Accept::Any);
        assert_eq!(e.text, "hello");
        // Ctrl+C with nothing selected leaves the clipboard alone.
        clip.0 = Some("kept".into());
        e.key(Key::C, CTRL, &mut clip, 0, Accept::Any);
        assert_eq!(clip.0.as_deref(), Some("kept"));
    }

    #[test]
    fn double_click_word_and_delete_forward() {
        let mut e = Edit::new("deep ocean");
        e.select_word_at(e.byte_at_col(7));
        assert_eq!(e.selection(), Some((5, 10)));
        e.select_word_at(4);
        assert_eq!(e.selection(), Some((4, 5)));
        e.set_caret(0, false);
        press(&mut e, Key::Delete, CTRL);
        assert_eq!(e.text, "ocean");
    }
}
