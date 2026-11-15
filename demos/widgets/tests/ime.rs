//! The input method sheet drawn offscreen in both themes, and checked pixel by pixel. Each row replays the
//! input method's messages through a fresh `Ui`, so the pixels prove the state the caption names.
//!
//! Rows 0..4 are text fields at (330, y), y = 20 + 38 i, 250 wide: the content starts at x 337 and its top
//! at y + 4, the glyph box 1.2 lower and 15.6 tall, the underline row at y + 19. A cell is 7.2 wide, so
//! "world " (6 cells) ends at 337 + 43.2 = 380.2, where the composition starts.

use nui_base::{Cmd, Color, Gfx};
use nui_ui::Theme;
use nui_win32::Renderer;
use widgets::ime::{FIELD_X, ROW_STEP, ROW_Y0};
use widgets::{IME, ime_sheet};

fn draw(theme: Theme, name: &str) -> (Vec<u8>, nui_base::DrawList) {
    let list = ime_sheet(theme);
    let (w, h) = IME;
    let mut r = Renderer::offscreen(w, h, 1.0).expect("offscreen renderer");
    r.begin(theme.ground);
    r.draw_list(&list);
    r.end();
    let rgba = r.read_rgba().unwrap();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, nui_png::encode(w, h, &rgba)).unwrap();
    println!("wrote {}", path.display());
    (rgba, list)
}

fn px(rgba: &[u8], x: f32, y: f32) -> [u8; 3] {
    let i = (y as usize * IME.0 as usize + x as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn same(got: [u8; 3], want: Color, what: &str) {
    let w = [want.r, want.g, want.b].map(|v| (v * 255.0).round() as i32);
    assert!(got.iter().zip(w).all(|(a, b)| (*a as i32 - b).abs() <= 1), "{what}: got {got:?}, want {w:?}");
}

fn check(theme: Theme, name: &str) {
    let (rgba, list) = draw(theme, name);
    let at = |x: f32, y: f32| px(&rgba, x, y);
    let row_y = |i: usize| ROW_Y0 + ROW_STEP * i as f32;

    // Row 0: にほんご (8 cells) from 380.2 to 437.8, underlined, the caret at its end (round 437.8 = 438).
    let y = row_y(0);
    same(at(394.0, y + 19.0), theme.text, "row 0 underline");
    same(at(445.0, y + 19.0), theme.ground, "row 0 nothing past the composition");
    same(at(438.0, y + 12.0), theme.text, "row 0 caret at the end");
    // Row 1: 日本語 (6 cells) from 380.2 to 423.4, the caret after 日: 380.2 + 14.4 = 394.6, round 395.
    let y = row_y(1);
    same(at(420.0, y + 19.0), theme.text, "row 1 underline");
    same(at(395.0, y + 12.0), theme.text, "row 1 caret after the first char");
    // Row 2: committed: "world 日本語" is 12 cells, the caret at 337 + 86.4 = 423.4, and no underline.
    let y = row_y(2);
    same(at(423.0, y + 12.0), theme.text, "row 2 caret after the commit");
    same(at(438.0, y + 12.0), theme.ground, "row 2 no caret where the preedit was");
    same(at(445.0, y + 19.0), theme.ground, "row 2 no underline");
    // Row 3: cancelled: back to "world " with the caret at 380.2, and nothing under the text.
    let y = row_y(3);
    same(at(380.0, y + 12.0), theme.text, "row 3 caret");
    same(at(400.0, y + 19.0), theme.ground, "row 3 no underline");
    // Row 4: "ab" + かな + "ef", the composition at cells 2..6 (351.4 to 380.2), the caret at its end.
    let y = row_y(4);
    same(at(365.0, y + 19.0), theme.text, "row 4 underline");
    same(at(380.0, y + 12.0), theme.text, "row 4 caret");
    // Row 5: the number field has the keyboard (its frame is in the text colour) and shows no composition. Its
    // top line is the row's padding (2) under the row's top.
    let y = row_y(5);
    same(at(550.0, y + 2.0), theme.text, "row 5 focused frame");

    // Whatever the fallback font, a composition draws glyph ink inside its boxes.
    for i in 0..3 {
        let y = row_y(i);
        let mut ink = 0;
        for py in (y as usize + 5)..(y as usize + 21) {
            for pxx in 381..437 {
                let p = px(&rgba, pxx as f32, py as f32);
                if p.iter().zip([theme.ground.r, theme.ground.g, theme.ground.b]).any(|(a, g)| (*a as i32 - (g * 255.0).round() as i32).abs() > 1) {
                    ink += 1;
                }
            }
        }
        assert!(ink >= 20, "row {i}: only {ink} pixels differ from the ground inside the composition's box");
    }

    // The draw list agrees: rows 2, 3 and 5 hold no 1 px fill on the underline row.
    for i in [2, 3, 5] {
        let y = row_y(i);
        let stray = list.cmds.iter().any(|c| match c {
            Cmd::Fill { rect, .. } => rect.h == 1.0 && rect.x >= FIELD_X - 80.0 && ((rect.y - (y + 19.0)).abs() < 0.01 || (rect.y - (y + 18.0)).abs() < 0.01),
            _ => false,
        });
        assert!(!stray, "row {i} has an underline");
    }
    assert!(rgba.chunks(4).any(|p| p[..3] != rgba[..3]), "the picture is one colour");
}

#[test]
fn ime_dark() {
    check(Theme::dark(), "ime_dark.png");
}

#[test]
fn ime_light() {
    check(Theme::light(), "ime_light.png");
}
