//! The pointer as the widgets see it: where it is, whether the left button is down, and the edges that
//! happened since the last frame.

use nui_base::{Button, Event, Vec2};

#[derive(Copy, Clone, Debug, Default)]
pub struct Input {
    /// None until the pointer has been seen.
    pub pos: Option<Vec2>,
    pub down: bool,
    /// The button went down since the last frame.
    pub pressed: bool,
    /// The button went up since the last frame. Both edges can be true in one frame (a fast click).
    pub released: bool,
}

impl Input {
    /// Stores what the event says; true when the screen must be drawn again. Only the left button counts.
    pub fn feed(&mut self, e: &Event) -> bool {
        match *e {
            Event::PointerMoved { pos } => {
                self.pos = Some(pos);
                true
            }
            Event::PointerDown { pos, button: Button::Left, .. } => {
                self.pos = Some(pos);
                self.down = true;
                self.pressed = true;
                true
            }
            Event::PointerUp { pos, button: Button::Left } => {
                self.pos = Some(pos);
                self.down = false;
                self.released = true;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn end_frame(&mut self) {
        self.pressed = false;
        self.released = false;
    }
}
