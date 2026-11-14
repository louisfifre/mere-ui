//! The shell drawn offscreen at the page's three widths and in both themes, and checked pixel by pixel. The
//! pictures are written to the target's tmp folder and copied to docs/shots by hand: they are a milestone.

use nui_base::{Color, Gfx, Rect, Vec2};
use nui_ui::{GUTTER, Theme};
use nui_win32::Renderer;
use preview::{ShellIds, picture};

const H: u32 = 940;

fn draw(theme: Theme, w: u32, name: &str) -> (Vec<u8>, nui_ui::Ui, ShellIds) {
    let (list, ui, ids) = picture(theme, Vec2::new(w as f32, H as f32));
    let mut r = Renderer::offscreen(w, H, 1.0).expect("offscreen renderer");
    r.begin(theme.ground);
    r.draw_list(&list);
    r.end();
    let rgba = r.read_rgba().unwrap();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, nui_png::encode(w, H, &rgba)).unwrap();
    println!("wrote {}", path.display());
    (rgba, ui, ids)
}

fn px(rgba: &[u8], w: u32, x: f32, y: f32) -> [u8; 3] {
    let i = (y as usize * w as usize + x as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn same(got: [u8; 3], want: Color, what: &str) {
    let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
    assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what}: got {got:?}, want {w:?}");
}

fn check(theme: Theme, w: u32, name: &str) {
    let (rgba, ui, ids) = draw(theme, w, name);
    let at = |x: f32, y: f32| px(&rgba, w, x, y);
    let rect = |id| -> Rect { ui.rect(id).expect("built") };

    // P1: the ground, and the gap between the left and middle columns.
    same(at(2.0, 2.0), theme.ground, "ground");
    let (left, middle) = (rect(ids.left), rect(ids.middle));
    same(at((left.x + left.w + middle.x) / 2.0, 500.0), theme.ground, "column gap");

    // P2: no world, so the frame is empty ground; its note is centred, far from the corner.
    let f = rect(ids.frame);
    same(at(f.x + 5.0, f.y + 5.0), theme.ground, "empty frame");

    // P3: the pressed Relief tab is filled with --text (inside its 1 px frame).
    let relief = rect(ids.tabs[2]);
    same(at(relief.x + 3.0, relief.y + 3.0), theme.text, "pressed tab");

    // P4: the first band's swatch: row top + 3 padding + 6 to the swatch's middle, 6 px in.
    let row = rect(ids.bands[0]);
    same(at(row.x + 6.0, row.y + 3.0 + 3.0 + 6.0), Color::hex(0x17314f), "first band swatch");

    // P5: the disabled generate: its top hairline in --edge at mid width, the ground just inside its corner.
    let g = rect(ids.generate);
    same(at(g.x + g.w / 2.0, g.y), theme.edge, "generate top border");
    same(at(g.x + 3.0, g.y + 3.0), theme.ground, "generate inside");

    // P6: the select's arrow: its second row of ink, 13 px from the right edge plus 4, 9 + 2 down.
    let sel = rect(ids.select);
    same(at(sel.x + sel.w - 13.0 + 4.0, sel.y + 9.0 + 2.0), theme.text, "select chevron");

    // P7: the settings thumb, in the last 7 px of the scroll box's gutter, 10 px under its top. The box is 10 px wider\n    // than the column (the page's margin-right: -10px), so the thumb sits just outside the column's edge.
    let sc = rect(ids.settings_scroll);
    same(at(sc.x + sc.w - GUTTER + 2.0 + 3.0, sc.y + 10.0), theme.line, "settings thumb");

    assert!(rgba.chunks(4).any(|p| p[..3] != rgba[..3]), "the picture is one colour");
}

#[test]
fn preview_1400_dark() {
    check(Theme::dark(), 1400, "preview_1400.png");
}

#[test]
fn preview_1200_dark() {
    check(Theme::dark(), 1200, "preview_1200.png");
}

#[test]
fn preview_1000_dark() {
    check(Theme::dark(), 1000, "preview_1000.png");
}

#[test]
fn preview_1400_light() {
    check(Theme::light(), 1400, "preview_1400_light.png");
}
