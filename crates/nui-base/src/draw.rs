//! Interface drawing as plain data: a list of commands any backend can draw, or a test can read.

use crate::Color;
use crate::math::{Rect, Vec2};

#[derive(Clone, Debug)]
pub enum Cmd {
    Fill { rect: Rect, color: Color },
    /// A frame of `width` physical pixels, drawn inside `rect`: 1 is the page's hairline at any scale.
    Frame { rect: Rect, width_px: f32, color: Color },
    Text { at: Vec2, text: String, size: f32, color: Color },
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
        self.cmds.push(Cmd::Text { at, text: text.into(), size, color });
    }
}
