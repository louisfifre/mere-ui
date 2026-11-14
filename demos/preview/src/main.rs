//! preview [--data DIR] [--light] [--size WxH] [--shot out.png] [--view k[,k]] [--biome id,..]
//!         [--structure surface|under|id,..] [--look k,x,y]
//!
//! Without --shot, opens the world preview's window. With it, draws one frame offscreen and writes the PNG: the
//! same frame the window would show. With --data, the 2D map of an out folder fills the stage (step 14).
//! The view keys, --biome and --structure are the page's own `?view=`, `&biome=` and `&structure=`.

use std::path::PathBuf;

use nui_base::{Vec2, log};
use nui_ui::Theme;
use preview::map::Look;
use preview::paint::View;
use preview::world::World;
use preview::{PreviewApp, PreviewState, text};

fn usage() -> ! {
    eprintln!(
        "usage: preview [--data DIR] [--light] [--size WxH] [--shot out.png] [--view k[,k]] [--biome id,..] [--structure surface|under|id,..] [--look k,x,y]"
    );
    std::process::exit(2);
}

fn parse_size(s: &str) -> Option<(u32, u32)> {
    let (w, h) = s.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

fn parse_look(s: &str) -> Option<Look> {
    let v: Vec<f32> = s.split(',').map(|p| p.trim().parse().ok()).collect::<Option<_>>()?;
    let &[k, x, y] = v.as_slice() else { return None };
    // The page keeps k within 1 and 16 and the corner within the frame, whatever it is asked.
    Some(Look { k: k.clamp(1.0, 16.0), x, y }.clamped())
}

/// Applies the page's `?view=` keys, one after the other as clicks on its tabs do.
fn apply_view(state: &mut PreviewState, key: &str) {
    match key {
        "" => {}
        "section" => state.view = 5,
        "flat2d" => state.in3d = false,
        "relief3d" => state.in3d = true,
        _ => match View::from_key(key) {
            Some(v) => state.view = v.index(),
            None => eprintln!("preview: unknown view '{key}'"),
        },
    }
}

fn main() {
    log::add_sink(log::stderr_sink(log::Level::Info));
    let (mut light, mut shot, mut size) = (false, None, (1400u32, 940u32));
    let (mut data, mut views, mut biomes, mut structures, mut look) = (None::<PathBuf>, Vec::new(), String::new(), String::new(), None);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = || args.next().unwrap_or_else(|| usage());
        match a.as_str() {
            "--light" => light = true,
            "--size" => size = parse_size(&value()).unwrap_or_else(|| usage()),
            "--shot" => shot = Some(value()),
            "--data" => data = Some(PathBuf::from(value())),
            "--view" => views.extend(value().split(',').map(str::to_string)),
            "--biome" => biomes = value(),
            "--structure" => structures = value(),
            "--look" => look = Some(parse_look(&value()).unwrap_or_else(|| usage())),
            _ => usage(),
        }
    }
    let world = data.map(|dir| {
        World::load(&dir).unwrap_or_else(|e| {
            eprintln!("preview: {e}");
            std::process::exit(1);
        })
    });

    let mut state = PreviewState::default();
    for key in &views {
        apply_view(&mut state, key);
    }
    for id in biomes.split(',').filter(|s| !s.is_empty()) {
        state.chosen[0].insert(id.to_string());
    }
    // "surface" and "under" stand for every structure of that part, whatever the pack calls them (lines 2866-2870).
    for id in structures.split(',').filter(|s| !s.is_empty()) {
        let part = match id {
            "surface" => Some(false),
            "under" => Some(true),
            _ => None,
        };
        match (part, world.as_ref().and_then(|w| w.structures.as_ref())) {
            (None, _) => {
                state.chosen[4].insert(id.to_string());
            }
            (Some(under), Some(s)) => {
                for (i, kind) in s.ids.iter().enumerate() {
                    if s.under[i] == under && s.count[i] > 0 {
                        state.chosen[4].insert(kind.clone());
                    }
                }
            }
            (Some(_), None) => {}
        }
    }
    if let Some(l) = look {
        state.look = l;
    }

    let theme = if light { Theme::light() } else { Theme::dark() };
    let (w, h) = size;
    let mut app = PreviewApp::new(theme, !light, Vec2::new(w as f32, h as f32), state, world);
    app.ui_mut().set_clipboard(Box::new(nui_win32::SystemClipboard));
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
