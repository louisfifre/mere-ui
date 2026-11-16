//! The interface font: IBM Plex Mono from the repository's files, Pi glyphs included, at scale 1 and 2.

use mere_base::{App, Color, DrawList, Event, Gfx, Vec2, log};
use mere_win32::Renderer;

const SAMPLE: &str = "Seed 0123456789 abcXYZ \u{2190} \u{2192} \u{2264} \u{2265} \u{2713}";
const GROUND: [u8; 3] = [0x1a, 0x24, 0x33];

struct Sample;

impl App for Sample {
    fn start(&mut self, _: &mut dyn Gfx) {}
    fn event(&mut self, _: Event) -> bool {
        false
    }
    fn frame(&mut self, gfx: &mut dyn Gfx, _: f32) -> bool {
        gfx.begin(Color::hex(0x1a2433));
        let mut list = DrawList::default();
        list.text(Vec2::new(16.0, 14.0), SAMPLE, 16.0, Color::hex(0xececec));
        list.text(Vec2::new(16.0, 44.0), SAMPLE, 12.0, Color::hex(0xececec));
        gfx.draw_list(&list);
        gfx.end();
        false
    }
}

fn picture(scale: f32, name: &str) -> (String, Vec<u8>, u32, u32) {
    let (w, h) = ((440.0 * scale) as u32, (72.0 * scale) as u32);
    let mut r = Renderer::offscreen(w, h, scale).expect("offscreen renderer");
    Sample.frame(&mut r, 0.0);
    let rgba = r.read_rgba().unwrap();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, mere_png::encode(w, h, &rgba)).unwrap();
    println!("wrote {}", path.display());
    (r.text_family().to_string(), rgba, w, h)
}

fn inked(rgba: &[u8], w: u32, x0: u32, x1: u32, y0: u32, y1: u32) -> bool {
    (x0..x1).any(|x| {
        (y0..y1).any(|y| {
            let i = ((y * w + x) * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2]] != GROUND
        })
    })
}

#[test]
fn text_is_drawn_in_ibm_plex_mono_with_pi_glyphs() {
    log::add_sink(log::stderr_sink(log::Level::Info));
    for (scale, name) in [(1.0, "fonts.png"), (2.0, "fonts_x2.png")] {
        let (family, rgba, w, _) = picture(scale, name);
        assert_eq!(family, "IBM Plex Mono", "the embedded files loaded, no fallback to Consolas");
        let s = scale as u32;
        assert!(inked(&rgba, w, 16 * s, 200 * s, 14 * s, 34 * s), "16 px line drew something");
        assert!(inked(&rgba, w, 16 * s, 200 * s, 44 * s, 60 * s), "12 px line drew something");
    }
}

#[test]
fn semibold_draws_more_ink_than_regular() {
    use mere_base::Weight;
    let mut r = Renderer::offscreen(400, 80, 1.0).expect("offscreen renderer");
    r.begin(Color::hex(0x000000));
    let mut list = DrawList::default();
    let white = Color::hex(0xececec);
    list.text_weighted(Vec2::new(10.0, 10.0), "Generate preview", 12.0, white, Weight::Regular);
    list.text_weighted(Vec2::new(10.0, 40.0), "Generate preview", 12.0, white, Weight::SemiBold);
    r.draw_list(&list);
    r.end();
    let rgba = r.read_rgba().unwrap();
    let sum = |y0: usize, y1: usize| -> u64 { (y0..y1).flat_map(|y| (0..400).map(move |x| (y, x))).map(|(y, x)| rgba[(y * 400 + x) * 4] as u64).sum() };
    let (regular, semibold) = (sum(8, 30), sum(38, 60));
    println!("ink regular {regular}, semibold {semibold}");
    assert!(semibold as f64 > 1.15 * regular as f64, "semibold {semibold} vs regular {regular}");
}
