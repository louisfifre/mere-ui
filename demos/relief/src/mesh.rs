//! The terrain as a mesh, and the sheet of water over it. One unit is one block, heights included.

use crate::world::World;
use nui_base::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// One vertex per sample up to 512 a side; a wider grid gives one vertex every other sample, which the eye
/// does not miss at that distance (the page does the same). The square is centred on the origin, so the
/// camera can turn around it.
pub fn land(world: &World) -> Mesh {
    let n = world.samples;
    let skip = n.div_ceil(512);
    let m = (n - 1) / skip + 1;
    let side = ((m - 1) * skip) as f32 * world.step;
    let mut vertices = Vec::with_capacity(m * m);
    for row in 0..m {
        for col in 0..m {
            let (sc, sr) = (col * skip, row * skip);
            vertices.push(Vertex {
                pos: [
                    (sc as f32) * world.step - side / 2.0,
                    world.heights[sr * n + sc] as f32,
                    (sr as f32) * world.step - side / 2.0,
                ],
                // The middle of the sample's texel, so each cell of the map lies under its own vertex.
                uv: [(sc as f32 + 0.5) / n as f32, (sr as f32 + 0.5) / n as f32],
            });
        }
    }
    let mut indices = Vec::with_capacity((m - 1) * (m - 1) * 6);
    for row in 0..m - 1 {
        for col in 0..m - 1 {
            let a = (row * m + col) as u32;
            let (b, c, d) = (a + 1, a + m as u32, a + m as u32 + 1);
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    Mesh { vertices, indices }
}

/// Half a block under the sea level: ground at that level is land, and must not lie in the sheet of water.
pub fn water(world: &World) -> Mesh {
    let half = world.width() / 2.0;
    let y = world.sea_level as f32 - 0.5;
    let v = |x: f32, z: f32| Vertex { pos: [x, y, z], uv: [0.0, 0.0] };
    Mesh {
        vertices: vec![v(-half, -half), v(half, -half), v(-half, half), v(half, half)],
        indices: vec![0, 2, 1, 1, 2, 3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(samples: usize) -> World {
        World {
            seed: String::new(),
            samples,
            step: 16.0,
            sea_level: 63,
            heights: (0..samples * samples).map(|i| (i % 100) as i16).collect(),
            biome_count: 1,
            biome_total: None,
        }
    }

    #[test]
    fn land_has_a_vertex_per_sample_and_two_triangles_per_cell() {
        let m = land(&world(256));
        assert_eq!(m.vertices.len(), 256 * 256);
        assert_eq!(m.indices.len(), 255 * 255 * 6);
        assert!(m.indices.iter().all(|&i| (i as usize) < m.vertices.len()));
        // Centred on the origin, 255 steps across.
        assert_eq!(m.vertices[0].pos[0], -255.0 * 16.0 / 2.0);
        assert_eq!(m.vertices[255].pos[0], 255.0 * 16.0 / 2.0);
        assert_eq!(m.vertices[1].pos[1], 1.0);
    }

    #[test]
    fn a_wide_grid_is_thinned() {
        let m = land(&world(1024));
        assert_eq!(m.vertices.len(), 512 * 512);
    }

    #[test]
    fn water_is_one_quad_below_the_sea_level() {
        let m = water(&world(256));
        assert_eq!(m.indices.len(), 6);
        assert!(m.vertices.iter().all(|v| v.pos[1] == 62.5));
        assert_eq!(m.vertices[3].pos[0], 2048.0);
    }
}
