//! The lists sheet drawn offscreen in both themes, and checked pixel by pixel. Positions come from the sheet's
//! constants: A's list box starts at y 42.5 and is scrolled by `sheet_offset`.

use nui_base::{Color, Gfx};
use nui_ui::tokens::{FADE_DIM, FADE_LIT, SPACE_24};
use nui_ui::{ARROW, GUTTER, PLOT_H, THUMB_W, TextLook, Theme, Tone, ellipsize, text_width};
use nui_win32::Renderer;
use widgets::lists::{
    A_H, A_LIST_TOP, A_POS, A_ROWS, A_W, B_POS, B_W, C_POS, HOVER_ROW, LONG_ROWS, NAMES, PRESSED, ROW_H, figures, sheet_offset, swatch,
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

    // The range is 20 rows x 26 + 24 (the scroll style's bottom padding) - 240 = 304; the sheet scrolls to 30 %, rounded.
    let range = A_ROWS as f32 * ROW_H + SPACE_24 - A_H;
    let y_a = sheet_offset(range);
    let row_top = |i: usize| A_LIST_TOP + ROW_H * i as f32 - y_a;

    // The scroll view runs 12 past the column (margin-right -12), so its gutter starts at x 350 = column right + 12 -
    // GUTTER. The thumb sits at gutter_x + 4, THUMB_W (4) wide, and the 1 px track at gutter_x + 6. The track runs
    // between two ARROW (12) arrows, 216 long from 54.5; the thumb is 216 x 240 / 544 = 95.3 high at
    // 54.5 + (216 - 95.3) x y / 304, muted. The track above it is a 1 px line, and the gutter beside both is the ground.
    let track = A_H - 2.0 * ARROW;
    let thumb_h = track * A_H / (A_H + range);
    let thumb_top = A_LIST_TOP + ARROW + (track - thumb_h) * y_a / range;
    let gx = A_POS.x + A_W + 12.0 - GUTTER;
    let mid = thumb_top + thumb_h / 2.0;
    same(at(gx + 4.0, mid), theme.muted, "thumb left pixel");
    same(at(gx + 4.0 + THUMB_W - 1.0, mid), theme.muted, "thumb is 4 px wide");
    same(at(gx + 4.0 + THUMB_W, mid), theme.ground, "gutter beside the thumb");
    same(at(gx + 3.0, mid), theme.ground, "gutter before the thumb");
    same(at(gx + 6.0, A_LIST_TOP + 20.0), theme.line, "track above the thumb");
    same(at(gx + 3.0, A_LIST_TOP + 20.0), theme.ground, "gutter beside the track");

    // Swatch interiors: 12 px at the row's top + 7 (padding 4, centred in 18). Pressed rows are not faded;
    // the others are at .5, and the hovered one at .85.
    let swatch_at = |i: usize| at(A_POS.x + 6.0, row_top(i) + 13.0);
    for i in PRESSED {
        same(swatch_at(i), swatch(i), "pressed swatch");
    }
    same(swatch_at(4), theme.faded(swatch(4), FADE_DIM), "unpressed swatch");
    same(swatch_at(HOVER_ROW), theme.faded(swatch(HOVER_ROW), FADE_LIT), "hovered swatch");
    // The hovered name's underline: the row's top + 4 (padding) + 13.5 (baseline) + 3, a whole pixel row
    // because the offset is whole. x 50 is inside the second glyph, which has no descender to cover it.
    same(at(50.0, row_top(HOVER_ROW) + 20.5), theme.faded(theme.text, FADE_LIT), "hovered underline");

    // B has 5 x 26 + 24 = 154 of content in 160: no thumb, so its gutter column is the ground.
    let bx = B_POS.x + B_W + 12.0 - GUTTER + 6.0;
    same(at(bx, A_LIST_TOP + 60.0), theme.ground, "B has no thumb");
    // B's rows start flush with its legend: the first row's swatch (top 42.5 + 7) at the column's left.
    same(at(B_POS.x + 6.0, A_LIST_TOP + 13.0), theme.faded(swatch(1), 1.0), "B first swatch");

    // C: the plot slot of the first trend row is at the column's top + 18, 140 wide, PLOT_H (32) high, framed by a hairline.
    same(at(C_POS.x + 70.0, C_POS.y + 18.0), theme.line, "C plot hairline");
    same(at(C_POS.x + 70.0, C_POS.y + 18.0 + PLOT_H - 1.0), theme.line, "C plot bottom hairline");

    // An ellipsised name in A: the last cell of the cut text holds the ellipsis glyph, so it is not blank.
    let body = TextLook::body(Tone::Text);
    for i in LONG_ROWS {
        let (width, _) = figures(i);
        // The row is 318 wide with the gutter: its name column is what the swatch, three gaps and the two
        // right cells leave.
        let column = 318.0 - 12.0 - 24.0 - text_width(width, 12.0) - 43.2;
        let cut = ellipsize(NAMES[i], body, column);
        assert!(cut.ends_with('\u{2026}'), "row {i} is cut");
        let x0 = A_POS.x + 20.0 + 7.2 * (cut.chars().count() - 1) as f32;
        let mut ink = 0;
        for y in (row_top(i) + 4.0) as usize..(row_top(i) + 22.0) as usize {
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
