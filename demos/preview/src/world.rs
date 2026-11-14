//! A run's output folder, read the way the page reads it (preview.html `loadWorld` 1142-1167, `loadStructures`
//! 1171-1185, `measure` 1255-1322 for the level and flat land, `loadSection` 1975-1982 for the cut).

use std::path::Path;

use crate::colour::{biome_colour, structure_colour, underground};
use crate::json::Json;

#[derive(Clone, Debug)]
pub struct Biome {
    pub id: String,
    pub temperature: f64,
}

/// Where each structure would start, as the generator places them (structures.json).
#[derive(Clone, Debug)]
pub struct Structures {
    pub ids: Vec<String>,
    pub colours: Vec<[u8; 3]>,
    /// Built under the ground rather than on the surface.
    pub under: Vec<bool>,
    /// How many points each kind has.
    pub count: Vec<u32>,
    /// (kind, x, z) of every start whose kind is in range.
    pub points: Vec<(usize, f64, f64)>,
    pub by_kind: Vec<Vec<(f64, f64)>>,
    /// The kinds that have points, the most frequent first: the rare ones are drawn over them.
    pub order: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: String,
    /// Points on a side of the square grid.
    pub samples: usize,
    /// Blocks between two points.
    pub step: f64,
    pub origin_x: f64,
    pub origin_z: f64,
    pub sea_level: f64,
    pub heights: Vec<i16>,
    pub biome_at: Vec<u16>,
    pub biomes: Vec<Biome>,
    /// One colour per biome (`colour::biome_colour`).
    pub colours: Vec<[u8; 3]>,
    /// 1 where the land is flat, 0 elsewhere.
    pub flat_at: Vec<u8>,
    pub structures: Option<Structures>,
    /// The z of the first cut of the section, when the page would show its ticks: cuts exist and ores.json parses.
    pub cut_z: Option<f64>,
}

impl World {
    /// Blocks across the whole area.
    pub fn span(&self) -> f64 {
        self.samples as f64 * self.step
    }

    /// A map stopped at a coarser detail: level land is told 16 blocks apart, so some views are not offered.
    pub fn coarse(&self) -> bool {
        self.step > 16.0
    }

    pub fn load(dir: &Path) -> Result<World, String> {
        let read = |name: &str| std::fs::read(dir.join(name)).map_err(|e| format!("{}: {e}", dir.join(name).display()));
        let text = String::from_utf8(read("preview.json")?).map_err(|_| "preview.json is not UTF-8".to_string())?;
        let json = Json::parse(&text)?;
        let number = |key: &str| json.get(key).and_then(Json::num).ok_or_else(|| format!("preview.json has no number \"{key}\""));

        let samples = number("samples")? as usize;
        let step = number("step")?;
        let n = samples * samples;
        let heights_raw = read("heights.bin")?;
        let biomes_raw = read("biomes.bin")?;
        if samples < 2 || heights_raw.len() != n * 2 || biomes_raw.len() != n * 2 {
            return Err(format!(
                "{samples} samples a side need {} bytes in heights.bin and biomes.bin, found {} and {}",
                n * 2,
                heights_raw.len(),
                biomes_raw.len()
            ));
        }
        let mut biomes = Vec::new();
        for b in json.get("biomes").and_then(Json::array).ok_or("preview.json has no \"biomes\" list")? {
            let id = b.get("id").and_then(Json::text).ok_or("a biome of preview.json has no id")?;
            biomes.push(Biome { id, temperature: b.get("temperature").and_then(Json::num).unwrap_or(0.0) });
        }
        let heights: Vec<i16> = heights_raw.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect();
        let biome_at: Vec<u16> = biomes_raw.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect();
        if let Some((i, &b)) = biome_at.iter().enumerate().find(|(_, b)| **b as usize >= biomes.len()) {
            return Err(format!("biomes.bin point {i} names biome {b}, but preview.json lists {}", biomes.len()));
        }
        let sea_level = number("seaLevel")?;
        let colours = biomes.iter().map(|b| biome_colour(&b.id)).collect();
        let flat_at = flat_land(&heights, samples, sea_level);
        Ok(World {
            seed: json.get("seed").and_then(Json::text).unwrap_or_default(),
            samples,
            step,
            origin_x: number("originX")?,
            origin_z: number("originZ")?,
            sea_level,
            heights,
            biome_at,
            biomes,
            colours,
            flat_at,
            structures: load_structures(dir),
            cut_z: load_cut(dir),
        })
    }
}

/// `measure`, lines 1265-1293: the level land (the ground moves by 2 blocks at most to each neighbour point,
/// a neighbour under water not counting), then the flat land, level ground among level ground: at least 5 of
/// the 9 points around.
fn flat_land(heights: &[i16], n: usize, sea: f64) -> Vec<u8> {
    let total = n * n;
    let mut level = vec![0u8; total];
    for i in 0..total {
        let y = heights[i] as f64;
        if y < sea {
            continue;
        }
        let x = i % n;
        let gap = |j: usize| if heights[j] as f64 >= sea { (y - heights[j] as f64).abs() } else { 0.0 };
        let d = [
            if x > 0 { gap(i - 1) } else { 0.0 },
            if x < n - 1 { gap(i + 1) } else { 0.0 },
            if i >= n { gap(i - n) } else { 0.0 },
            if i < total - n { gap(i + n) } else { 0.0 },
        ]
        .into_iter()
        .fold(0.0, f64::max);
        if d <= 2.0 {
            level[i] = 1;
        }
    }
    let mut flat = vec![0u8; total];
    for i in 0..total {
        if (heights[i] as f64) < sea {
            continue;
        }
        let x = i % n;
        let mut around = 0;
        for dz in [-(n as isize), 0, n as isize] {
            for dx in -1isize..=1 {
                let j = i as isize + dz + dx;
                if j >= 0 && (j as usize) < total && x as isize + dx >= 0 && (x as isize + dx) < n as isize {
                    around += level[j as usize] as u32;
                }
            }
        }
        if around >= 5 {
            flat[i] = 1;
        }
    }
    flat
}

fn read_json(dir: &Path, name: &str) -> Option<Json> {
    let text = std::fs::read(dir.join(name)).ok()?;
    Json::parse(std::str::from_utf8(&text).ok()?).ok()
}

/// Lines 1171-1185. Nothing when the file is missing or lacks either array: older previews have none.
fn load_structures(dir: &Path) -> Option<Structures> {
    let data = read_json(dir, "structures.json")?;
    let kinds_json = data.get("structures")?.array()?;
    let points_json = data.get("points")?.array()?;
    let kinds = kinds_json.len();
    let mut ids = Vec::new();
    let mut colours = Vec::new();
    let mut under = Vec::new();
    for s in kinds_json {
        let id = s.get("id").and_then(Json::text).unwrap_or_default();
        let set = s.get("set").and_then(Json::text);
        let step = s.get("step").and_then(Json::text);
        colours.push(structure_colour(&id, set.as_deref()));
        under.push(underground(&id, step.as_deref()));
        ids.push(id);
    }
    let mut count = vec![0u32; kinds];
    let mut points = Vec::new();
    let mut by_kind = vec![Vec::new(); kinds];
    for p in points_json {
        let Some([k, x, z]) = p.array().and_then(|a| Some([a.first()?.num()?, a.get(1)?.num()?, a.get(2)?.num()?])) else { continue };
        // Line 1175: a kind outside the list is dropped.
        if k < 0.0 || k >= kinds as f64 || k.fract() != 0.0 {
            continue;
        }
        let k = k as usize;
        count[k] += 1;
        by_kind[k].push((x, z));
        points.push((k, x, z));
    }
    let mut order: Vec<usize> = (0..kinds).filter(|&i| count[i] > 0).collect();
    // sort_by is stable, as Array.prototype.sort is.
    order.sort_by(|&a, &b| count[b].cmp(&count[a]));
    Some(Structures { ids, colours, under, count, points, by_kind, order })
}

/// Line 1978: the section exists when it has cuts and the ores file is there; the ticks sit at the first cut.
fn load_cut(dir: &Path) -> Option<f64> {
    let meta = read_json(dir, "section.json")?;
    read_json(dir, "ores.json")?;
    meta.get("cuts")?.array()?.first()?.get("z")?.num()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_folder_is_an_error_not_a_panic() {
        assert!(World::load(Path::new(r"C:\Games\no\such\folder")).is_err());
    }

    #[test]
    fn reads_the_real_out_folder_when_it_is_there() {
        let dir = Path::new(r"C:\Games\preview\out");
        if !dir.join("preview.json").exists() {
            println!("skipped: no C:\\Games\\preview\\out");
            return;
        }
        let w = World::load(dir).unwrap();
        assert_eq!((w.samples, w.step, w.origin_x, w.origin_z, w.sea_level), (256, 16.0, -2048.0, -2048.0, 63.0));
        assert_eq!(w.span(), 4096.0);
        assert!(!w.coarse());
        assert_eq!((w.biomes.len(), w.colours.len()), (25, 25));
        let s = w.structures.as_ref().expect("structures.json is there");
        assert_eq!((s.ids.len(), s.points.len()), (75, 655));
        assert!(s.order.windows(2).all(|p| s.count[p[0]] >= s.count[p[1]]));
        assert_eq!(w.cut_z, Some(-1504.0));
        assert!(w.flat_at.iter().any(|&f| f == 1));
    }

    #[test]
    fn a_flat_plain_is_flat_and_a_cliff_is_not() {
        // 4 x 4: the left half at 70, a wall to 90 on the right half; the sea is 63.
        let mut h = vec![70i16; 16];
        for i in 0..16 {
            if i % 4 >= 2 {
                h[i] = 90;
            }
        }
        let f = flat_land(&h, 4, 63.0);
        // Points on the cliff's two sides have a neighbour 20 away: not level, so not flat.
        assert_eq!(f[1], 0);
        assert_eq!(f[2], 0);
        // A corner of the high ground has 4 level points around it (itself and 3), which is not 5.
        assert_eq!(f[3], 0);
        // The sea is never flat land.
        assert_eq!(flat_land(&[10; 16], 4, 63.0), vec![0; 16]);
        // Open flat ground: an inner point has all 9 around it level.
        assert_eq!(flat_land(&[70; 16], 4, 63.0)[5], 1);
    }
}
