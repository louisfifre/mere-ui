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

const GROUND: [u8; 3] = [0x1a, 0x24, 0x33];

/// An application drawing whatever `draw` says over one triangle, to test one thing at a time.
struct Scene {
    triangle: [[f32; 2]; 3],
    draw: fn(&mut dyn Gfx, f32, f32),
}

impl App for Scene {
    fn start(&mut self, gfx: &mut dyn Gfx) {
        let [a, b, c] = self.triangle.map(|[x, y]| Vertex { pos: [x, y, 0.5], uv: [0.0, 0.0] });
        gfx.upload_mesh(MeshId(1), &[a, b, c], &[0, 1, 2]);
    }
    fn event(&mut self, _: Event) -> bool {
        false
    }
    fn frame(&mut self, gfx: &mut dyn Gfx, _: f32) -> bool {
        let (w, h) = gfx.size_px();
        let s = gfx.scale();
        gfx.begin(Color::hex(0x1a2433));
        (self.draw)(gfx, w as f32 / s, h as f32 / s);
        gfx.end();
        false
    }
}

fn scene_shot(scene: &mut Scene, width: u32, height: u32, scale: f32, name: &str) -> Vec<u8> {
    let rgba = nui_win32::shot(scene, width, height, scale).expect("offscreen frame");
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, nui_png::encode(width, height, &rgba)).unwrap();
    println!("wrote {}", path.display());
    rgba
}

fn flat_triangle(gfx: &mut dyn Gfx, w: f32, h: f32) {
    gfx.draw_mesh(Rect::new(0.0, 0.0, w, h), MeshId(1), &Mat4::IDENTITY, Material::Flat { color: Color::hex(0xe8a33d) });
}

#[test]
fn a_slanted_edge_has_intermediate_pixels() {
    let (w, h) = (320u32, 240u32);
    // The right side climbs gently, so each row crosses it at a fractional pixel: without multisampling
    // every pixel is either the fill or the ground, with it the crossing pixels are blends.
    let mut scene = Scene { triangle: [[-0.7, -0.7], [0.55, 0.8], [0.7, -0.7]], draw: |g, w, h| flat_triangle(g, w, h) };
    let rgba = scene_shot(&mut scene, w, h, 1.0, "msaa_edge.png");
    let fill = [0xe8, 0xa3, 0x3d];
    let blended = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|&(x, y)| {
        let i = ((y * w + x) * 4) as usize;
        let p = [rgba[i], rgba[i + 1], rgba[i + 2]];
        // Strictly between ground and fill on the red channel, and not a colour of either.
        p != GROUND && p != fill && p[0] > GROUND[0] && p[0] < fill[0]
    });
    assert!(blended.count() > 50, "an antialiased edge leaves many blended pixels");
}

#[test]
fn a_hairline_is_one_logical_pixel_wide() {
    let ground = GROUND;
    let line = [0xec, 0xec, 0xec];
    for (scale, device_px) in [(1.0f32, 1u32), (2.0, 2), (3.0, 3)] {
        let (w, h) = ((480.0 * scale) as u32, (320.0 * scale) as u32);
        let mut scene = Scene {
            triangle: [[0.0; 2]; 3],
            draw: |g, _, _| {
                let mut list = DrawList::default();
                list.frame(Rect::new(20.0, 20.0, 260.0, 70.0), 1.0, Color::hex(0xececec));
                g.draw_list(&list);
            },
        };
        let rgba = scene_shot(&mut scene, w, h, scale, &format!("hairline_x{scale}.png"));
        let px = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2]]
        };
        let (left, mid_y) = ((20.0 * scale) as u32, (50.0 * scale) as u32);
        assert_eq!(px(left - 1, mid_y), ground, "scale {scale}: nothing outside the line");
        for k in 0..device_px {
            assert_eq!(px(left + k, mid_y), line, "scale {scale}: device pixel {k} of the line");
        }
        assert_eq!(px(left + device_px, mid_y), ground, "scale {scale}: the line is {device_px} device px wide, no more");
    }
}

#[test]
fn a_short_rgba_is_refused() {
    let mut r = nui_win32::Renderer::offscreen(64, 64, 1.0).expect("renderer");
    // A mesh covering the target and textured by image 7, which is only uploaded when the data is whole.
    let v = |x, y| Vertex { pos: [x, y, 0.5], uv: [0.5, 0.5] };
    r.upload_mesh(MeshId(1), &[v(-1.0, -1.0), v(-1.0, 3.0), v(3.0, -1.0)], &[0, 1, 2]);
    let draw = |r: &mut nui_win32::Renderer| {
        r.begin(Color::hex(0x1a2433));
        r.draw_mesh(Rect::new(0.0, 0.0, 64.0, 64.0), MeshId(1), &Mat4::IDENTITY, Material::Image { image: nui_base::ImageId(7), nearest: true });
        r.end();
        let rgba = r.read_rgba().unwrap();
        [rgba[(32 * 64 + 32) * 4], rgba[(32 * 64 + 32) * 4 + 1], rgba[(32 * 64 + 32) * 4 + 2]]
    };
    r.upload_image(nui_base::ImageId(7), 2, 2, &[255, 0, 0, 255]); // 4 bytes for 16 expected
    assert_eq!(draw(&mut r), GROUND, "the short image was not stored");
    r.upload_image(nui_base::ImageId(7), 2, 2, &[255, 0, 0, 255].repeat(4));
    assert_eq!(draw(&mut r), [255, 0, 0], "a whole image works");
}

#[test]
fn no_seam_at_the_middle_column() {
    for (w, h) in [(480u32, 320u32), (481, 321)] {
        let mut scene = Scene {
            triangle: [[-1.5, -1.5], [0.0, 2.0], [1.5, -1.5]],
            draw: |g, w, h| {
                g.draw_mesh(Rect::new(0.0, 0.0, w, h), MeshId(1), &Mat4::IDENTITY, Material::Flat { color: Color::hex(0xe8a33d).with_alpha(0.5) });
                let mut list = DrawList::default();
                list.fill(Rect::new(40.0, 200.0, 400.0, 80.0), Color::hex(0x000000).with_alpha(0.5));
                g.draw_list(&list);
            },
        };
        let rgba = scene_shot(&mut scene, w, h, 1.0, &format!("seam_{w}.png"));
        let at = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2]]
        };
        for y in [60, 150, 240] {
            let (a, b, c) = (at(w / 2 - 2, y), at(w / 2 - 1, y), at(w / 2, y));
            println!("{w}x{h} y={y}: {a:?} {b:?} {c:?}");
            // The scene is uniform across a row inside the triangle (a plate or a plain fill).
            assert_eq!(a, b, "{w}x{h} y={y}: columns {} and {}", w / 2 - 2, w / 2 - 1);
            assert_eq!(b, c, "{w}x{h} y={y}: columns {} and {}", w / 2 - 1, w / 2);
        }
    }
}

#[test]
fn a_uniform_texture_on_a_big_triangle_has_no_seam_either() {
    // The relief's seam at x = width/2 was a texel edge of the map (the grid's middle falls exactly between
    // two samples, and the camera looks along it), not the renderer. A texture of one colour proves the
    // renderer adds none: a triangle covering the whole target, nearest-sampled, must be uniform.
    let (w, h) = (480u32, 320u32);
    let mut r = nui_win32::Renderer::offscreen(w, h, 1.0).expect("renderer");
    let v = |x: f32, y: f32, u: f32, t: f32| Vertex { pos: [x, y, 0.5], uv: [u, t] };
    r.upload_mesh(MeshId(1), &[v(-3.0, -3.0, 0.0, 1.0), v(-3.0, 9.0, 0.0, -3.0), v(9.0, -3.0, 4.0, 1.0)], &[0, 1, 2]);
    r.upload_image(nui_base::ImageId(1), 4, 4, &[0x80, 0x40, 0x20, 0xff].repeat(16));
    r.begin(Color::hex(0x1a2433));
    r.draw_mesh(Rect::new(0.0, 0.0, w as f32, h as f32), MeshId(1), &Mat4::IDENTITY, Material::Image { image: nui_base::ImageId(1), nearest: true });
    r.end();
    let rgba = r.read_rgba().unwrap();
    let first = [rgba[0], rgba[1], rgba[2]];
    assert_eq!(first, [0x80, 0x40, 0x20]);
    for (i, px) in rgba.chunks_exact(4).enumerate() {
        assert_eq!([px[0], px[1], px[2]], first, "pixel ({}, {})", i % w as usize, i / w as usize);
    }
}

#[test]
fn a_clip_cuts_what_is_drawn_inside_it() {
    let (w, h) = (100u32, 100u32);
    let mut scene = Scene {
        triangle: [[0.0; 2]; 3],
        draw: |g, _, _| {
            let mut list = DrawList::default();
            list.push_clip(Rect::new(20.0, 20.0, 40.0, 40.0));
            list.fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::hex(0xff0000));
            list.pop_clip();
            // After the pop the clip is gone: this strip is drawn whole.
            list.fill(Rect::new(0.0, 90.0, 100.0, 10.0), Color::hex(0x0000ff));
            g.draw_list(&list);
        },
    };
    let rgba = scene_shot(&mut scene, w, h, 1.0, "clip.png");
    let px = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    };
    assert_eq!(px(10, 10), GROUND, "outside the clip");
    assert_eq!(px(30, 30), [0xff, 0, 0], "inside the clip");
    assert_eq!(px(70, 70), GROUND, "outside the clip, past its far corner");
    assert_eq!(px(50, 95), [0, 0, 0xff], "after the pop");
}

#[test]
fn a_clip_never_popped_still_draws() {
    // Direct2D refuses EndDraw with a clip open: the renderer closes it, and the frame is drawn.
    let (w, h) = (100u32, 100u32);
    let mut scene = Scene {
        triangle: [[0.0; 2]; 3],
        draw: |g, _, _| {
            let mut list = DrawList::default();
            list.push_clip(Rect::new(20.0, 20.0, 40.0, 40.0));
            list.fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::hex(0xff0000));
            g.draw_list(&list);
        },
    };
    let rgba = scene_shot(&mut scene, w, h, 1.0, "clip_open.png");
    let px = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    };
    assert_eq!(px(30, 30), [0xff, 0, 0]);
    assert_eq!(px(70, 70), GROUND);
}

#[test]
fn image_nearest_blocks() {
    use nui_base::ImageId;
    let (w, h) = (64u32, 64u32);
    // 16 distinct colours, row-major, opaque.
    let texel = |i: usize| [(i * 15 + 10) as u8, (255 - i * 13) as u8, ((i * 37) % 256) as u8];
    let mut scene = Scene {
        triangle: [[0.0; 2]; 3],
        draw: |g, _, _| {
            let mut rgba = Vec::new();
            for i in 0..16 {
                let t = [(i * 15 + 10) as u8, (255 - i * 13) as u8, ((i * 37) % 256) as u8];
                rgba.extend_from_slice(&[t[0], t[1], t[2], 255]);
            }
            g.upload_image(ImageId(5), 4, 4, &rgba);
            let mut list = DrawList::default();
            list.image(ImageId(5), Rect::new(10.0, 10.0, 32.0, 32.0));
            list.disc(Vec2::new(52.0, 52.0), 6.0, Color::hex(0xff0000));
            g.draw_list(&list);
        },
    };
    let rgba = scene_shot(&mut scene, w, h, 1.0, "image_nearest.png");
    let px = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    };
    let mut checked = 0;
    for y in 10..42u32 {
        for x in 10..42u32 {
            // Pixel centre in texel space; skip pixels whose centre lies within 1 px of a block edge.
            let (cx, cy) = (x as f32 + 0.5 - 10.0, y as f32 + 0.5 - 10.0);
            let near = |c: f32| {
                let m = c % 8.0;
                m < 1.0 || m > 7.0
            };
            if near(cx) || near(cy) {
                continue;
            }
            let (tx, ty) = ((cx / 8.0) as usize, (cy / 8.0) as usize);
            assert_eq!(px(x, y), texel(ty * 4 + tx), "pixel {x},{y}");
            checked += 1;
        }
    }
    assert!(checked > 400, "checked {checked} interior pixels");
    assert_eq!(px(52, 52), [255, 0, 0], "disc centre is red");
    assert_eq!(px(52 + 10, 52), GROUND, "ten pixels from the disc is ground");
}
fn fill_scene(rows: fn(&mut dyn Gfx, f32, f32), w: u32, h: u32, scale: f32) -> Vec<u8> {
    let mut scene = Scene { triangle: [[0.0; 2]; 3], draw: rows };
    nui_win32::shot(&mut scene, w, h, scale).expect("offscreen frame")
}

/// F1: a rule one DIP thick is whole device rows at any scale, exactly the colour, nothing grey beside it.
#[test]
fn f1_a_one_dip_fill_is_whole_device_rows() {
    let rule: fn(&mut dyn Gfx, f32, f32) = |gfx, _, _| {
        let mut l = DrawList::default();
        l.fill(Rect::new(0.0, 39.0, 40.0, 1.0), Color::hex(0xececec));
        gfx.draw_list(&l);
    };
    for (scale, rows) in [(1.0f32, 1u32), (1.5, 2), (2.0, 2)] {
        let (w, h) = ((40.0 * scale) as u32, (60.0 * scale) as u32);
        let rgba = fill_scene(rule, w, h, scale);
        let px = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2]]
        };
        let first = (39.0 * scale).round() as u32;
        for y in 0..h {
            for x in 0..w {
                let want = if (first..first + rows).contains(&y) { [0xec; 3] } else { GROUND };
                assert_eq!(px(x, y), want, "scale {scale}, pixel ({x}, {y})");
            }
        }
    }
}

/// F2: a box with fractional edges covers whole pixels: columns 10..30 at scale 1, no partial pixel.
#[test]
fn f2_a_fractional_fill_covers_whole_pixels() {
    let boxed: fn(&mut dyn Gfx, f32, f32) = |gfx, _, _| {
        let mut l = DrawList::default();
        l.fill(Rect::new(10.4, 5.0, 20.3, 10.0), Color::hex(0xececec));
        gfx.draw_list(&l);
    };
    let (w, h) = (50u32, 20u32);
    let rgba = fill_scene(boxed, w, h, 1.0);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let want = if (10..31).contains(&x) && (5..15).contains(&y) { [0xec; 3] } else { GROUND };
            // 10.4 rounds to 10 and 30.7 rounds to 31: columns 10..31.
            assert_eq!([rgba[i], rgba[i + 1], rgba[i + 2]], want, "pixel ({x}, {y})");
        }
    }
}
