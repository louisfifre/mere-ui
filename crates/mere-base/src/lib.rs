//! The platform contract. Everything above a backend speaks only these types and traits, so a backend for
//! another system implements `Gfx` and a `run` function, and nothing else changes.

pub mod camera;
pub mod draw;
pub mod log;
pub mod math;
pub mod process;

pub use camera::Orbit;
pub use draw::{Cmd, DrawList, Weight};
pub use math::{Mat4, Rect, Vec2, Vec3};pub use process::Launcher;

/// An sRGB colour, each channel 0..1, alpha straight (not premultiplied).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgb8(r: u8, g: u8, b: u8) -> Color {
        Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
    }
    /// `0xRRGGBB`, the way the page's CSS writes its colours.
    pub const fn hex(rgb: u32) -> Color {
        Color::rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }
    pub const fn with_alpha(self, a: f32) -> Color {
        Color { a, ..self }
    }
}

/// A point of a mesh. The texture coordinate is ignored by a material without an image.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageId(pub u32);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MeshId(pub u32);

/// How a mesh is drawn. Two kinds cover what the applications need: an opaque surface showing an image
/// (the land), and a flat translucent sheet (the water), which is tested against depth but does not hide
/// what is drawn after it.
#[derive(Copy, Clone, Debug)]
pub enum Material {
    Image { image: ImageId, nearest: bool },
    Flat { color: Color },
}

/// What a backend draws with. Meshes and images are uploaded once and drawn by id, so a frame sends only
/// a camera and a draw list. Sizes are in device-independent pixels (96 a inch); the backend scales them.
pub trait Gfx {
    /// Physical pixels of the target.
    fn size_px(&self) -> (u32, u32);
    /// Physical pixels per device-independent pixel: 1 at 96 dpi, 1.5 at 144.
    fn scale(&self) -> f32;
    fn upload_image(&mut self, id: ImageId, width: u32, height: u32, rgba: &[u8]);
    fn upload_mesh(&mut self, id: MeshId, vertices: &[Vertex], indices: &[u32]);
    /// Starts a frame: the whole target cleared to `ground`, depth cleared.
    fn begin(&mut self, ground: Color);
    /// Draws a mesh inside `viewport` (device-independent pixels), seen through `view_proj`.
    fn draw_mesh(&mut self, viewport: Rect, mesh: MeshId, view_proj: &Mat4, material: Material);
    /// Draws interface commands over whatever was drawn before, in order.
    fn draw_list(&mut self, list: &DrawList);
    /// Ends the frame: shows it in a window, or keeps it for `read_rgba` offscreen.
    fn end(&mut self);
}

/// What the system's input method (IME, the tool that turns keystrokes into characters a keyboard does not
/// have: kana, kanji, pinyin) does to the focused text field. Each variant is one message of the system, so each
/// can be tested alone.
#[derive(Clone, Debug, PartialEq)]
pub enum Ime {
    /// A composition (preedit: the text the input method is still building) begins.
    Start,
    /// Replaces the whole composition. An empty 	ext means nothing is shown. cursor is in chars into 	ext.
    Update { text: String, cursor: usize },
    /// The finished text, which the field inserts as part of its value.
    Commit { text: String },
    /// Closes the composition, whether it was committed or cancelled.
    End,
}

/// What the system tells an application.
#[derive(Clone, Debug)]
pub enum Event {
    Resized { width_px: u32, height_px: u32, scale: f32 },
    PointerMoved { pos: Vec2 },
    /// `clicks` counts presses in a row at one spot: 1, 2, 3 (capped) for the left button, always 1 for the others.
    PointerDown { pos: Vec2, button: Button, clicks: u8 },
    PointerUp { pos: Vec2, button: Button },
    Wheel { pos: Vec2, delta: f32 },
    /// A key the interface acts on, pressed (auto-repeat sends it again).
    Key { key: Key, mods: Mods },
    /// A typed character, never a control character.
    Char { ch: char },
    CloseRequested,
    /// The system's light or dark setting changed, as the page's prefers-color-scheme.
    Scheme { dark: bool },
    /// The window's state: sent at start and whenever it changes.
    Window { maximized: bool },
    /// The pointer left the window, so nothing is hovered any more.
    PointerLeft,
    /// The input method's work on the focused text field; only sent while App::ime_spot is Some.
    Ime(Ime),
}

/// Where a point of a window with its own frame falls: what the system does with a press there.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Hit {
    Client,
    Caption,
    Resize(Edge),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Edge {
    N,
    S,
    E,
    W,
    NW,
    NE,
    SW,
    SE,
}

/// The pointer's shapes the pages use (CSS cursor). Grab stands for grab and grabbing.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Arrow,
    Hand,
    IBeam,
    Crosshair,
    Grab,
    ResizeNS,
    ResizeEW,
    ResizeNWSE,
    ResizeNESW,
}

impl Hit {
    /// The shape the system shows there: the sizing arrows on an edge, the arrow elsewhere.
    pub fn cursor(self) -> Cursor {
        match self {
            Hit::Client | Hit::Caption => Cursor::Arrow,
            Hit::Resize(Edge::N | Edge::S) => Cursor::ResizeNS,
            Hit::Resize(Edge::E | Edge::W) => Cursor::ResizeEW,
            Hit::Resize(Edge::NW | Edge::SE) => Cursor::ResizeNWSE,
            Hit::Resize(Edge::NE | Edge::SW) => Cursor::ResizeNESW,
        }
    }
}

/// What an application asks of its window, taken once.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowRequest {
    Minimize,
    ToggleMaximize,
    Close,
    Title(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
}

/// The keys the interface acts on; a backend sends no others. The letters exist only as Ctrl shortcuts: a
/// backend sends them only with Ctrl held and Alt up, so typing a letter arrives as Event::Char.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Backspace,
    Delete,
    Enter,
    Escape,
    Tab,
    A,
    C,
    V,
    X,
    Z,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub ctrl: bool,
    pub shift: bool,
}

/// The system clipboard as text. A backend implements it; tests use a memory one.
pub trait Clipboard {
    fn get(&mut self) -> Option<String>;
    fn set(&mut self, text: &str);
}

/// An application, driven by a backend. Frames are drawn on demand: after an event that asked for one, or
/// while the previous frame said it is still moving.
pub trait App {
    /// Takes an event; true when the screen must be drawn again.
    fn event(&mut self, event: Event) -> bool;
    /// Draws one frame; true while something still moves (a camera gliding to rest), to get another.
    fn frame(&mut self, gfx: &mut dyn Gfx, seconds: f32) -> bool;
    /// Runs once, after the backend is ready and before the first frame: uploads meshes and images.
    fn start(&mut self, gfx: &mut dyn Gfx);
    /// What a press at `pos` (DIP) does in a window with its own frame. Only asked there.
    fn hit(&self, _pos: Vec2) -> Hit {
        Hit::Client
    }
    /// The pointer's shape over the client area, as of the last frame.
    fn cursor(&self) -> Cursor {
        Cursor::Arrow
    }
    /// What the application asks of its window; the backend asks again until it gets None.
    fn take_request(&mut self) -> Option<WindowRequest> {
        None
    }
    /// Seconds until a frame is wanted with no event (a tooltip's delay, a progress read); None for never.
    fn wake_in(&self) -> Option<f32> {
        None
    }
    /// Work with no drawing, called while the window is hidden (minimised or covered); true when a frame
    /// is then wanted.
    fn tick(&mut self) -> bool {
        false
    }
    /// Where the input method may write, as of the last frame: the caret of a focused text field, in DIP
    /// (x, top, 1, glyph height). None turns the input method off.
    fn ime_spot(&self) -> Option<Rect> {
        None
    }
}
