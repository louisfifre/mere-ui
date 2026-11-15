//! The parts of a window drawn by the application: the grips along its edges, the caption area of its bar
//! and the icon buttons of its tool strip and window buttons (launcher lines 75-129).

use nui_base::{Cmd, Color, Cursor, Edge, Hit, Rect, Vec2};
use nui_layout::{Align, Dim, Justify, Margin, Position, Side, Style};

use crate::icon::{Icon, icon_shape};
use crate::id::Id;
use crate::text::{TextLook, Tone};
use crate::theme::mix;
use crate::ui::{Paint, Resp, Ui};

/// A tool button with its name (`Ui::tool_button`): 10 px before the icon, 7 between the icon and the word, 12 after.
pub const TOOL_PAD_L: f32 = 10.0;
pub const TOOL_GAP: f32 = 7.0;
pub const TOOL_PAD_R: f32 = 12.0;

/// How wide a tool button with this name is: what the bar measures before it builds.
pub fn tool_button_width(label: &str) -> f32 {
    TOOL_PAD_L + 16.0 + TOOL_GAP + TextLook::body(Tone::Text).width(label) + TOOL_PAD_R
}

/// `.grip` bands along the sides are 5 px, and 8 x 8 boxes at the corners (launcher lines 94-103).
pub const GRIP: f32 = 5.0;
pub const CORNER: f32 = 8.0;

/// Which edge of a `size` window the point falls on, if any. Corners first, then the four sides between them;
/// a maximised window has no edge to pull.
pub fn grip(pos: Vec2, size: Vec2, maximized: bool) -> Option<Edge> {
    if maximized || pos.x < 0.0 || pos.y < 0.0 || pos.x >= size.x || pos.y >= size.y {
        return None;
    }
    let left = pos.x < CORNER;
    let right = pos.x >= size.x - CORNER;
    let top = pos.y < CORNER;
    let bottom = pos.y >= size.y - CORNER;
    match (left, right, top, bottom) {
        (true, _, true, _) => return Some(Edge::NW),
        (_, true, true, _) => return Some(Edge::NE),
        (true, _, _, true) => return Some(Edge::SW),
        (_, true, _, true) => return Some(Edge::SE),
        _ => {}
    }
    // Between the corners: the bands are 5 px, and run from 8 to the far side less 8.
    if pos.y < GRIP {
        Some(Edge::N)
    } else if pos.y >= size.y - GRIP {
        Some(Edge::S)
    } else if pos.x < GRIP {
        Some(Edge::W)
    } else if pos.x >= size.x - GRIP {
        Some(Edge::E)
    } else {
        None
    }
}

/// What a press at `pos` does, for a window that draws its own bar. The grips are `position: fixed` over
/// everything (z-index 10), so they come first, buttons included; then the bar that is not a button drags
/// the window; the rest is the page's.
pub fn chrome_hit(ui: &Ui, pos: Vec2, size: Vec2, maximized: bool, bar: Id, buttons: &[Id]) -> Hit {
    if let Some(edge) = grip(pos, size, maximized) {
        return Hit::Resize(edge);
    }
    let in_bar = ui.rect(bar).is_some_and(|r| r.contains(pos));
    if in_bar && !buttons.iter().any(|b| ui.rect(*b).is_some_and(|r| r.contains(pos))) {
        Hit::Caption
    } else {
        Hit::Client
    }
}

/// The icon buttons of the bars and of the launcher's Home.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IconLook {
    /// `.strip button` with an icon (lines 110-112, 122): 36 x 28, framed, drawn in reverse when pressed.
    Tool,
    /// `.ends button` (lines 89-91): 46 wide and as tall as the bar, no frame.
    Caption,
    /// `.ends #close`: red under the pointer.
    Close,
    /// The launcher's `.icon` (index.html lines 160-165): 30 x 30, framed in --edge, a muted 16 px icon; under
    /// the pointer the icon and the frame take --text; `pressed` (aria-expanded) is drawn in reverse.
    Square,
    /// `.icon.play` (lines 180-181): filled with --text, the icon in --ground; 0.85 opacity under the pointer.
    Filled,
    /// `.icon.back` (line 166): no frame, 24 wide, 30 tall, pulled 4 px left.
    Bare,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct IconOpts {
    pub pressed: bool,
    pub disabled: bool,
    /// `.icon + .icon` (line 162): pulled 1 px over the one before, so the two share a hairline.
    pub joined: bool,
    /// A 5 x 5 square on the button's top right corner, 4 px in (the launcher's `.marker`, line 129), in this
    /// colour: where a run of the preview stands while it is not on screen.
    pub marker: Option<nui_base::Color>,
}

impl Ui {
    /// An icon button. `Resp.clicked` is the click; the app decides what it does and which one is pressed.
    pub fn icon_button(&mut self, key: &str, icon: &Icon, look: IconLook, opts: IconOpts) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, opts.disabled, Cursor::Hand);
        let t = *self.theme();
        let lit = !opts.disabled && (resp.hovered || resp.held);
        let (style, fill, frame, color) = match look {
            IconLook::Tool => {
                let style = Style {
                    w: Dim::Px(36.0),
                    h: Dim::Px(28.0),
                    shrink: 0.0,
                    margin: Margin { left: Side::Px(-1.0), ..Margin::default() },
                    align_items: Align::Center,
                    justify: Justify::Center,
                    // As the strip's tabs: the pressed one is painted over its neighbours' hairlines.
                    position: if opts.pressed { Position::Relative } else { Position::Static },
                    ..Style::default()
                };
                if opts.pressed {
                    (style, Some(t.text), Some(t.text), t.ground)
                } else {
                    (style, None, Some(t.edge), if lit { t.text } else { t.muted })
                }
            }
            IconLook::Square | IconLook::Filled | IconLook::Bare => {
                let (w, left) = match look {
                    IconLook::Bare => (24.0, -4.0),
                    _ => (30.0, if opts.joined { -1.0 } else { 0.0 }),
                };
                let style = Style {
                    w: Dim::Px(w),
                    h: Dim::Px(30.0),
                    shrink: 0.0,
                    margin: Margin { left: Side::Px(left), ..Margin::default() },
                    align_items: Align::Center,
                    justify: Justify::Center,
                    ..Style::default()
                };
                match look {
                    IconLook::Bare => (style, None, None, if lit { t.text } else { t.muted }),
                    IconLook::Filled if lit => {
                        let c = mix(t.text, 0.85, t.ground);
                        (style, Some(c), Some(c), t.ground)
                    }
                    IconLook::Filled => (style, Some(t.text), Some(t.text), t.ground),
                    // aria-expanded keeps its reversed look on hover (line 165).
                    _ if opts.pressed => (style, Some(t.text), Some(t.text), t.ground),
                    _ if lit => (style, None, Some(t.text), t.text),
                    _ => (style, None, Some(t.edge), t.muted),
                }
            }
            IconLook::Caption | IconLook::Close => {
                let style = Style {
                    w: Dim::Px(46.0),
                    shrink: 0.0,
                    align_items: Align::Center,
                    justify: Justify::Center,
                    ..Style::default()
                };
                match (lit, look) {
                    (true, IconLook::Close) => (style, Some(Color::hex(0xc42b1c)), None, Color::hex(0xffffff)),
                    (true, _) => (style, Some(mix(t.text, 0.10, t.ground)), None, t.text),
                    (false, _) => (style, None, None, t.muted),
                }
            }
        };
        // The disabled look is the page's `button:disabled { opacity: 0.4 }`.
        let (fill, frame, color) = if opts.disabled {
            (fill.map(|c| t.faded(c, 0.4)), frame.map(|c| t.faded(c, 0.4)), t.faded(color, 0.4))
        } else {
            (fill, frame, color)
        };
        self.open(key, id, style, Paint::Box { fill, frame });
        let slot = Style { w: Dim::Px(16.0), h: Dim::Px(16.0), shrink: 0.0, ..Style::default() };
        let mut cmds = vec![icon_shape(icon, 16.0, color, icon.axis_aligned())];
        if let (Some(m), IconLook::Tool) = (opts.marker, look) {
            // Positioned against the button: 4 px from its top and right edges. The icon sits at (10, 6) in it.
            cmds.push(Cmd::Fill { rect: Rect::new(36.0 - 4.0 - 5.0 - 10.0, 4.0 - 6.0, 5.0, 5.0), color: m });
        }
        self.add("icon", slot, Paint::Cmds(cmds));
        self.close();
        resp
    }

    /// A tool of the bar as an icon and its name: the look of `IconLook::Tool` (framed, reversed when pressed),
    /// as wide as its word. The marker sits on the icon's top right corner, where the icon-only button has it.
    pub fn tool_button(&mut self, key: &str, icon: &Icon, label: &str, opts: IconOpts) -> Resp {
        let id = self.key(key);
        let resp = self.interact(id, false, Cursor::Hand);
        let t = *self.theme();
        let lit = resp.hovered || resp.held;
        let style = Style {
            h: Dim::Px(28.0),
            pad: nui_layout::Edges::new(0.0, TOOL_PAD_R, 0.0, TOOL_PAD_L),
            gap: Vec2::new(TOOL_GAP, 0.0),
            shrink: 0.0,
            margin: Margin { left: Side::Px(-1.0), ..Margin::default() },
            align_items: Align::Center,
            position: if opts.pressed { Position::Relative } else { Position::Static },
            ..Style::default()
        };
        let (fill, frame, color) = if opts.pressed { (Some(t.text), Some(t.text), t.ground) } else { (None, Some(t.edge), if lit { t.text } else { t.muted }) };
        self.open(key, id, style, Paint::Box { fill, frame });
        let slot = Style { w: Dim::Px(16.0), h: Dim::Px(16.0), shrink: 0.0, ..Style::default() };
        let mut cmds = vec![icon_shape(icon, 16.0, color, icon.axis_aligned())];
        if let Some(m) = opts.marker {
            cmds.push(Cmd::Fill { rect: Rect::new(16.0 + 1.0, -2.0, 5.0, 5.0), color: m });
        }
        self.add("icon", slot, Paint::Cmds(cmds));
        self.add_text(label, TextLook::body(Tone::Text), color, false, Style::default());
        self.close();
        resp
    }
}
