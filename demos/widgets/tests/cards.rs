//! Every card of the Design System drawn offscreen in both themes (`target/tmp/card_<name>_<theme>.png`).
//! The list is `widgets::cards::CARDS`; a new card adds nothing here.

use mere_base::Gfx;
use mere_ui::Theme;
use mere_win32::Renderer;
use widgets::cards::CARDS;

fn draw(theme: Theme, name: &str, themed: &str) {
    let build = CARDS.iter().find(|(n, _)| *n == name).unwrap().1;
    let card = build(theme);
    let (w, h) = (card.size.0.max(1), card.size.1.max(1));
    let mut r = Renderer::offscreen(w, h, 1.0).expect("offscreen renderer");
    r.begin(theme.ground);
    r.draw_list(&card.list);
    r.end();
    let rgba = r.read_rgba().unwrap();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("card_{name}_{themed}.png"));
    std::fs::write(&path, mere_png::encode(w, h, &rgba)).unwrap();
    println!("wrote {}", path.display());
}

#[test]
fn every_card_in_both_themes() {
    for (name, _) in CARDS {
        draw(Theme::dark(), name, "dark");
        draw(Theme::light(), name, "light");
    }
}
