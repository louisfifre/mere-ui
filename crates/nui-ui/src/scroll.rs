//! A vertical scroll area in the page's look: `.scroll` (preview.html lines 133-137), its thin scrollbar and
//! the fade of its bottom 28 px. The application owns the offset (`&mut f32`), as it owns a fold's state; the
//! Ui keeps only what last frame measured, which is all a widget may judge the pointer against.

use nui_base::{Color, Rect, Vec2, observe};
use nui_layout::{Dim, Edges, Margin, Overflow, Side, Style};

use crate::id::Id;
use crate::ui::{Paint, Ui};

/// What the scrollbar reserves on the right of the box once the content overflows.
pub const GUTTER: f32 = 12.0;
/// The shortest the thumb gets.
pub const THUMB_MIN: f32 = 16.0;
/// How far one wheel notch or one arrow press moves the content.
pub const WHEEL_STEP: f32 = 100.0;
/// The room of one arrow at an end of the track; the gutter's width stays `GUTTER`.
pub const ARROW: f32 = 12.0;
/// The thumb's width; the track is 1 px.
pub const THUMB_W: f32 = 3.0;
/// Where the thumb starts in the gutter, and where the 1 px track runs: on whole pixels, the track in the thumb's
/// middle column.
const THUMB_X: f32 = 5.0;
const TRACK_X: f32 = 6.0;

#[derive(Copy, Clone, Debug, Default)]
pub struct ScrollOpts {
    /// Height in px of the mask that fades the bottom of the box into the ground (0 for none).
    pub fade: f32,
}

impl ScrollOpts {
    /// The page's `.scroll` mask: the bottom 28 px fade out (line 136).
    pub const LIST: ScrollOpts = ScrollOpts { fade: 28.0 };
}

/// What the area was at the end of the last frame; zero before its first.
#[derive(Copy, Clone, Debug)]
pub struct ScrollResp {
    pub id: Id,
    /// How far the content can go up: its height minus the box's, never below 0.
    pub max: f32,
    /// True when the content is taller than the box, so the scrollbar shows.
    pub overflow: bool,
}

/// Which part of the scrollbar the pointer is on (or holds): it is drawn in the text colour instead of the
/// muted one.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BarHot {
    None,
    Thumb,
    Up,
    Down,
}

/// What the painter draws for a scroll node.
#[derive(Copy, Clone, Debug)]
pub(crate) struct ScrollPaint {
    pub id: Id,
    pub y: f32,
    pub fade: f32,
    pub gutter: bool,
    /// The theme's line, muted and text colours: the track, the resting thumb and arrows, the hot ones.
    pub line: Color,
    pub muted: Color,
    pub text: Color,
    pub hot: BarHot,
    pub ground: Color,
}

/// What painting measured, kept for the next frame's wheel, drag and gutter.
#[derive(Copy, Clone, Debug)]
pub(crate) struct ScrollMetrics {
    /// The box cut to its own ancestors' clips: where the wheel counts.
    pub visible: Rect,
    pub view: Rect,
    pub content_h: f32,
    pub max: f32,
    pub off: f32,
    pub overflow: bool,
    /// Whether the frame reserved the gutter, to compare with `overflow`.
    pub gutter: bool,
}

/// The gutter's left edge.
fn gutter_x(view: Rect) -> f32 {
    view.x + view.w - GUTTER
}

/// The track: the box's height less the room of the two arrows, which is kept at both ends even when an arrow
/// is absent, so that the thumb never jumps when one appears.
pub(crate) fn track_rect(view: Rect) -> Rect {
    Rect::new(gutter_x(view) + TRACK_X, view.y + ARROW, 1.0, (view.h - 2.0 * ARROW).max(0.0))
}

/// The thumb: its length is the share of the content that shows (of the track), and it runs along the track in
/// the same proportion as the offset along its range.
pub(crate) fn thumb_rect(view: Rect, content_h: f32, max: f32, off: f32) -> Rect {
    let track = track_rect(view);
    let h = (track.h * view.h / content_h).max(THUMB_MIN).min(track.h);
    let y = if max > 0.0 { track.y + (track.h - h) * off / max } else { track.y };
    Rect::new(gutter_x(view) + THUMB_X, y, THUMB_W, h)
}

/// Where a press grabs the thumb: the thumb's length by the whole gutter's width, since the thumb is too thin to aim at.
pub(crate) fn thumb_grip(view: Rect, thumb: Rect) -> Rect {
    Rect::new(view.x + view.w - GUTTER, thumb.y, GUTTER, thumb.h)
}

/// The two arrow buttons: the gutter's width by `ARROW`, at the top and at the bottom of the box.
pub(crate) fn arrow_rects(view: Rect) -> (Rect, Rect) {
    let x = view.x + view.w - GUTTER;
    (Rect::new(x, view.y, GUTTER, ARROW), Rect::new(x, view.y + view.h - ARROW, GUTTER, ARROW))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ArrowDir {
    Up,
    Down,
}

/// The chevron of an arrow: about 5 px wide and 3 high, centred in the arrow's rect, drawn as a polyline.
pub(crate) fn chevron(area: Rect, dir: ArrowDir) -> Vec<Vec2> {
    let (cx, cy) = (area.x + area.w / 2.0, area.y + area.h / 2.0);
    let (tip, foot) = match dir {
        ArrowDir::Up => (cy - 1.5, cy + 1.5),
        ArrowDir::Down => (cy + 1.5, cy - 1.5),
    };
    vec![Vec2::new(cx - 2.5, foot), Vec2::new(cx, tip), Vec2::new(cx + 2.5, foot)]
}

/// Which arrows show: up when there is content above the box, down when there is content below. Half a pixel
/// of slack, so that a rounding left over does not keep an arrow that leads nowhere.
pub fn arrows_shown(max: f32, off: f32) -> (bool, bool) {
    (off > 0.5, off < max - 0.5)
}

/// A flex column that fills its parent's height and scrolls: it reaches `GUTTER` into the right margin, so the
/// bar sits there, and keeps `section` after its last row. The rows come with their own gaps.
pub fn scroll_style() -> Style {
    Style {
        column: true,
        grow: 1.0,
        shrink: 1.0,
        basis: Dim::Px(0.0),
        min_h: Dim::Px(0.0),
        margin: Margin { right: Side::Px(-GUTTER), ..Margin::default() },
        pad: Edges::new(0.0, GUTTER, crate::tokens::SECTION, 0.0),
        overflow: Overflow::ScrollY,
        ..Style::default()
    }
}

impl Ui {
    /// A box that scrolls its content vertically. `y` is the offset in px from the top; the wheel over the
    /// box and a drag on the thumb change it, and it is kept within what the last frame measured. The
    /// gutter of the scrollbar is reserved from the frame after the content first overflows, which is why
    /// `want_redraw` is true after that frame.
    pub fn scroll(&mut self, key: &str, y: &mut f32, style: Style, opts: ScrollOpts, body: impl FnOnce(&mut Ui)) -> ScrollResp {
        let id = self.key(key);
        let m = self.scrolls.get(&id).copied();
        let gutter = m.is_some_and(|m| m.overflow);
        let mut style = Style { overflow: Overflow::ScrollY, ..style };
        if gutter {
            style.pad.right += GUTTER;
        }
        let node = self.open(key, id, style, Paint::None);
        body(self);

        let thumb_id = self.key("thumb");
        let mut hot = BarHot::None;
        if let Some(m) = m {
            let thumb = thumb_rect(m.view, m.content_h, m.max, m.off);
            let grip = thumb_grip(m.view, thumb);
            let (up, down) = arrow_rects(m.view);
            let (show_up, show_down) = arrows_shown(m.max, m.off);
            if m.overflow && self.input.pressed && self.active.is_none() {
                if let Some(p) = self.input.press_pos {
                    if grip.contains(p) {
                        self.active = Some(thumb_id);
                        self.grab = Some(p.y - thumb.y);
                    } else if show_up && up.contains(p) {
                        *y -= WHEEL_STEP;
                    } else if show_down && down.contains(p) {
                        *y += WHEEL_STEP;
                    }
                }
            }
            if self.active == Some(thumb_id) && self.input.down {
                let track = track_rect(m.view);
                let room = track.h - thumb.h;
                if let (Some(p), Some(grab)) = (self.input.pos, self.grab) {
                    if room > 0.0 {
                        *y = (p.y - grab - track.y) / room * m.max;
                    }
                }
            }
            if m.overflow {
                hot = if self.active == Some(thumb_id) {
                    BarHot::Thumb
                } else {
                    match self.input.pos {
                        Some(p) if self.active.is_none() && grip.contains(p) => BarHot::Thumb,
                        Some(p) if self.active.is_none() && show_up && up.contains(p) => BarHot::Up,
                        Some(p) if self.active.is_none() && show_down && down.contains(p) => BarHot::Down,
                        _ => BarHot::None,
                    }
                };
            }
            if self.input.wheel != 0.0 && m.max > 0.0 && self.input.wheel_pos.is_some_and(|p| m.visible.contains(p)) {
                *y -= self.input.wheel * WHEEL_STEP;
                // Used up: an outer area, which closes later, does not get it. The innermost one wins.
                self.input.wheel = 0.0;
                observe!(Debug, "scroll {:x} to {}", id.0, *y);
            }
            *y = y.clamp(0.0, m.max);
        } else {
            *y = y.max(0.0);
        }

        let t = *self.theme();
        self.set_paint(node, Paint::Scroll(ScrollPaint { id, y: *y, fade: opts.fade, gutter, line: t.line, muted: t.muted, text: t.text, hot, ground: t.ground }));
        self.close();
        ScrollResp { id, max: m.map_or(0.0, |m| m.max), overflow: m.is_some_and(|m| m.overflow) }
    }
}
