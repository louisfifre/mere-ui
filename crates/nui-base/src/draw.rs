//! Interface drawing as plain data: a list of commands any backend can draw, or a test can read.

use crate::Color;
use crate::math::{Rect, Vec2};

/// How heavy text is drawn: the page uses regular and the 600 weight of its main button.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum Weight {
    #[default]
    Regular,
    SemiBold,
}

#[derive(Clone, Debug)]
pub enum Cmd {
    Fill { rect: Rect, color: Color },
    /// A frame of `width` physical pixels, drawn inside `rect`: 1 is the page's hairline at any scale.
    Frame { rect: Rect, width_px: f32, color: Color },
    Text { at: Vec2, text: String, size: f32, color: Color, weight: Weight },
    /// What is drawn up to the matching `PopClip` is cut to `rect`. Clips nest: the visible part is the
    /// intersection of the rects pushed so far.
    PushClip { rect: Rect },
    PopClip,
}

#[derive(Clone, Debug, Default)]
pub struct DrawList {
    pub cmds: Vec<Cmd>,
}

impl DrawList {
    pub fn fill(&mut self, rect: Rect, color: Color) {
        self.cmds.push(Cmd::Fill { rect, color });
    }
    pub fn frame(&mut self, rect: Rect, width_px: f32, color: Color) {
        self.cmds.push(Cmd::Frame { rect, width_px, color });
    }
    /// `at` is the top-left corner of the text's line box.
    pub fn text(&mut self, at: Vec2, text: impl Into<String>, size: f32, color: Color) {
        self.text_weighted(at, text, size, color, Weight::Regular);
    }
    /// Like 	ext, in the given weight.
    pub fn text_weighted(&mut self, at: Vec2, text: impl Into<String>, size: f32, color: Color, weight: Weight) {
        self.cmds.push(Cmd::Text { at, text: text.into(), size, color, weight });
    }
    /// Starts clipping to `rect`: the box of an `overflow: hidden` or scrolling area. Clips nest; what is drawn
    /// between this and the matching `pop_clip` is cut to the rect.
    pub fn push_clip(&mut self, rect: Rect) {
        self.cmds.push(Cmd::PushClip { rect });
    }
    /// Ends the clip of the last `push_clip`.
    pub fn pop_clip(&mut self) {
        self.cmds.push(Cmd::PopClip);
    }
}
