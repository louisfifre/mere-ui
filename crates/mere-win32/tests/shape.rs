//! Filled shapes (Cmd::Shape) and a lost device, offscreen.

use mere_base::{Color, DrawList, Gfx, Rect, Vec2};
use mere_ui::Icon;
use mere_win32::Renderer;

/// Q3: after `mark_lost`, drawing and ending are quiet no-ops.
#[test]
fn q3_a_lost_renderer_draws_nothing_and_does_not_panic() {
    let mut r = Renderer::offscreen(64, 64, 1.0).expect("offscreen");
    assert!(!r.lost());
    r.mark_lost();
    assert!(r.lost());
    let mut l = DrawList::default();
    l.fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::hex(0xffffff));
    l.shape(Rect::new(0.0, 0.0, 16.0, 16.0), 16.0, vec![vec![Vec2::new(0.0, 0.0), Vec2::new(8.0, 0.0), Vec2::new(8.0, 8.0)]], Color::hex(0xffffff), true);
    r.begin(Color::hex(0));
    r.draw_list(&l);
    r.end();
    assert!(r.lost());
}
const GROUND: [u8; 3] = [0x00, 0x00, 0x00];
const INK: [u8; 3] = [0xec, 0xec, 0xec];
const HOME: &str = "M4 20h16v2H4zm16-10h2v10h-2zM2 10h2v10H2zm2-2h2v2H4zm2-2h2v2H6zm2-2h2v2H8zm2-2h4v2h-4zm4 2h2v2h-2zm2 2h2v2h-2zm2 2h2v2h-2zM8 14h2v6H8zm2-2h4v2h-4zm4 2h2v6h-2z";
const PACK: [&str; 3] = ["M16 2l14 7-14 7L2 9z", "M2 9l14 7v14L2 23z", "M30 9l-14 7v14l14-7z"];

/// The icon drawn into a 16-DIP box at (4, 4) of a 24-DIP canvas; returns the picture and its width.
fn draw_icon(d: &str, grid: f32, crisp: bool, scale: f32) -> (Vec<u8>, u32) {
    let icon = Icon::parse(d, grid).unwrap();
    let side = (24.0 * scale) as u32;
    let mut r = Renderer::offscreen(side, side, scale).expect("offscreen");
    let mut l = DrawList::default();
    l.shape(Rect::new(4.0, 4.0, 16.0, 16.0), grid, icon.figures, Color::hex(0xececec), crisp);
    r.begin(Color::hex(0));
    r.draw_list(&l);
    r.end();
    (r.read_rgba().unwrap(), side)
}

/// Q1: a pixel icon is fills of whole device pixels at every scale; at 1.5 a unit of the grid is one device
/// pixel, so the pixels coloured are exactly the area of the path.
#[test]
fn q1_a_crisp_pixel_icon_is_exactly_ground_or_ink() {
    for scale in [1.0f32, 1.5, 2.0] {
        let (rgba, side) = draw_icon(HOME, 24.0, true, scale);
        let mut inked = 0usize;
        for y in 0..side {
            for x in 0..side {
                let i = ((y * side + x) * 4) as usize;
                let px = [rgba[i], rgba[i + 1], rgba[i + 2]];
                assert!(px == GROUND || px == INK, "scale {scale}: ({x}, {y}) is {px:?}");
                inked += (px == INK) as usize;
            }
        }
        let area = Icon::parse(HOME, 24.0).unwrap().mask().unwrap().iter().filter(|c| **c).count();
        if scale == 1.5 {
            assert_eq!(inked, area, "one grid unit is one device pixel at 1.5");
        } else {
            assert!(inked > 0);
        }
    }
}

/// Q2: the pack's cube has slopes, which are antialiased: some pixel is neither ground nor ink.
#[test]
fn q2_a_sloped_icon_has_intermediate_values() {
    let mut all = Vec::new();
    for d in PACK {
        all.push(Icon::parse(d, 32.0).unwrap().figures.remove(0));
    }
    let side = 24u32;
    let mut r = Renderer::offscreen(side, side, 1.0).unwrap();
    let mut l = DrawList::default();
    l.shape(Rect::new(4.0, 4.0, 16.0, 16.0), 32.0, all, Color::hex(0xececec), false);
    r.begin(Color::hex(0));
    r.draw_list(&l);
    r.end();
    let rgba = r.read_rgba().unwrap();
    let between = rgba.chunks_exact(4).filter(|p| p[0] != 0 && p[0] != 0xec).count();
    assert!(between > 10, "{between} antialiased pixels");
}