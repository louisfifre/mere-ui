//! Step 1 check: clear colour, one flat triangle, a hairline frame and a line of text, offscreen, to a PNG.

use nui_base::{App, Color, DrawList, Event, Gfx, Mat4, Material, MeshId, Rect, Vec2, Vertex};

struct Sheet;

impl App for Sheet {
    fn start(&mut self, gfx: &mut dyn Gfx) {
        // Clip-space corners, so the identity matrix is the whole camera. Depth 0.5: reversed depth is
        // cleared to 0, and a nearer value is greater.
        let v = |x, y| Vertex { pos: [x, y, 0.5], uv: [0.0, 0.0] };
        gfx.upload_mesh(MeshId(1), &[v(-0.6, -0.6), v(0.0, 0.7), v(0.6, -0.6)], &[0, 1, 2]);
    }
    fn event(&mut self, _: Event) -> bool {
        false
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
        list.text(Vec2::new(32.0, 56.0), "hairline frame, legible text", 12.0, Color::hex(0x8a8a8a));
        gfx.draw_list(&list);
        gfx.end();
        false
    }
}

fn shot_at(width: u32, height: u32, scale: f32, name: &str) -> Vec<u8> {
    let rgba = nui_win32::shot(&mut Sheet, width, height, scale).expect("offscreen frame");
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, nui_png::encode(width, height, &rgba)).unwrap();
    println!("wrote {}", path.display());
    rgba
}

#[test]
fn offscreen_frame_shows_clear_triangle_frame_and_text() {
    let (w, h) = (480u32, 320u32);
    let rgba = shot_at(w, h, 1.0, "step1.png");
    let px = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    };
    assert_eq!(px(470, 310), [0x1a, 0x24, 0x33], "clear colour in a corner");
    assert_eq!(px(w / 2, h / 2 + 40), [0xe8, 0xa3, 0x3d], "triangle fill");
    assert_eq!(px(20, 50), [0xec, 0xec, 0xec], "hairline frame is one crisp pixel");
    assert_eq!(px(21, 50), [0x1a, 0x24, 0x33], "and no blur beside it");
    // Text: some pixel in its box differs from the ground.
    let inked = (32..200).any(|x| (30..50).any(|y| px(x, y) != [0x1a, 0x24, 0x33]));
    assert!(inked, "text drew something");
}

#[test]
fn offscreen_frame_at_scale_two() {
    shot_at(960, 640, 2.0, "step1_x2.png");
}
