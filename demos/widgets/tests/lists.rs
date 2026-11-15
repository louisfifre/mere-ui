//! The lists sheet drawn offscreen in both themes, and checked pixel by pixel. Positions come from the sheet's
//! constants: A's list box starts at y 42.5 and is scrolled by `sheet_offset`.

use nui_base::{Color, Gfx};
use nui_ui::{TextLook, Theme, Tone, ellipsize, text_width};
use nui_win32::Renderer;
use widgets::lists::{
    A_H, A_LIST_TOP, A_POS, A_W, B_POS, B_W, C_POS, HOVER_ROW, LONG_ROWS, NAMES, PRESSED, ROW_H, figures, sheet_offset, swatch,
};
use widgets::{LISTS, lists_sheet};

fn draw(theme: Theme, name: &str) -> Vec<u8> {
    let list = lists_sheet(theme);
    let (w, h) = LISTS;
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
    let i = (y as usize * LISTS.0 as usize + x as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn same(got: [u8; 3], want: Color, what: &str) {
    let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
    assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what}: got {got:?}, want {w:?}");
}

fn check(theme: Theme, name: &str) {
    let rgba = draw(theme, name);
    let at = |x: f32, y: f32| px(&rgba, x, y);
    same(at(5.0, 5.0), theme.ground, "ground");

    // The range is 20 rows x 24 + 28 - 240 = 268; the sheet scrolls to 30 %, rounded.
    let y_a = sheet_offset(268.0);
    let row_top = |i: usize| A_LIST_TOP + ROW_H * i as f32 - y_a;

    // The bar sits in the middle of the 11 px gutter (x 20 + 340 - 5.5). The track runs between two 10 px arrows,
    // 220 long from 52.5; the thumb is 220 x 240 / 508 = 103.9 high at 52.5 + (220 - 103.9) x y / 268, muted. The
    // track above it is a 1 px line, and the gutter beside both is the ground.
    let thumb_h = (A_H - 20.0) * A_H / 508.0;
    let thumb_top = A_LIST_TOP + 10.0 + (A_H - 20.0 - thumb_h) * y_a / 268.0;
    let tx = A_POS.x + A_W + 10.0 - 5.5;
    same(at(tx, thumb_top + thumb_h / 2.0), theme.muted, "thumb centre");
    same(at(tx + 1.0, thumb_top + thumb_h / 2.0), theme.muted, "thumb is 3 px wide");
    same(at(tx + 2.0, thumb_top + thumb_h / 2.0), theme.ground, "gutter beside the thumb");
    same(at(tx, A_LIST_TOP + 20.0), theme.line, "track above the thumb");
    same(at(tx - 3.0, A_LIST_TOP + 20.0), theme.ground, "gutter beside the track");

    // Swatch interiors: 12 px at the row's top + 6 (padding 3, centred in 18). Pressed rows are not faded;
    // the others are at .5, and the hovered one at .85.
    let swatch_at = |i: usize| at(A_POS.x + 6.0, row_top(i) + 12.0);
    for i in PRESSED {
        same(swatch_at(i), swatch(i), "pressed swatch");
    }
    same(swatch_at(4), theme.faded(swatch(4), 0.5), "unpressed swatch");
    same(swatch_at(HOVER_ROW), theme.faded(swatch(HOVER_ROW), 0.85), "hovered swatch");
    // The hovered name's underline: the row's top + 3 (padding) + 13.5 (baseline) + 3, a whole pixel row
    // because the offset is whole. x 50 is inside the second glyph, which has no descender to cover it.
    same(at(50.0, row_top(HOVER_ROW) + 19.5), theme.faded(theme.text, 0.85), "hovered underline");

    // B has 5 x 24 + 28 = 148 of content in 160: no thumb, so its gutter column is the ground.
    let bx = B_POS.x + B_W + 10.0 - 5.5;
    same(at(bx, A_LIST_TOP + 60.0), theme.ground, "B has no thumb");
    // B's rows start flush with its legend: the first row's swatch (top 42.5 + 6) at the column's left.
    same(at(B_POS.x + 6.0, A_LIST_TOP + 12.0), theme.faded(swatch(1), 1.0), "B first swatch");

    // C: the plot slot of the first trend row is at the column's top + 18, 140 wide, framed by a hairline.
    same(at(C_POS.x + 70.0, C_POS.y + 18.0), theme.line, "C plot hairline");

    // An ellipsised name in A: the last cell of the cut text holds the ellipsis glyph, so it is not blank.
    let body = TextLook::body(Tone::Text);
    for i in LONG_ROWS {
        let (width, _) = figures(i);
        // The row is 319 wide with the gutter: its name column is what the swatch, three gaps and the two
        // right cells leave.
        let column = 319.0 - 12.0 - 24.0 - text_width(width, 12.0) - 43.2;
        let cut = ellipsize(NAMES[i], body, column);
        assert!(cut.ends_with('\u{2026}'), "row {i} is cut");
        let x0 = A_POS.x + 20.0 + 7.2 * (cut.chars().count() - 1) as f32;
        let mut ink = 0;
        for y in (row_top(i) + 3.0) as usize..(row_top(i) + 21.0) as usize {
            for x in x0 as usize..(x0 + 7.0) as usize {
                if at(x as f32, y as f32) != [rgba[0], rgba[1], rgba[2]] {
                    ink += 1;
                }
            }
        }
        assert!(ink > 0, "row {i}: nothing is drawn in the ellipsis cell");
    }
    assert!(rgba.chunks(4).any(|p| p[..3] != rgba[..3]), "the picture is one colour");
}

#[test]
fn lists_dark() {
    check(Theme::dark(), "lists_dark.png");
}

#[test]
fn lists_light() {
    check(Theme::light(), "lists_light.png");
}
