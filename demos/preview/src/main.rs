//! preview [--data DIR] [--light] [--size WxH] [--shot out.png] [--view k[,k]] [--biome id,..]
//!         [--structure surface|under|id,..] [--look k,x,y] [--camera cx,cy,cz] [--cut N] [--ore kind,..] [--cut-look k,x,y]
//!
//! Without --shot, opens the world preview's window. With it, draws one frame offscreen and writes the PNG: the
//! same frame the window would show. With --data, the 2D map of an out folder fills the stage (step 14).
//! The view keys, --biome and --structure are the page's own `?view=`, `&biome=` and `&structure=`; --camera is its
//! `?camera=`, the 3D view's start in widths of the area (step 15). In the section view (`--view section`, step 16),
//! --cut is the cut shown (0-based, kept within the cuts), --ore the ores chosen in the list (their kind names, as
//! the cut then draws them alone) and --cut-look the look at the cut: k pixels a block, and the corner at x, y pixels
//! (settled on the first frame, so a place outside the cut is brought back into it).

use std::path::PathBuf;

use nui_base::{Vec2, Vec3, log};
use nui_ui::Theme;
use preview::cut::CutLook;
use preview::map::Look;
use preview::paint::View;
use preview::scene;
use preview::world::World;
use preview::{PreviewApp, PreviewState, text};

fn usage() -> ! {
    eprintln!(
        "usage: preview [--data DIR] [--light] [--size WxH] [--shot out.png] [--view k[,k]] [--biome id,..] [--structure surface|under|id,..] [--look k,x,y] [--camera cx,cy,cz] [--cut N] [--ore kind,..] [--cut-look k,x,y]"
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

fn parse_cut_look(s: &str) -> Option<CutLook> {
    let v: Vec<f64> = s.split(',').map(|p| p.trim().parse().ok()).collect::<Option<_>>()?;
    let &[k, x, y] = v.as_slice() else { return None };
    (k.is_finite() && x.is_finite() && y.is_finite()).then_some(CutLook { k, x, y })
}

fn parse_camera(s: &str) -> Option<Vec3> {
    let v: Vec<f32> = s.split(',').map(|p| p.trim().parse().ok()).collect::<Option<_>>()?;
    let &[x, y, z] = v.as_slice() else { return None };
    (x.is_finite() && y.is_finite() && z.is_finite()).then(|| Vec3::new(x, y, z))
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
    let (mut camera, mut cut, mut ores, mut cut_look) = (None, None, String::new(), None);
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
            "--camera" => camera = Some(parse_camera(&value()).unwrap_or_else(|| usage())),
            "--cut" => cut = Some(value().parse::<usize>().unwrap_or_else(|_| usage())),
            "--ore" => ores = value(),
            "--cut-look" => cut_look = Some(parse_cut_look(&value()).unwrap_or_else(|| usage())),
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
    for kind in ores.split(',').filter(|s| !s.is_empty()) {
        state.chosen_ores.insert(kind.to_string());
    }
    // A cut past the last is the last, once the world says how many there are; without a section there is none to name.
    if let Some(n) = cut {
        let cuts = world.as_ref().and_then(|w| w.section.as_ref()).map_or(0, |s| s.cuts.len());
        state.cut_at = n.min(cuts.saturating_sub(1));
    }
    if let Some(l) = cut_look {
        state.cut_look = l;
    }
    // Without a world there is nothing to look at: the page ignores it too.
    if let (Some(c), Some(w)) = (camera, world.as_ref()) {
        state.orbit = Some(scene::orbit(w, Some(c)));
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
