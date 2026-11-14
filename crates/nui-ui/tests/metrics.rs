//! Checks the ascent constant against a picture: "HHHH" at 100 px, its ink bottom is the baseline.

use nui_base::{Gfx, Vec2};
use nui_layout::{Dim, Insets, Position, Style};
use nui_ui::{ASCENT_EM, TextLook, Theme, Tone, Ui};
use nui_win32::Renderer;

#[test]
fn plex_mono_ascent_matches_the_constant() {
    let theme = Theme::dark();
    let mut ui = Ui::new(theme);
    ui.begin_frame(Vec2::new(600.0, 200.0));
    let style = Style { position: Position::Absolute, inset: Insets { left: Dim::Px(0.0), top: Dim::Px(0.0), ..Insets::default() }, ..Style::default() };
    ui.node("probe", style, |ui| {
        ui.text("HHHH", TextLook { size: 100.0, line: 150.0, weight: nui_base::Weight::Regular, tone: Tone::Text });
    });
    let list = ui.end_frame().clone();
    let mut r = Renderer::offscreen(600, 200, 1.0).expect("offscreen renderer");
    r.begin(theme.ground);
    r.draw_list(&list);
    r.end();
    let rgba = r.read_rgba().unwrap();
    let red = |x: usize, y: usize| rgba[(y * 600 + x) * 4];
    let ink_bottom = (0..200).rev().find(|&y| (0..300).any(|x| red(x, y) > 64)).expect("ink") as f32 + 1.0;
    let right = (0..600).rev().find(|&x| (0..200).any(|y| red(x, y) > 64)).expect("ink") + 1;
    println!("ink bottom {ink_bottom}, ink right {right}, expected baseline {}", 10.0 + ASCENT_EM * 100.0);
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("metrics.png");
    std::fs::write(&path, nui_png::encode(600, 200, &rgba)).unwrap();
    assert!((ink_bottom - 112.5).abs() <= 2.0, "ink bottom {ink_bottom}");
    assert!((200..=240).contains(&right), "ink right {right}");
}
