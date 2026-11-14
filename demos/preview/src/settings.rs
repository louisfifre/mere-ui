//! The 43 settings of the pack's Tectonic config that the page knows, in the order of its `known` table
//! (preview.html lines 660-677): which group each goes in, its label (the page's `text.settings`, lines
//! 532-574), its range and step, and the pack's value.
//!
//! The pack values are those of the run "tectonic", the run cap-1, in `C:\Games\preview\out\run.json`. They are
//! copied here once and the file is never read at runtime: the live values come with a later step.

use nui_ui::NumberSpec;

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    Number(NumberSpec),
    /// A setting with no range: shown as on or off.
    Switch,
}

#[derive(Copy, Clone, Debug)]
pub struct Setting {
    pub key: &'static str,
    /// Index in `text::GROUPS`.
    pub group: usize,
    pub label: &'static str,
    pub kind: Kind,
    pub pack: f64,
}

/// The values of the pack, in `SETTINGS` order.
pub const SETTINGS: [Setting; 43] = [
    Setting { key: "temperature_scale", group: 0, label: "Temperature scale", kind: Kind::Number(NumberSpec { min: 0.03, max: 0.5, step: 0.01 }), pack: 0.05 },
    Setting { key: "vegetation_scale", group: 0, label: "Vegetation scale", kind: Kind::Number(NumberSpec { min: 0.03, max: 0.5, step: 0.01 }), pack: 0.05 },
    Setting { key: "temperature_multiplier", group: 0, label: "Temperature range", kind: Kind::Number(NumberSpec { min: 0.5, max: 1.5, step: 0.05 }), pack: 0.9 },
    Setting { key: "vegetation_multiplier", group: 0, label: "Vegetation range", kind: Kind::Number(NumberSpec { min: 0.5, max: 1.5, step: 0.05 }), pack: 1.1 },
    Setting { key: "temperature_offset", group: 0, label: "Temperature offset", kind: Kind::Number(NumberSpec { min: -0.5, max: 0.5, step: 0.05 }), pack: 0.1 },
    Setting { key: "vegetation_offset", group: 0, label: "Vegetation offset", kind: Kind::Number(NumberSpec { min: -0.5, max: 0.5, step: 0.05 }), pack: 0.0 },
    Setting { key: "vertical_scale", group: 1, label: "Vertical scale", kind: Kind::Number(NumberSpec { min: 0.5, max: 3.0, step: 0.025 }), pack: 1.5 },
    Setting { key: "elevation_boost", group: 1, label: "Elevation boost", kind: Kind::Number(NumberSpec { min: 0.0, max: 1.0, step: 0.05 }), pack: 0.0 },
    Setting { key: "flat_terrain_skew", group: 1, label: "Flat against plateaus", kind: Kind::Number(NumberSpec { min: -0.5, max: 0.5, step: 0.05 }), pack: -0.35 },
    Setting { key: "erosion_scale", group: 1, label: "Erosion scale", kind: Kind::Number(NumberSpec { min: 0.05, max: 0.5, step: 0.01 }), pack: 0.1 },
    Setting { key: "ridge_scale", group: 1, label: "Ridge scale", kind: Kind::Number(NumberSpec { min: 0.05, max: 0.5, step: 0.01 }), pack: 0.1 },
    Setting { key: "rolling_hills", group: 1, label: "Rolling hills", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "jungle_pillars", group: 1, label: "Jungle pillars", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "ultrasmooth", group: 1, label: "Ultrasmooth", kind: Kind::Switch, pack: 0.0 },
    Setting { key: "alternate_erosion_scaling", group: 1, label: "Alternate erosion scaling", kind: Kind::Switch, pack: 0.0 },
    Setting { key: "min_y", group: 1, label: "Bottom of the world", kind: Kind::Number(NumberSpec { min: -256.0, max: 0.0, step: 16.0 }), pack: -64.0 },
    Setting { key: "max_y", group: 1, label: "Top of the world", kind: Kind::Number(NumberSpec { min: 256.0, max: 512.0, step: 16.0 }), pack: 320.0 },
    Setting { key: "continents_scale", group: 2, label: "Continent scale", kind: Kind::Number(NumberSpec { min: 0.01, max: 0.3, step: 0.01 }), pack: 0.05 },
    Setting { key: "ocean_offset", group: 2, label: "Land against ocean", kind: Kind::Number(NumberSpec { min: -1.0, max: -0.2, step: 0.05 }), pack: -0.6 },
    Setting { key: "alternate_continents_scaling", group: 2, label: "Alternate continents scaling", kind: Kind::Switch, pack: 0.0 },
    Setting { key: "ocean_depth", group: 2, label: "Ocean depth", kind: Kind::Number(NumberSpec { min: -1.0, max: 0.0, step: 0.01 }), pack: -0.22 },
    Setting { key: "deep_ocean_depth", group: 2, label: "Deep ocean depth", kind: Kind::Number(NumberSpec { min: -1.0, max: 0.0, step: 0.01 }), pack: -0.45 },
    Setting { key: "enabled", group: 2, label: "Islands", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "noise_scale", group: 2, label: "Island scale", kind: Kind::Number(NumberSpec { min: 0.02, max: 0.5, step: 0.01 }), pack: 0.11 },
    Setting { key: "noise_multiplier", group: 2, label: "Island contrast", kind: Kind::Number(NumberSpec { min: 0.0, max: 2.0, step: 0.05 }), pack: 1.0 },
    Setting { key: "noise_offset", group: 2, label: "Island amount", kind: Kind::Number(NumberSpec { min: -1.0, max: 1.0, step: 0.05 }), pack: 0.0 },
    Setting { key: "cheese_enabled", group: 3, label: "Cheese caves", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "cheese_additive", group: 3, label: "Cheese caves size", kind: Kind::Number(NumberSpec { min: -0.5, max: 1.0, step: 0.01 }), pack: 0.27 },
    Setting { key: "spaghetti_enabled", group: 3, label: "Spaghetti caves", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "noodle_enabled", group: 3, label: "Noodle caves", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "noodle_additive", group: 3, label: "Noodle caves size", kind: Kind::Number(NumberSpec { min: -0.5, max: 0.5, step: 0.005 }), pack: -0.075 },
    Setting { key: "depth_cutoff_start", group: 3, label: "Caves close to the ground", kind: Kind::Number(NumberSpec { min: 0.0, max: 1.0, step: 0.05 }), pack: 0.1 },
    Setting { key: "depth_cutoff_size", group: 3, label: "Caves closing", kind: Kind::Number(NumberSpec { min: 0.0, max: 0.5, step: 0.05 }), pack: 0.1 },
    Setting { key: "lava_tunnels", group: 3, label: "Lava tunnels", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "underground_rivers", group: 3, label: "Underground rivers", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "carvers_enabled", group: 3, label: "Old caves and ravines", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "ore_fix", group: 3, label: "Ore fix", kind: Kind::Switch, pack: 0.0 },
    Setting { key: "mod_enabled", group: 1, label: "Tectonic", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "snow_start_offset", group: 4, label: "Snow start", kind: Kind::Number(NumberSpec { min: 0.0, max: 256.0, step: 8.0 }), pack: 128.0 },
    Setting { key: "monument_offset", group: 4, label: "Ocean monuments", kind: Kind::Number(NumberSpec { min: -64.0, max: 0.0, step: 1.0 }), pack: -30.0 },
    Setting { key: "river_ice", group: 4, label: "Ice on underground rivers", kind: Kind::Switch, pack: 0.0 },
    Setting { key: "river_lanterns", group: 4, label: "Lanterns over underground rivers", kind: Kind::Switch, pack: 1.0 },
    Setting { key: "remove_frozen_ocean_ice", group: 4, label: "No ice on frozen oceans", kind: Kind::Switch, pack: 0.0 },
];

/// Indices into `SETTINGS` for each group, in `SETTINGS` order (the page appends each setting to its group
/// as it walks `known`, so "Tectonic" is the last of the relief group).
pub fn grouped() -> [Vec<usize>; 5] {
    let mut out: [Vec<usize>; 5] = Default::default();
    for (i, s) in SETTINGS.iter().enumerate() {
        out[s.group].push(i);
    }
    out
}

/// The pack's values, in `SETTINGS` order.
pub fn pack_values() -> [f64; 43] {
    let mut v = [0.0; 43];
    for (i, s) in SETTINGS.iter().enumerate() {
        v[i] = s.pack;
    }
    v
}
