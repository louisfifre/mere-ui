//! Interface drawing as plain data: a list of commands any backend can draw, or a test can read.

use crate::{Color, ImageId};
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
    /// The whole uploaded image stretched over `rect` (DIP), nearest-neighbour: each texel is a hard-edged
    /// block, as the page's pixelated canvas.
    Image { image: ImageId, rect: Rect },
    /// An antialiased filled circle.
    Disc { center: Vec2, radius: f32, color: Color },
    /// An antialiased circle outline of `width` DIP, centred on `radius` (a canvas stroke).
    Ring { center: Vec2, radius: f32, width: f32, color: Color },
    /// An open stroke of width DIP through the points: round joins, flat ends (SVG stroke-linejoin: round
    /// with the default butt caps). Fewer than two points draw nothing.
    Polyline { points: Vec<Vec2>, width: f32, color: Color },
}

impl Cmd {
    /// The same command moved by d: its rect, text position or centre. A clip's end has no place, so it stays.
    pub fn translated(&self, d: Vec2) -> Cmd {
        let mv = |r: Rect| Rect::new(r.x + d.x, r.y + d.y, r.w, r.h);
        let mp = |p: Vec2| Vec2::new(p.x + d.x, p.y + d.y);
        match self {
            Cmd::Fill { rect, color } => Cmd::Fill { rect: mv(*rect), color: *color },
            Cmd::Frame { rect, width_px, color } => Cmd::Frame { rect: mv(*rect), width_px: *width_px, color: *color },
            Cmd::Text { at, text, size, color, weight } => {
                Cmd::Text { at: mp(*at), text: text.clone(), size: *size, color: *color, weight: *weight }
            }
            Cmd::PushClip { rect } => Cmd::PushClip { rect: mv(*rect) },
            Cmd::PopClip => Cmd::PopClip,
            Cmd::Image { image, rect } => Cmd::Image { image: *image, rect: mv(*rect) },
            Cmd::Disc { center, radius, color } => Cmd::Disc { center: mp(*center), radius: *radius, color: *color },
            Cmd::Ring { center, radius, width, color } => {
                Cmd::Ring { center: mp(*center), radius: *radius, width: *width, color: *color }
            }
            Cmd::Polyline { points, width, color } => {
                Cmd::Polyline { points: points.iter().map(|p| mp(*p)).collect(), width: *width, color: *color }
            }
        }
    }
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
    pub fn image(&mut self, image: ImageId, rect: Rect) {
        self.cmds.push(Cmd::Image { image, rect });
    }
    pub fn disc(&mut self, center: Vec2, radius: f32, color: Color) {
        self.cmds.push(Cmd::Disc { center, radius, color });
    }
    pub fn ring(&mut self, center: Vec2, radius: f32, width: f32, color: Color) {
        self.cmds.push(Cmd::Ring { center, radius, width, color });
    }
    pub fn polyline(&mut self, points: Vec<Vec2>, width: f32, color: Color) {
        self.cmds.push(Cmd::Polyline { points, width, color });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translated_moves_every_variant_and_keeps_the_rest() {
        let d = Vec2::new(10.0, 20.0);
        let c = Color::hex(0x123456);
        match (Cmd::Fill { rect: Rect::new(1.0, 2.0, 3.0, 4.0), color: c }).translated(d) {
            Cmd::Fill { rect, color } => assert_eq!((rect, color), (Rect::new(11.0, 22.0, 3.0, 4.0), c)),
            _ => panic!(),
        }
        match (Cmd::Frame { rect: Rect::new(1.0, 2.0, 3.0, 4.0), width_px: 1.0, color: c }).translated(d) {
            Cmd::Frame { rect, width_px, .. } => assert_eq!((rect, width_px), (Rect::new(11.0, 22.0, 3.0, 4.0), 1.0)),
            _ => panic!(),
        }
        match (Cmd::Text { at: Vec2::new(1.0, 2.0), text: "hi".into(), size: 12.0, color: c, weight: Weight::SemiBold }).translated(d) {
            Cmd::Text { at, text, size, weight, .. } => {
                assert_eq!((at, text.as_str(), size, weight), (Vec2::new(11.0, 22.0), "hi", 12.0, Weight::SemiBold))
            }
            _ => panic!(),
        }
        match (Cmd::PushClip { rect: Rect::new(1.0, 2.0, 3.0, 4.0) }).translated(d) {
            Cmd::PushClip { rect } => assert_eq!(rect, Rect::new(11.0, 22.0, 3.0, 4.0)),
            _ => panic!(),
        }
        assert!(matches!(Cmd::PopClip.translated(d), Cmd::PopClip));
        match (Cmd::Image { image: ImageId(7), rect: Rect::new(1.0, 2.0, 3.0, 4.0) }).translated(d) {
            Cmd::Image { image, rect } => assert_eq!((image, rect), (ImageId(7), Rect::new(11.0, 22.0, 3.0, 4.0))),
            _ => panic!(),
        }
        match (Cmd::Disc { center: Vec2::new(1.0, 2.0), radius: 5.0, color: c }).translated(d) {
            Cmd::Disc { center, radius, .. } => assert_eq!((center, radius), (Vec2::new(11.0, 22.0), 5.0)),
            _ => panic!(),
        }
        match (Cmd::Ring { center: Vec2::new(1.0, 2.0), radius: 5.0, width: 2.0, color: c }).translated(d) {
            Cmd::Ring { center, radius, width, .. } => assert_eq!((center, radius, width), (Vec2::new(11.0, 22.0), 5.0, 2.0)),
            _ => panic!(),
        }
        match (Cmd::Polyline { points: vec![Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)], width: 1.5, color: c }).translated(d) {
            Cmd::Polyline { points, width, .. } => {
                assert_eq!((points, width), (vec![Vec2::new(11.0, 22.0), Vec2::new(13.0, 24.0)], 1.5))
            }
            _ => panic!(),
        }
    }
}

