//! The page's wording (preview.html, line numbers in the comments). Nothing is written here that the page
//! does not say.

pub const WINDOW_TITLE: &str = "World preview";
/// Line 455.
pub const WORLD: &str = "World";
/// Line 456.
pub const SEED: &str = "Seed";
/// Line 457.
pub const SEED_RANDOM: &str = "random";
/// Line 458.
pub const SPAN: &str = "Scanned area";
/// Line 581.
pub const PROFILES: &str = "Settings";
/// Line 582.
pub const PACK: &str = "Pack";
/// Line 467.
pub const UP_TO_DATE: &str = "The map shows these settings.";
/// Line 464, `detailValue` of 64, 32 and 16.
pub const DETAILS: [&str; 3] = ["64 blocks", "32 blocks", "16 blocks"];
/// Line 462.
pub const GENERATE: &str = "Generate preview";
/// Line 480.
pub const EXPORT: &str = "Export";
/// Line 491.
pub const FIGURES: &str = "In figures";
/// Line 492.
pub const HISTORY: &str = "History";
/// Line 588.
pub const VIEWS: [&str; 5] = ["Biomes", "Flat", "Relief", "Temperature", "Structures"];
/// Line 619.
pub const SECTION: &str = "Section";
/// Line 589.
pub const SHAPES: [&str; 2] = ["2D", "3D"];
/// Line 508, the height view: its title and the name of its first column.
pub const LIST_HEIGHT: [&str; 2] = ["Heights", "height"];
/// Line 520.
pub const COL_WIDTH: &str = "typical width";
/// Line 521.
pub const COL_SHARE: &str = "share";
/// Line 510.
pub const PICK_HINT: &str = "Click rows, here or on the map.";
/// Line 580.
pub const RESET_HINT: &str = "Click \u{201c}pack\u{2026}\u{201d} beside a value to put it back to the pack\u{2019}s.";
/// Lines 528-529.
pub const ON: &str = "on";
pub const OFF: &str = "off";
/// Line 525, in `groupOrder` order (the ores group is not built: it shows in the section view only).
pub const GROUPS: [&str; 5] = ["Biome size and climate", "Relief", "Continents, oceans and islands", "Caves", "Not drawn here"];
/// Lines 605-611, in key order.
pub const FIGURE_NAMES: [&str; 7] = ["Flat land", "Typical flat zone", "Typical biome", "Sea", "High land", "Highest peak", "Biomes present"];

/// Line 593.
pub const NO_PREVIEW: &str = "No preview yet. Choose a seed and generate one.";
/// Line 599.
pub const NO_STRUCTURES: &str = "No structures for this preview. Generate it again.";

/// Line 597: `cut` says the ticks of the section's cut are on the frame.
pub fn area_probe(blocks: &str, square: &str, cut: bool) -> String {
    format!("{blocks} \u{d7} {blocks} blocks, north up. One square: {square} blocks. Cross: 0,0.{}", if cut { " Ticks: the cut." } else { "" })
}

/// Line 596: three spaces between the fields.
pub fn probe(x: &str, z: &str, y: &str, biome: &str) -> String {
    format!("x {x}   z {z}   height {y}   {biome}")
}

/// Line 602.
pub fn structure_probe(x: &str, z: &str, name: &str) -> String {
    format!("x {x}   z {z}   {name}")
}

/// Line 652 with no decimals: `toLocaleString('en-US')`, groups of three, halves away from zero.
pub fn num(v: f64) -> String {
    let r = (v.abs() + 0.5).floor() as u64;
    let digits = r.to_string();
    let mut grouped = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    if v < 0.0 && r != 0 { format!("-{grouped}") } else { grouped }
}

/// Lines 1120-1123: the name alone, without the mod it comes from, in words.
pub fn pretty(id: &str) -> String {
    let name = id.rsplit(':').next().unwrap_or("");
    let mut words = String::new();
    let mut gap = false;
    for c in name.chars() {
        if c == '_' || c == '/' {
            // A run of them is one space.
            gap = true;
        } else {
            if gap {
                words.push(' ');
                gap = false;
            }
            words.push(c);
        }
    }
    if gap {
        words.push(' ');
    }
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}