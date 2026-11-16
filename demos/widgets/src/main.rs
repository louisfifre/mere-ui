//! widgets [--card NAME] [--light] [--shot out.png]
//!
//! Shows one card of the Design System (default: button), drawn once with its forced states: the window and
//! `--shot` draw the same list. With --shot, draws one frame offscreen, writes the PNG and exits.

use nui_base::{App, Event, Gfx, log};
use nui_ui::Theme;
use widgets::cards::{CARDS, Card};
use widgets::text;

const DEFAULT_CARD: &str = "button";

struct Showcase {
    theme: Theme,
    card: Card,
}

impl App for Showcase {
    fn start(&mut self, _: &mut dyn Gfx) {}

    fn event(&mut self, e: Event) -> bool {
        // A resize needs a new frame; nothing else changes a card.
        matches!(e, Event::Resized { .. })
    }

    fn frame(&mut self, gfx: &mut dyn Gfx, _: f32) -> bool {
        // The forced states (hover, held) are part of the picture, so the card is built once, not per frame.
        gfx.begin(self.theme.ground);
        gfx.draw_list(&self.card.list);
        gfx.end();
        false
    }
}

fn usage() -> ! {
    let names: Vec<_> = CARDS.iter().map(|c| c.0).collect();
    eprintln!("usage: widgets [--card NAME] [--light] [--shot out.png]\ncards: {}", names.join(", "));
    std::process::exit(2);
}

fn main() {
    log::add_sink(log::stderr_sink(log::Level::Info));
    let (mut light, mut shot, mut name) = (false, None, DEFAULT_CARD.to_string());
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--light" => light = true,
            "--card" => name = args.next().unwrap_or_else(|| usage()),
            "--shot" => shot = Some(args.next().unwrap_or_else(|| usage())),
            _ => usage(),
        }
    }
    let theme = if light { Theme::light() } else { Theme::dark() };
    let Some(build) = widgets::cards::find(&name) else {
        eprintln!("widgets: no card {name}");
        usage();
    };
    let card = build(theme);
    let (w, h) = (card.size.0.max(1), card.size.1.max(1));
    let mut app = Showcase { theme, card };
    match shot {
        Some(path) => {
            let rgba = nui_win32::shot(&mut app, w, h, 1.0).unwrap_or_else(|e| {
                eprintln!("widgets: {e}");
                std::process::exit(1);
            });
            if let Err(e) = std::fs::write(&path, nui_png::encode(w, h, &rgba)) {
                eprintln!("widgets: {path}: {e}");
                std::process::exit(1);
            }
            nui_base::observe!(Info, "wrote {path}");
        }
        None => {
            if let Err(e) = nui_win32::run(text::WINDOW_TITLE, &mut app) {
                eprintln!("widgets: {e}");
                std::process::exit(1);
            }
        }
    }
}
