//! Opens a window drawing the step-1 scene (a triangle, a hairline frame, text), to check the window, the
//! resize handling and the idle loop by eye. Run: cargo run -p mere-win32 --example window

use mere_base::{App, Color, DrawList, Event, Gfx, Mat4, Material, MeshId, Rect, Vec2, Vertex, log, observe};

struct Sheet;

impl App for Sheet {
    fn start(&mut self, gfx: &mut dyn Gfx) {
        // Clip-space corners, so the identity matrix is the whole camera.
        let v = |x, y| Vertex { pos: [x, y, 0.5], uv: [0.0, 0.0] };
        gfx.upload_mesh(MeshId(1), &[v(-0.6, -0.6), v(0.0, 0.7), v(0.6, -0.6)], &[0, 1, 2]);
    }

    fn event(&mut self, event: Event) -> bool {
        observe!(Debug, "{event:?}");
        // Anything that changes what is on screen: a new size. Pointer moves change nothing here.
        matches!(event, Event::Resized { .. })
    }

    fn frame(&mut self, gfx: &mut dyn Gfx, _: f32) -> bool {
        let (w, h) = gfx.size_px();
        let s = gfx.scale();
        let (w, h) = (w as f32 / s, h as f32 / s);
        gfx.begin(Color::hex(0x1a2433));
        gfx.draw_mesh(Rect::new(0.0, 0.0, w, h), MeshId(1), &Mat4::IDENTITY, Material::Flat { color: Color::hex(0xe8a33d) });
        let mut list = DrawList::default();
        list.frame(Rect::new(20.0, 20.0, 260.0, 70.0), 1.0, Color::hex(0xececec));
        list.text(Vec2::new(32.0, 30.0), "Relief 94427382", 18.0, Color::hex(0xececec));
        list.text(Vec2::new(32.0, 56.0), &format!("{w:.0} x {h:.0} at scale {s}"), 12.0, Color::hex(0x8a8a8a));
        gfx.draw_list(&list);
        gfx.end();
        false
    }
}

fn main() -> Result<(), String> {
    log::add_sink(log::stderr_sink(log::Level::Debug));
    mere_win32::run("mere window example", &mut Sheet).map_err(|e| e.to_string())
}

