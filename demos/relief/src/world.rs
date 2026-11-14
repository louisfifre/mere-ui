//! A run's output folder, read: `preview.json` for the numbers, `heights.bin` and `biomes.bin` for the grid.

use crate::json::Json;
use std::path::Path;

pub struct World {
    pub seed: String,
    /// Points on a side of the square grid.
    pub samples: usize,
    /// Blocks between two points.
    pub step: f32,
    pub sea_level: i32,
    /// One height per point, row by row from the north-west corner.
    pub heights: Vec<i16>,
    /// Biomes the grid actually uses, out of `biome_count` named in `preview.json`.
    pub biomes_used: usize,
    pub biome_count: usize,
}

impl World {
    pub fn load(dir: &Path) -> Result<World, String> {
        let read = |name: &str| std::fs::read(dir.join(name)).map_err(|e| format!("{}: {e}", dir.join(name).display()));
        let text = String::from_utf8(read("preview.json")?).map_err(|_| "preview.json is not UTF-8".to_string())?;
        let json = Json::parse(&text)?;
        let number = |key: &str| json.get(key).and_then(Json::num).ok_or_else(|| format!("preview.json has no number \"{key}\""));

        let samples = number("samples")? as usize;
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
        Ok(World {
            seed: json.get("seed").and_then(Json::text).unwrap_or_default(),
            samples,
            step: number("step")? as f32,
            sea_level: number("seaLevel")? as i32,
            heights: heights_raw.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect(),
            biomes_used: biomes_raw
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect::<std::collections::BTreeSet<u16>>()
                .len(),
            biome_count: json.get("biomes").and_then(Json::array).map_or(0, <[Json]>::len),
        })
    }

    /// Blocks across the whole area: the page's `n * step`, which the camera and the water are sized by.
    pub fn width(&self) -> f32 {
        self.samples as f32 * self.step
    }

    /// The ground's height under (x, z) of the centred square, between the four nearest points as the
    /// mesh's triangles do closely enough; None outside the square.
    pub fn ground_at(&self, x: f32, z: f32) -> Option<f32> {
        let n = self.samples;
        let half = (n - 1) as f32 * self.step / 2.0;
        let (c, r) = ((x + half) / self.step, (z + half) / self.step);
        let top = (n - 1) as f32;
        if !(0.0..=top).contains(&c) || !(0.0..=top).contains(&r) {
            return None;
        }
        let (c0, r0) = ((c.floor() as usize).min(n - 2), (r.floor() as usize).min(n - 2));
        let (fc, fr) = (c - c0 as f32, r - r0 as f32);
        let h = |col: usize, row: usize| self.heights[row * n + col] as f32;
        let north = h(c0, r0) + (h(c0 + 1, r0) - h(c0, r0)) * fc;
        let south = h(c0, r0 + 1) + (h(c0 + 1, r0 + 1) - h(c0, r0 + 1)) * fc;
        Some(north + (south - north) * fr)
    }

    pub fn height_range(&self) -> (i16, i16) {
        let lo = self.heights.iter().copied().min().unwrap_or(0);
        let hi = self.heights.iter().copied().max().unwrap_or(0);
        (lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_real_out_folder_when_it_is_there() {
        let dir = Path::new(r"C:\Games\preview\out");
        if !dir.join("preview.json").exists() {
            return;
        }
        let w = World::load(dir).unwrap();
        assert_eq!(w.samples, 256);
        assert_eq!(w.heights.len(), 256 * 256);
        assert_eq!(w.step, 16.0);
        let (lo, hi) = w.height_range();
        assert!(lo < w.sea_level as i16 && hi > w.sea_level as i16, "{lo}..{hi}");
        assert!(w.biomes_used > 1 && w.biomes_used <= w.biome_count);
    }

    #[test]
    fn a_missing_folder_is_an_error_not_a_panic() {
        assert!(World::load(Path::new(r"C:\Games\no\such\folder")).is_err());
    }
}
