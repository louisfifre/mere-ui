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

/// Line 468.
pub const GENERATING: &str = "Generating\u{2026}";
/// Line 469, kept for the tooltips of a later step.
pub const STOP: &str = "Stop the run";
/// Line 470.
pub fn running(elapsed: &str, about: &str) -> String {
    format!("{elapsed} of about {about}.")
}
/// Line 471.
pub fn measuring(elapsed: &str) -> String {
    format!("{elapsed}. A first run at this detail: its time is kept for the next ones.")
}
/// Line 473.
pub const STEP_START: &str = "Starting the game";
/// Line 474.
pub fn step_sketch(step: u32) -> String {
    format!("Sketch, every {step} blocks")
}
/// Line 475.
pub fn step_last(step: u32) -> String {
    format!("Map, every {step} blocks")
}
/// Line 476.
pub const STEP_EXTRAS: &str = "Cuts, ores and structures";
/// Line 479.
pub const STOPPED: &str = "Stopped. The map is the one from before.";
/// Line 482.
pub const EXPORTING: &str = "Exporting\u{2026}";
/// Line 483.
pub fn export_step(view: &str) -> String {
    format!("Pictures: {view}")
}
/// Line 485.
pub const EXPORTED: &str = "Exported.";
/// Line 486.
pub const EXPORTED_TO: &str = "Exported to";
/// Line 487.
pub const OPEN_FOLDER: &str = "Open the folder";
/// Line 488.
pub const CLOSE_NOTICE: &str = "Close";
/// Line 489.
pub const OPEN_FAILED: &str = "The folder could not be opened: it may have been moved or deleted.";
/// Line 490.
pub const EXPORT_FAILED: &str = "The export failed.";
/// Line 590.
pub fn done(wait: &str) -> String {
    format!("Preview ready in {wait}.")
}
/// Line 591.
pub const FAILED: &str = "The run failed. Try again, then see last-run.log in the preview folder.";
/// Line 592.
pub const REFUSED: &str = "Seed refused: it takes letters, digits, spaces and hyphens.";
/// Line 2571 and 2579: the marks of a done step and of the one under way.
pub const DONE_MARK: &str = "\u{2713}";
pub const NOW_MARK: &str = "\u{203a}";

/// Line 461.
pub fn duration(s: u64) -> String {
    if s < 60 {
        format!("{s} s")
    } else if s % 60 == 0 {
        format!("{} min", s / 60)
    } else {
        format!("{} min {} s", s / 60, s % 60)
    }
}

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

/// Line 508: what the list beside the map holds in each view, by `View::index`: its title and the name of its
/// first column.
pub const LISTS: [[&str; 2]; 5] = [["Biomes", "biome"], ["Ground", "ground"], ["Heights", "height"], ["Temperatures", "climate"], ["Structures", "structure"]];
/// Line 522.
pub const COL_SPACING: &str = "one every";
/// Line 524.
pub const COL_COUNT: &str = "count";
/// Line 512: no structure is drawn until it is chosen.
pub const PICK_HINT_MARKS: &str = "Click the rows to draw on the map. None is drawn until then.";
/// Line 518.
pub const PICK_NONE: &str = "Show all";
/// Line 513.
pub const PICK_NONE_MARKS: &str = "Draw none";
/// Line 514.
pub fn picked_many(count: usize) -> String {
    format!("{count} chosen")
}
/// Line 515.
pub fn picked_share(share: &str) -> String {
    format!(" \u{b7} {share} of the area")
}
/// Line 516.
pub fn picked_count(count: &str, of: &str) -> String {
    format!(" \u{b7} {count} of {of}")
}
/// Line 519.
pub const FILTER: &str = "Filter by name\u{2026}";
/// Line 614.
pub const ABSENT: &str = "Not in this area";
/// Line 600: the structures' two parts, surface first.
pub const PARTS: [&str; 2] = ["On the surface", "Under the ground"];
/// Line 648.
pub const LEGEND_FLAT: [&str; 3] = ["Flat", "Slopes", "Water"];
/// Line 649.
pub const LEGEND_TEMPERATURE: [&str; 5] = ["Frozen", "Cold", "Temperate", "Warm", "Scorching"];
/// Line 502.
pub const CHANGE: &str = "change";
/// Line 504.
pub const SAME: &str = "=";
/// Line 505.
pub const CLOSE: &str = "\u{2248}";
/// Line 506.
pub const POINTS: &str = " pt";
/// Line 613.
pub fn count_of(count: usize, total: usize) -> String {
    format!("{count} of {total}")
}
/// Line 617: a whole number of blocks as JavaScript writes a number.
pub fn altitude(v: f64) -> String {
    format!("height {v}")
}
/// Line 495.
pub const HISTORY_FEW: &str = "Generate again with this seed, over this area, at this detail, and each figure gets a line from run to run.";
/// Line 496.
pub fn history_hint(n: usize) -> String {
    format!("The {n} runs of this seed over this area, at this detail, oldest left. Point at one to see what it changed.")
}
/// Line 497.
pub fn history_run(at: usize, of: usize, what: &str) -> String {
    format!("Run {at} of {of}: {what}.")
}
/// Line 498.
pub const HISTORY_FIRST: &str = "the first one kept";
/// Line 499.
pub const HISTORY_SAME: &str = "same settings";
/// Line 500: `from` and `to` as JavaScript writes a number.
pub fn history_moved(name: &str, from: f64, to: f64) -> String {
    format!("{name} {from} \u{2192} {to}")
}
/// Line 501.
pub fn history_many(count: usize) -> String {
    format!("{count} settings changed")
}

/// Line 620.
pub const NO_SECTION: &str = "No section for this preview. Generate it again.";
/// Line 621.
pub const NEXT_CUT: &str = "Another cut";

/// Line 623: `at` and `of` are printed as they are, `z`, `from` and `to` already formatted.
pub fn cut_rest(at: usize, of: usize, z: &str, from: &str, to: &str) -> String {
    format!("Cut {at} of {of}, west to east at z {z}, x {from} to {to}. True scale. Wheel: closer.")
}

/// Line 624.
pub fn nav_note(times: &str) -> String {
    format!("The whole cut, heights \u{d7} {times}. Click or drag the frame to look along it.")
}

/// Line 625: three spaces between the fields. `y` is a whole number of blocks.
pub fn cut_probe(x: &str, y: f64, depth: &str, what: &str, biome: &str) -> String {
    format!("x {x}   height {y}   {depth}   {what}   {biome}")
}

/// Line 626: `depth` is a whole number of blocks.
pub fn cut_depth(depth: f64) -> String {
    if depth > 0.0 { format!("{depth} under the ground") } else { "above the ground".to_string() }
}

/// Line 627, indexed by the cell: air, rock, water, lava.
pub const CUT_CELLS: [&str; 4] = ["Air", "Rock", "Water", "Lava"];
/// Line 628.
pub const CUT_EARTH: &str = "Top layers";
/// Line 629, in `section::ORE_KINDS` order.
pub const ORES: [&str; 8] = ["Coal", "Copper", "Iron", "Gold", "Lapis", "Redstone", "Diamond", "Emerald"];
/// Line 630: the list's title and the name of its first column.
pub const ORE_LIST: [&str; 2] = ["Ores", "ore"];
/// Line 631.
pub const ORE_COLUMNS: [&str; 2] = ["low ground", "high ground"];
/// Line 633.
pub const ORE_PICK_HINT: &str = "Click an ore to see it alone.";

/// Line 635.
pub fn ore_unplayed(count: usize) -> String {
    format!("{count} ore rules use a placement step this page cannot play: they are left out.")
}

/// Line 652 with `digits` fraction digits: `toLocaleString('en-US', { minimumFractionDigits: digits,
/// maximumFractionDigits: digits })`, halves away from zero, computed on the double as it is. UNVERIFIED for the
/// exact binary halves (Intl may round on the shortest decimal of the double instead); the Node test compares
/// the strings of the real data.
pub fn num_digits(v: f64, digits: usize) -> String {
    let scale = 10f64.powi(digits as i32);
    let r = (v.abs() * scale + 0.5).floor() as u64;
    let unit = scale as u64;
    let whole = (r / unit).to_string();
    let mut out = String::new();
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if digits > 0 {
        out.push_str(&format!(".{:0width$}", r % unit, width = digits));
    }
    if v < 0.0 && r != 0 { format!("-{out}") } else { out }
}

/// Line 2303: how many times the heights are stretched, with a decimal below 1.95 and whole above.
pub fn times(v: f64) -> String {
    if v < 1.95 { num_digits(v, 1) } else { num((v + 0.5).floor()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numbers_are_written_as_the_page_writes_them() {
        assert_eq!(num_digits(1234.56, 1), "1,234.6");
        assert_eq!(num_digits(0.04, 1), "0.0");
        assert_eq!(num_digits(-0.04, 1), "0.0", "no minus sign on a zero");
        assert_eq!(num_digits(-1500.25, 1), "-1,500.3");
        assert_eq!(num_digits(7.0, 0), "7");
        assert_eq!(times(1.3), "1.3");
        assert_eq!(times(1.94), "1.9");
        assert_eq!(times(12.4), "12");
        assert_eq!(times(1.95), "2");
        assert_eq!(nav_note("12"), "The whole cut, heights \u{d7} 12. Click or drag the frame to look along it.");
    }

    #[test]
    fn the_runs_speak_as_the_page_does() {
        assert_eq!(GENERATING, "Generating…");
        assert_eq!(running("20 s", "50 s"), "20 s of about 50 s.");
        assert_eq!(measuring("5 s"), "5 s. A first run at this detail: its time is kept for the next ones.");
        assert_eq!((step_sketch(64).as_str(), step_last(16).as_str()), ("Sketch, every 64 blocks", "Map, every 16 blocks"));
        assert_eq!(STOPPED, "Stopped. The map is the one from before.");
        assert_eq!(done("1 min 5 s"), "Preview ready in 1 min 5 s.");
        assert_eq!(export_step("Biomes"), "Pictures: Biomes");
        assert_eq!((EXPORTED, EXPORTED_TO, OPEN_FOLDER, CLOSE_NOTICE), ("Exported.", "Exported to", "Open the folder", "Close"));
        assert_eq!(REFUSED, "Seed refused: it takes letters, digits, spaces and hyphens.");
        assert_eq!((DONE_MARK, NOW_MARK), ("✓", "›"));
    }

    #[test]
    fn the_cut_speaks_as_the_page_does() {
        assert_eq!(cut_rest(1, 4, "-1,504", "-2,048", "2,048"), "Cut 1 of 4, west to east at z -1,504, x -2,048 to 2,048. True scale. Wheel: closer.");
        assert_eq!(cut_probe("-100", 12.0, &cut_depth(5.0), "Rock", "Plains"), "x -100   height 12   5 under the ground   Rock   Plains");
        assert_eq!(cut_depth(0.0), "above the ground");
        assert_eq!(cut_depth(-3.0), "above the ground");
    }
}
