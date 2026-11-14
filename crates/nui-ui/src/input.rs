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
    /// Where the left button went down this frame, which is where a click is judged (the pointer may move on
    /// before the frame is built).
    pub press_pos: Option<Vec2>,
    /// Presses in a row at one spot, as the backend counted them: 1, 2 or 3. Stored on a left press.
    pub clicks: u8,
    /// Wheel notches since the last frame, positive away from the user. A scroll area that uses them sets
    /// this back to 0, so only the innermost area under the pointer moves.
    pub wheel: f32,
    /// Where the pointer was at the last wheel event.
    pub wheel_pos: Option<Vec2>,
}

impl Input {
    /// Stores what the event says; true when the screen must be drawn again. Only the left button counts.
    pub fn feed(&mut self, e: &Event) -> bool {
        match *e {
            Event::PointerMoved { pos } => {
                self.pos = Some(pos);
                true
            }
            Event::PointerDown { pos, button: Button::Left, clicks } => {
                self.pos = Some(pos);
                self.press_pos = Some(pos);
                self.clicks = clicks;
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
            Event::Wheel { pos, delta } => {
                self.wheel += delta;
                self.wheel_pos = Some(pos);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn end_frame(&mut self) {
        self.pressed = false;
        self.released = false;
        self.press_pos = None;
        self.wheel = 0.0;
        self.wheel_pos = None;
    }
}
