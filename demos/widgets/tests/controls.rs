//! The controls sheet drawn offscreen in both themes, and checked pixel by pixel.

use nui_base::{Color, Gfx};
use nui_ui::Theme;
use nui_ui::tokens::FADE_DISABLED;
use nui_win32::Renderer;
use widgets::{CONTROLS, controls_sheet};

fn draw(theme: Theme, name: &str) -> Vec<u8> {
    let list = controls_sheet(theme);
    let (w, h) = CONTROLS;
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
    let i = (y as usize * CONTROLS.0 as usize + x as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn same(got: [u8; 3], want: Color, what: &str) {
    let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
    assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what}: got {got:?}, want {w:?}");
}

fn check(theme: Theme, name: &str) {
    let rgba = draw(theme, name);
    let at = |x, y| px(&rgba, x, y);
    let off = |c: Color| theme.faded(c, FADE_DISABLED);
    same(at(5, 5), theme.ground, "ground");
    same(at(92, 40), theme.text, "pressed Flat interior");
    same(at(25, 40), theme.ground, "normal Biomes interior");
    same(at(20, 50), theme.edge, "Biomes left hairline");
    same(at(502, 40), theme.text, "pressed 2D interior");
    same(at(25, 100), theme.text, "coarse: pressed Biomes interior");
    same(at(330, 96), off(theme.edge), "coarse: disabled Structures top hairline");
    same(at(22, 158), theme.text, "Pack fill");
    same(at(565, 160), theme.text, "pressed 32 interior");
    same(at(480, 156), theme.edge, "normal 64 top hairline");
    same(at(700, 156), off(theme.edge), "disabled 16 top hairline");
    same(at(100, 234), theme.edge, "World rule");
    same(at(100, 240), theme.ground, "World head interior");
    same(at(600, 234), theme.edge, "Settings rule");
    same(at(740, 300), theme.edge, "first number field top hairline");
    assert!(rgba.chunks(4).any(|p| p[..3] != rgba[..3]), "the picture is one colour");
}

#[test]
fn controls_dark() {
    check(Theme::dark(), "controls_dark.png");
}

#[test]
fn controls_light() {
    check(Theme::light(), "controls_light.png");
}
