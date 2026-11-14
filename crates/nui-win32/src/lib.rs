//! The Windows backend. One `Renderer` draws both meshes (Direct3D 11) and the interface (Direct2D and
//! DirectWrite) on the same picture: Direct2D is given the very texture Direct3D draws into, so the
//! interface lies over the 3D without a copy. The picture is either a window's swap chain (two images
//! taking turns on screen) or an offscreen texture read back for a PNG.

mod clipboard;
mod fonts;
mod process;
mod renderer;
mod scheme;
mod window;

pub use clipboard::SystemClipboard;
pub use process::SystemLauncher;
pub use renderer::Renderer;
pub use scheme::system_dark;
pub use window::{Frame, run, run_with};

/// Draws one frame of `app` offscreen, at `width` x `height` physical pixels and `scale` physical pixels
/// per device-independent pixel, and returns it as RGBA rows. The same code path as the window, minus the
/// swap chain, so a picture shows what the window would.
pub fn shot(app: &mut dyn nui_base::App, width: u32, height: u32, scale: f32) -> windows::core::Result<Vec<u8>> {
    let mut r = Renderer::offscreen(width, height, scale)?;
    app.start(&mut r);
    app.event(nui_base::Event::Resized { width_px: width, height_px: height, scale });
    app.frame(&mut r, 0.0);
    r.read_rgba()
}
