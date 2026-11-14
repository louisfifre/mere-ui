//! What the picture tests share: drawing a list offscreen at scale 1 and writing it under `target/tmp`.

use std::path::PathBuf;

use nui_base::{Color, DrawList, Gfx};
use nui_win32::Renderer;

/// Draws `list` on a `w` x `h` image, writes it as `name` under the test temp dir and returns the path.
/// Fails if the image is all one colour: then nothing was drawn.
pub fn write_picture(list: &DrawList, w: u32, h: u32, name: &str) -> PathBuf {
    let mut r = Renderer::offscreen(w, h, 1.0).expect("offscreen renderer");
    r.begin(Color::hex(0x101214));
    r.draw_list(list);
    r.end();
    let rgba = r.read_rgba().unwrap();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, nui_png::encode(w, h, &rgba)).unwrap();
    println!("wrote {}", path.display());
    assert!(rgba.chunks(4).any(|p| p[..3] != rgba[..3]), "{name} is one colour");
    path
}
