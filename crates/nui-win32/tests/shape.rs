//! Filled shapes (Cmd::Shape) and a lost device, offscreen.

use nui_base::{Color, DrawList, Gfx, Rect, Vec2};
use nui_win32::Renderer;

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