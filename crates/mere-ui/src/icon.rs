//! Icons as SVG paths: the reduced grammar the pages' icons use (`M m L l H h V v Z z`), turned into closed
//! figures in the path's own grid, which the backend fills (`Cmd::Shape`).

use mere_base::{Color, Cmd, Rect, Vec2};
use mere_layout::{Dim, Style};

use crate::id::Id;
use crate::ui::{Paint, Ui};

/// A filled path: closed figures in a `grid` x `grid` box (the SVG's viewBox), nonzero rule.
#[derive(Clone, Debug, PartialEq)]
pub struct Icon {
    pub grid: f32,
    pub figures: Vec<Vec<Vec2>>,
}

/// What a path's tokens run through: the pen, the start of the subpath and the figure being drawn.
struct Pen {
    at: Vec2,
    start: Vec2,
    figure: Vec<Vec2>,
    done: Vec<Vec<Vec2>>,
}

impl Pen {
    /// Ends the figure under way. A lone point draws nothing, so it is dropped.
    fn finish(&mut self) {
        let figure = std::mem::take(&mut self.figure);
        if figure.len() >= 2 {
            self.done.push(figure);
        }
    }
    fn line_to(&mut self, to: Vec2) {
        // After a `z` the next drawing command starts a new subpath at the point the pen went back to.
        if self.figure.is_empty() {
            self.figure.push(self.at);
        }
        self.figure.push(to);
        self.at = to;
    }
}

struct Numbers<'a> {
    src: &'a [u8],
    at: usize,
}

impl Numbers<'_> {
    fn skip_separators(&mut self) {
        while self.at < self.src.len() && (self.src[self.at].is_ascii_whitespace() || self.src[self.at] == b',') {
            self.at += 1;
        }
    }
    /// The next token is a number, not a command letter or the end.
    fn more(&mut self) -> bool {
        self.skip_separators();
        self.at < self.src.len() && matches!(self.src[self.at], b'0'..=b'9' | b'.' | b'-' | b'+')
    }
    /// A number with an optional sign and decimals. "14 7-14 7" is four numbers: a sign starts a new one.
    fn number(&mut self) -> Result<f32, String> {
        self.skip_separators();
        let from = self.at;
        if self.at < self.src.len() && matches!(self.src[self.at], b'-' | b'+') {
            self.at += 1;
        }
        let mut dot = false;
        while self.at < self.src.len() {
            match self.src[self.at] {
                b'0'..=b'9' => self.at += 1,
                b'.' if !dot => {
                    dot = true;
                    self.at += 1;
                }
                _ => break,
            }
        }
        let text = std::str::from_utf8(&self.src[from..self.at]).unwrap_or("");
        text.parse::<f32>().map_err(|_| format!("a number was expected at byte {from}, found '{text}'"))
    }
}

impl Icon {
    /// Reads `d` as the path of an SVG whose viewBox is `grid` units square. Any command outside
    /// `M m L l H h V v Z z` is an error naming it. A subpath not closed with `z` is closed when filled.
    pub fn parse(d: &str, grid: f32) -> Result<Icon, String> {
        let mut n = Numbers { src: d.as_bytes(), at: 0 };
        let mut pen = Pen { at: Vec2::new(0.0, 0.0), start: Vec2::new(0.0, 0.0), figure: Vec::new(), done: Vec::new() };
        let mut first = true;
        loop {
            n.skip_separators();
            let Some(&c) = n.src.get(n.at) else { break };
            n.at += 1;
            if !c.is_ascii_alphabetic() {
                return Err(format!("a command was expected at byte {}, found '{}'", n.at - 1, c as char));
            }
            if first && !matches!(c, b'M' | b'm') {
                return Err("a path starts with M or m".to_string());
            }
            first = false;
            let rel = c.is_ascii_lowercase();
            match c.to_ascii_uppercase() {
                b'Z' => {
                    pen.at = pen.start;
                    pen.finish();
                }
                b'M' | b'L' | b'H' | b'V' => {
                    let kind = c.to_ascii_uppercase();
                    // The first pair after a moveto moves; the pairs after it are lines, whatever the case.
                    let mut moved = kind == b'M';
                    // At least one set of arguments, then as many as follow (implicit repeats).
                    let mut count = 0;
                    while count == 0 || n.more() {
                        count += 1;
                        let base = if rel { pen.at } else { Vec2::new(0.0, 0.0) };
                        let to = match kind {
                            b'H' => Vec2::new(base.x + n.number()?, pen.at.y),
                            b'V' => Vec2::new(pen.at.x, base.y + n.number()?),
                            _ => {
                                let x = n.number()?;
                                let y = n.number()?;
                                Vec2::new(base.x + x, base.y + y)
                            }
                        };
                        if moved {
                            pen.finish();
                            pen.at = to;
                            pen.start = to;
                            pen.figure.push(to);
                            moved = false;
                        } else {
                            pen.line_to(to);
                        }
                    }
                }
                _ => return Err(format!("unsupported path command '{}'", c as char)),
            }
        }
        pen.finish();
        Ok(Icon { grid, figures: pen.done })
    }

    /// Every edge, the closing one included, runs along x or y: a pixel icon.
    pub fn axis_aligned(&self) -> bool {
        self.figures.iter().all(|f| {
            f.iter().zip(f.iter().cycle().skip(1)).all(|(a, b)| a.x == b.x || a.y == b.y)
        })
    }

    /// The unit cells the path covers, `grid` x `grid` row by row, by the nonzero rule: for a path of
    /// whole-number, axis-aligned edges. None for any other path. Tests count areas with it.
    pub fn mask(&self) -> Option<Vec<bool>> {
        let g = self.grid as usize;
        let whole = self.figures.iter().flatten().all(|p| p.x.fract() == 0.0 && p.y.fract() == 0.0);
        if !self.axis_aligned() || !whole || g == 0 {
            return None;
        }
        let mut out = vec![false; g * g];
        for row in 0..g {
            for col in 0..g {
                // The centre of a cell never lies on an edge of whole numbers.
                let (cx, cy) = (col as f32 + 0.5, row as f32 + 0.5);
                let mut winding = 0i32;
                for f in &self.figures {
                    for (a, b) in f.iter().zip(f.iter().cycle().skip(1)) {
                        // A vertical edge to the right of the centre, spanning its height, counts by direction.
                        if a.x == b.x && a.x > cx && (a.y.min(b.y)..a.y.max(b.y)).contains(&cy) {
                            winding += if b.y > a.y { 1 } else { -1 };
                        }
                    }
                }
                out[row * g + col] = winding != 0;
            }
        }
        Some(out)
    }
}

impl Ui {
    /// A `size` x `size` box showing `icon` in `color`. `crisp` snaps the vertices to device pixels and fills
    /// without antialiasing (pixel icons); a path with slopes is not crisp.
    pub fn icon(&mut self, key: &str, icon: &Icon, size: f32, color: Color, crisp: bool) -> Id {
        let id = self.key(key);
        let style = Style { w: Dim::Px(size), h: Dim::Px(size), shrink: 0.0, ..Style::default() };
        self.open(key, id, style, Paint::Cmds(vec![icon_shape(icon, size, color, crisp)]));
        self.close();
        id
    }
}

/// The drawing command of an icon in a box of side `size` at the origin.
pub(crate) fn icon_shape(icon: &Icon, size: f32, color: Color, crisp: bool) -> Cmd {
    Cmd::Shape { rect: Rect::new(0.0, 0.0, size, size), grid: icon.grid, figures: icon.figures.clone(), color, crisp }
}
