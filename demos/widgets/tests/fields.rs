//! The fields sheet drawn offscreen in both themes, and checked pixel by pixel. Each row of the script is a
//! fresh `Ui` replaying the steps up to it, so the pixels prove the state the caption names.

use nui_base::{Color, Gfx};
use nui_ui::Theme;
use nui_win32::Renderer;
use widgets::fields::{CELL_Y, FIELD_DY, LEFT_X, SCRIPT_STEP, SCRIPT_X, SCRIPT_Y0};
use widgets::{FIELDS, fields_sheet};

fn draw(theme: Theme, name: &str) -> Vec<u8> {
    let list = fields_sheet(theme);
    let (w, h) = FIELDS;
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
    let i = (y as usize * FIELDS.0 as usize + x as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn same(got: [u8; 3], want: Color, what: &str) {
    let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
    assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what}: got {got:?}, want {w:?}");
}

fn check(theme: Theme, name: &str) {
    let rgba = draw(theme, name);
    let at = |x: f32, y: f32| px(&rgba, x, y);
    // Script row i: the field's top-left is (600, 44 + 34 i); its frame's top line is y itself. The content
    // box starts at x 609 and y + 5 (padding 9 and 5, the border counted).
    let row_y = |i: usize| SCRIPT_Y0 + SCRIPT_STEP * i as f32;
    let inner_y = |i: usize| row_y(i) + 5.0;
    for i in 0..13 {
        // Focused (rows 1..11) draws its frame in the text colour, the others in the hairline's.
        let want = if (1..=11).contains(&i) { theme.text } else { theme.edge };
        same(at(SCRIPT_X + 100.0, row_y(i)), want, &format!("row {i} frame top"));
    }
    // Row 4: Right x4 puts the caret at 609 + 4 x 7.2 = 637.8, which rounds to 638; its middle is below the
    // glyph ink's top (the caret is 15.6 tall from inner.y + 1.2).
    same(at(638.0, inner_y(4) + 8.0), theme.text, "row 4 caret");
    // Row 6: columns 2..6 selected: from 609 + 14.4 = 623.4 to 609 + 43.2 = 652.2. Two pixels under the
    // content top is above the glyphs' ink (the selection starts at inner.y + 1.2).
    same(at(636.0, inner_y(6) + 2.0), theme.select, "row 6 selection");
    // Row 9: 56 chars are 403.2 wide in 226, so the text is pushed 181.2 left (content 222 wide) and the caret, at 403.2, lands
    // on 609 + 403.2 - 181.2 = 831: the last pixel column of the content box plus its extra one.
    same(at(831.0, inner_y(9) + 8.0), theme.text, "row 9 caret at the right edge");
    // Row 11: all selected.
    same(at(SCRIPT_X + 100.0, inner_y(11) + 2.0), theme.select, "row 11 selection");

    // The left column. Each cell's field is at (20, CELL_Y + 20).
    let field_y = |i: usize| CELL_Y[i] + FIELD_DY;
    // Cell 1: the compact field (padding 3, the border counted) has its content top at y + 3 and its glyph box
    // 1.2 below; "Try 3" is selected, from x 29 to 65. A point 5 under the field's top is inside the fill.
    same(at(LEFT_X + 10.0, field_y(1) + 3.0 + 2.0), theme.select, "compact name selected");
    // Cell 0: "ocean" is selected, columns 5..10, from 29 + 36 = 65 to 101. Text field content top is y + 5; the fill starts 1.2 lower, so 2.5 under it is a whole pixel row.
    same(at(LEFT_X + 9.0 + 36.0 + 2.0, field_y(0) + 5.0 + 2.5), theme.select, "ocean selected");
    // Cells 2 and 3 are setting rows 330 wide (x 20..350) whose number field takes the last 57.6, so its top
    // line is the row's padding (2) under the row's top: focused after two Ups, blurred after Enter.
    let num_top = |i: usize| field_y(i) + 2.0;
    same(at(LEFT_X + 330.0 - 20.0, num_top(2)), theme.text, "number focused frame");
    same(at(LEFT_X + 330.0 - 20.0, num_top(3)), theme.edge, "number blurred frame");
    // Cell 4: the number under the seed (row top = field top + 28 + 8) is focused by Tab, with "1.5" selected
    // at the right of its content box (right edge 20 + 260 - 9 = 271; 3 chars = 21.6 wide).
    let n4 = field_y(4) + 28.0 + 8.0 + 2.0;
    same(at(260.0, n4), theme.text, "tabbed number focused frame");
    same(at(268.0, n4 + 4.0 + 2.0), theme.select, "tabbed number selected");
    // Cell 5: nothing is focused: a hairline frame.
    same(at(100.0, field_y(5)), theme.edge, "empty seed frame");
    assert!(rgba.chunks(4).any(|p| p[..3] != rgba[..3]), "the picture is one colour");
}

#[test]
fn fields_dark() {
    check(Theme::dark(), "fields_dark.png");
}

#[test]
fn fields_light() {
    check(Theme::light(), "fields_light.png");
}
