//! W2: a Polyline is drawn as an open stroke of the asked width, offscreen.

use mere_base::{App, Color, DrawList, Event, Gfx, Vec2};

struct Lines;

impl App for Lines {
    fn start(&mut self, _: &mut dyn Gfx) {}
    fn event(&mut self, _: Event) -> bool {
        false
    }
    fn frame(&mut self, gfx: &mut dyn Gfx, _: f32) -> bool {
        gfx.begin(Color::hex(0x000000));
        let mut list = DrawList::default();
        let white = Color::hex(0xffffff);
        list.polyline(vec![Vec2::new(5.0, 20.0), Vec2::new(35.0, 20.0)], 2.0, white);
        list.polyline(vec![Vec2::new(10.5, 0.0), Vec2::new(10.5, 30.0)], 1.0, white);
        // One point draws nothing.
        list.polyline(vec![Vec2::new(30.0, 5.0)], 4.0, white);
        gfx.draw_list(&list);
        gfx.end();
        false
    }
}

#[test]
fn polyline_strokes_between_its_points() {
    let rgba = mere_win32::shot(&mut Lines, 40, 40, 1.0).expect("offscreen frame");
    let r = |x: usize, y: usize| rgba[(y * 40 + x) * 4];
    assert!(r(20, 20) > 200, "on the horizontal stroke: {}", r(20, 20));
    assert!(r(20, 10) < 10, "off the stroke: {}", r(20, 10));
    assert!(r(10, 15) > 200, "the 1 px vertical line centred on a pixel: {}", r(10, 15));
    assert!(r(30, 5) < 10, "a single point draws nothing");
}