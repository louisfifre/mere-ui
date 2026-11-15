//! The launcher controls sheet drawn offscreen in both themes, and checked pixel by pixel.

use nui_base::{Color, Gfx};
use nui_ui::{Theme, mix};
use nui_win32::Renderer;
use widgets::{LAUNCHER, launcher_sheet};

fn draw(theme: Theme, name: &str) -> Vec<u8> {
    let list = launcher_sheet(theme);
    let (w, h) = LAUNCHER;
    let mut r = Renderer::offscreen(w, h, 1.0).expect("offscreen renderer");
    r.begin(theme.ground);
    r.draw_list(&list);
    r.end();
    let rgba = r.read_rgba().unwrap();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, nui_png::encode(w, h, &rgba)).unwrap();
    println!("wrote {}", path.display());
    rgba
}

fn px(rgba: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = (y as usize * LAUNCHER.0 as usize + x as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn same(got: [u8; 3], want: Color, what: &str) {
    let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
    assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what}: got {got:?}, want {w:?}");
}

fn check(theme: Theme, name: &str) {
    let rgba = draw(theme, name);
    let at = |x, y| px(&rgba, x, y);
    same(at(5, 5), theme.ground, "ground");
    // Cells start at (20, 30) and are 30 px squares: a top hairline, then the inside.
    same(at(30, 30), theme.edge, "Square rest: top hairline");
    same(at(80 + 15, 30), theme.text, "Square hover: top hairline");
    same(at(140 + 3, 30 + 3), theme.text, "Square expanded: fill");
    same(at(20 + 3, 90 + 3), theme.text, "Filled rest: fill");
    same(at(80 + 3, 90 + 3), mix(theme.text, 0.85, theme.ground), "Filled hover: fill");
    assert!(rgba.chunks(4).any(|p| p[..3] != rgba[..3]), "the picture is one colour");
}

#[test]
fn launcher_controls_dark() {
    check(Theme::dark(), "launcher_controls_dark.png");
}

#[test]
fn launcher_controls_light() {
    check(Theme::light(), "launcher_controls_light.png");
}
