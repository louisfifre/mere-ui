//! Every sentence the demo shows, in one place, so wording changes touch one file.

use crate::world::World;

pub const WINDOW_TITLE: &str = "Relief";
pub const TITLE: &str = "Relief";
pub const HINT: &str = "Drag to turn, wheel to zoom";

pub fn seed(w: &World) -> String {
    format!("Seed {}", w.seed)
}

pub fn grid(w: &World) -> String {
    format!("{0} x {0} points, {1} blocks apart", w.samples, w.step)
}

pub fn heights(w: &World) -> String {
    let (lo, hi) = w.height_range();
    format!("Heights {lo} to {hi}, sea at {}", w.sea_level)
}

pub fn biomes(w: &World) -> String {
    format!("{} of {} biomes", w.biomes_used, w.biome_count)
}

pub fn usage() -> &'static str {
    "usage: relief <data-dir> [--shot out.png --size WxH --scale S]"
}
