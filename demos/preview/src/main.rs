//! preview [--light] [--size WxH] [--shot out.png]
//!
//! Without --shot, opens the world preview's shell in a window. With it, draws one frame offscreen and writes
//! the PNG: the same frame the window would show. Step 13: static values, the map comes with step 14.

use nui_base::{App, Event, Gfx, Vec2, log};
use nui_ui::{Theme, Ui};
use preview::{PreviewState, build, text};

struct Preview {
    ui: Ui,
    theme: Theme,
    size: Vec2,
    state: PreviewState,
}

impl App for Preview {
    fn start(&mut self, _: &mut dyn Gfx) {}

    fn event(&mut self, e: Event) -> bool {
        if let Event::Resized { width_px, height_px, scale } = e {
            self.size = Vec2::new(width_px as f32 / scale, height_px as f32 / scale);
            return true;
        }
        self.ui.event(&e)
    }

    fn frame(&mut self, gfx: &mut dyn Gfx, _: f32) -> bool {
        gfx.begin(self.theme.ground);
        // The scrollbar's gutter comes a frame late, and a click changes the state after its widget was
        // built: build again until neither is pending, at most three times, so a picture shows the settled page.
        let mut list = nui_base::DrawList::default();
        for _ in 0..3 {
            let before = self.state.clone();
            self.ui.begin_frame(self.size);
            build(&mut self.ui, &mut self.state, self.size);
            list = self.ui.end_frame().clone();
            if !self.ui.want_redraw() && self.state == before {
                break;
            }
        }
        gfx.draw_list(&list);
        gfx.end();
        self.ui.want_redraw()
    }
}

fn usage() -> ! {
    eprintln!("usage: preview [--light] [--size WxH] [--shot out.png]");
    std::process::exit(2);
}

fn parse_size(s: &str) -> Option<(u32, u32)> {
    let (w, h) = s.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

fn main() {
    log::add_sink(log::stderr_sink(log::Level::Info));
    let (mut light, mut shot, mut size) = (false, None, (1400u32, 940u32));
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--light" => light = true,
            "--size" => size = args.next().and_then(|s| parse_size(&s)).unwrap_or_else(|| usage()),
            "--shot" => shot = Some(args.next().unwrap_or_else(|| usage())),
            _ => usage(),
        }
    }
    let theme = if light { Theme::light() } else { Theme::dark() };
    let (w, h) = size;
    let mut ui = Ui::new(theme);
    ui.set_clipboard(Box::new(nui_win32::SystemClipboard));
    let mut app = Preview { ui, theme, size: Vec2::new(w as f32, h as f32), state: PreviewState::default() };
    match shot {
        Some(path) => {
            let rgba = nui_win32::shot(&mut app, w, h, 1.0).unwrap_or_else(|e| {
                eprintln!("preview: {e}");
                std::process::exit(1);
            });
            if let Err(e) = std::fs::write(&path, nui_png::encode(w, h, &rgba)) {
                eprintln!("preview: {path}: {e}");
                std::process::exit(1);
            }
            nui_base::observe!(Info, "wrote {path}");
        }
        None => {
            if let Err(e) = nui_win32::run(text::WINDOW_TITLE, &mut app) {
                eprintln!("preview: {e}");
                std::process::exit(1);
            }
        }
    }
}
