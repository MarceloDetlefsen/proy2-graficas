use noise::{Fbm, NoiseFn, OpenSimplex};
use raylib::prelude::Vector3;
use crate::cube::Cube;
use crate::material::Material;

pub struct TerrainPalette {
    pub grass: Material,
    pub dirt: Material,
    pub stone: Material,
    pub sand: Material,
    pub snow: Material,
}

/// Genera un terreno de `width` x `depth` cubos (mínimo 16x16 según rúbrica),
/// con altura y material determinados por ruido Fbm/OpenSimplex.
pub fn generate_terrain(
    width: i32,
    depth: i32,
    max_height: i32,
    scale: f64,
    seed: u32,
    palette: &TerrainPalette,
) -> Vec<Cube> {
    let noise_fn: Fbm<OpenSimplex> = Fbm::new(seed);
    let mut cubes = Vec::with_capacity((width * depth) as usize);

    for x in 0..width {
        for z in 0..depth {
            let nx = x as f64 * scale;
            let nz = z as f64 * scale;
            let h = noise_fn.get([nx, nz]); // rango [-1, 1]
            let y = (((h * 0.5 + 0.5) * max_height as f64).floor() as i32)
                .clamp(0, max_height);

            let material = if y >= max_height {
                palette.stone
            } else if y >= 1 {
                palette.grass
            } else {
                palette.dirt
            };

            cubes.push(Cube::new(
                Vector3::new(
                    x as f32 - width as f32 * 0.5,
                    y as f32,
                    z as f32 - depth as f32 * 0.5,
                ),
                1.0,
                material,
            ));
        }
    }

    cubes
}
