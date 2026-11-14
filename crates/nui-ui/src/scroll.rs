//! A vertical scroll area in the page's look: `.scroll` (preview.html lines 133-137), its thin scrollbar and
//! the fade of its bottom 28 px. The application owns the offset (`&mut f32`), as it owns a fold's state; the
//! Ui keeps only what last frame measured, which is all a widget may judge the pointer against.

use nui_base::{Color, Rect, observe};
use nui_layout::{Dim, Edges, Margin, Overflow, Side, Style};

use crate::id::Id;
use crate::ui::{Paint, Ui};

/// What the browser's thin scrollbar reserves on the right of the box, in DIP. UNVERIFIED against Chrome.
pub const GUTTER: f32 = 11.0;
/// The shortest the thumb gets, in DIP. UNVERIFIED against Chrome.
pub const THUMB_MIN: f32 = 18.0;
/// How far one wheel notch moves the content, in DIP. UNVERIFIED against Chrome.
pub const WHEEL_STEP: f32 = 100.0;

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

/// What the painter draws for a scroll node.
#[derive(Copy, Clone, Debug)]
pub(crate) struct ScrollPaint {
    pub id: Id,
    pub y: f32,
    pub fade: f32,
    pub gutter: bool,
    pub thumb: Color,
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

/// The thumb: its length is the share of the content that shows, and it runs along the box in the same
/// proportion as the offset along its range.
pub(crate) fn thumb_rect(view: Rect, content_h: f32, max: f32, off: f32) -> Rect {
    let h = (view.h * view.h / content_h).max(THUMB_MIN);
    let y = if max > 0.0 { view.y + (view.h - h) * off / max } else { view.y };
    Rect::new(view.x + view.w - GUTTER + 2.0, y, GUTTER - 4.0, h)
}

/// `.scroll` (lines 133-137): `flex: 1; min-height: 0; overflow-y: auto; margin-right: -10px;
/// padding: 0 10px 28px 0`. A flex column: the page's rows come with their own gaps.
pub fn scroll_style() -> Style {
    Style {
        column: true,
        grow: 1.0,
        shrink: 1.0,
        basis: Dim::Px(0.0),
        min_h: Dim::Px(0.0),
        margin: Margin { right: Side::Px(-10.0), ..Margin::default() },
        pad: Edges::new(0.0, 10.0, 28.0, 0.0),
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
        if let Some(m) = m {
            let thumb = thumb_rect(m.view, m.content_h, m.max, m.off);
            if m.overflow && self.input.pressed && self.active.is_none() {
                if let Some(p) = self.input.press_pos.filter(|p| thumb.contains(*p)) {
                    self.active = Some(thumb_id);
                    self.grab = Some(p.y - thumb.y);
                }
            }
            if self.active == Some(thumb_id) && self.input.down {
                let track = m.view.h - thumb.h;
                if let (Some(p), Some(grab)) = (self.input.pos, self.grab) {
                    if track > 0.0 {
                        *y = (p.y - grab - m.view.y) / track * m.max;
                    }
                }
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
        self.set_paint(node, Paint::Scroll(ScrollPaint { id, y: *y, fade: opts.fade, gutter, thumb: t.line, ground: t.ground }));
        self.close();
        ScrollResp { id, max: m.map_or(0.0, |m| m.max), overflow: m.is_some_and(|m| m.overflow) }
    }
}
