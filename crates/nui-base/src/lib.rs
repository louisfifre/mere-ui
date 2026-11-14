//! The platform contract. Everything above a backend speaks only these types and traits, so a backend for
//! another system implements `Gfx` and a `run` function, and nothing else changes.

pub mod camera;
pub mod draw;
pub mod log;
pub mod math;
pub mod process;

pub use camera::Orbit;
pub use draw::{Cmd, DrawList, Weight};
pub use math::{Mat4, Rect, Vec2, Vec3};
pub use process::Launcher;

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

/// What the system tells an application.
#[derive(Copy, Clone, Debug)]
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
}
