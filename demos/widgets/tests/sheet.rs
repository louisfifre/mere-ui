//! The sheet of states drawn offscreen in both themes, and checked pixel by pixel.

use nui_base::{Color, Gfx};
use nui_ui::Theme;
use nui_win32::Renderer;
use widgets::{COLS, SHEET, sheet};

fn draw(theme: Theme, name: &str) -> Vec<u8> {
    let list = sheet(theme);
    let (w, h) = SHEET;
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

fn px(rgba: &[u8], x: f32, y: f32) -> [u8; 3] {
    let i = (y as usize * SHEET.0 as usize + x as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn same(got: [u8; 3], want: Color, what: &str) {
    let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
    assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what}: got {got:?}, want {w:?}");
}

fn check(theme: Theme, name: &str) {
    let rgba = draw(theme, name);
    same(px(&rgba, 5.0, 5.0), theme.ground, "ground");
    same(px(&rgba, COLS[0] + 3.0, 61.0), theme.text, "primary normal");
    same(px(&rgba, COLS[1] + 3.0, 61.0), theme.faded(theme.text, 0.85), "primary hover");
    // Held is the hover look: the two crops are the same picture.
    let (cw, ch) = (180usize, 34usize);
    for y in 0..ch {
        for x in 0..cw {
            let a = px(&rgba, COLS[1] + x as f32, 44.0 + y as f32);
            let b = px(&rgba, COLS[2] + x as f32, 44.0 + y as f32);
            assert!(a.iter().zip(b).all(|(p, q)| (*p as i32 - q as i32).abs() <= 2), "held differs from hover at {x},{y}");
        }
    }
    same(px(&rgba, COLS[3] + 3.0, 61.0), theme.ground, "primary disabled inside");
    same(px(&rgba, COLS[3], 61.0), theme.edge, "primary disabled frame");
    same(px(&rgba, COLS[0] + 140.0, 168.0), theme.edge, "toggle frame");
    same(px(&rgba, COLS[1] + 140.0, 168.0), theme.text, "toggle frame on hover");
    assert!(rgba.chunks(4).any(|p| p[..3] != rgba[..3]), "the picture is one colour");
}

#[test]
fn sheet_dark() {
    check(Theme::dark(), "widgets_dark.png");
}

#[test]
fn sheet_light() {
    check(Theme::light(), "widgets_light.png");
}
