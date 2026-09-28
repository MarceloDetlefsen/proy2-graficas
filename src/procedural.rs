use noise::{Fbm, NoiseFn, OpenSimplex};
use raylib::prelude::Vector3;
use crate::cube::Cube;
use crate::material::Material;

#[allow(dead_code)]
pub struct TerrainPalette {
    pub grass: Material,
    pub dirt: Material,
    pub stone: Material,
    pub sand: Material,
    pub snow: Material,
    pub water: Material,
    pub straw: Material,
}

/// Genera un terreno de `width` x `depth` cubos con relieve procedural,
/// columnas rellenas (sin huecos), bordes sólidos hasta y=0 estilo isla flotante,
/// un claro plano para el campamento con círculo de paja, un camino de tierra y un arroyo hundido con agua reflectiva.
pub fn generate_terrain(
    width: i32,
    depth: i32,
    _max_height: i32,
    scale: f64,
    seed: u32,
    palette: &TerrainPalette,
) -> Vec<Cube> {
    let noise_fn: Fbm<OpenSimplex> = Fbm::new(seed);
    let total_cols = (width * depth) as usize;

    let mut height_map = vec![3i32; total_cols];
    let mut material_map = vec![palette.grass; total_cols];
    let mut is_water_map = vec![false; total_cols];

    // Paso 1: Determinar altura superficial (y_top), material superficial y si es arroyo
    for z in 0..depth {
        for x in 0..width {
            let idx = (z * width + x) as usize;
            let wx = x as f32 - width as f32 * 0.5;
            let wz = z as f32 - depth as f32 * 0.5;
            let cx = wx + 0.5;
            let cz = wz + 0.5;
            let dist_center = (cx * cx + cz * cz).sqrt();

            // 1. Arroyo: franja que cruza en z in [4.0, 6.0] (wz = 4 o 5 en coordenadas centradas)
            // Hundido 1 bloque respecto al claro (y=2, cara superior en Y=3.0 vs claro en Y=4.0)
            let in_stream = wz >= 4.0 && wz < 6.0;

            // 2. Camino de tierra: franja de dirt de ~2 bloques de ancho desde el sur
            // cruzando el puente hacia el claro
            let in_path = cz > 3.5 && (cx >= -0.6 && cx <= 1.6);

            // 3. Claro plano a y = 4.0 (y_top = 3) en x in [-5.0, 5.0], z in [-5.0, 3.5]
            let in_clearing = cx >= -5.0 && cx <= 5.0 && cz >= -5.0 && cz <= 3.5;

            let is_stream_rock = (cx >= -3.5 && cx <= -2.5 && cz >= 3.5 && cz <= 4.0)
                || (cx >= 3.0 && cx <= 4.0 && cz >= 3.5 && cz <= 4.0);

            if in_stream {
                // Fondo del arroyo hundido 1 bloque (y=2, cara superior en Y=3.0)
                height_map[idx] = 2;
                let bed_mat = if (x + z) % 3 == 0 { palette.stone } else { palette.dirt };
                material_map[idx] = bed_mat;
                is_water_map[idx] = true;
            } else if in_path {
                height_map[idx] = 3;
                material_map[idx] = palette.dirt;
            } else if in_clearing {
                height_map[idx] = 3;
                if dist_center <= 2.5 {
                    // Círculo de paja alrededor de la fogata
                    material_map[idx] = palette.straw;
                } else {
                    material_map[idx] = palette.grass;
                }
            } else if is_stream_rock {
                height_map[idx] = 3;
                material_map[idx] = palette.stone;
            } else if cz > 3.5 {
                // Lado frontal fuera del claro: plano a y=4.0 (y_top=3, máximo +0 bloques)
                // para que ningún bloque tape la vista desde el encuadre inicial ni elevaciones bajas
                height_map[idx] = 3;
                material_map[idx] = palette.grass;
            } else {
                // Relieve suave fuera del claro de máximo +1 bloque (y_top = 3 o 4, cara sup Y=4.0 a 5.0)
                let dx = (cx.abs() - 5.0).max(0.0);
                let dz = if cz < -5.0 { -5.0 - cz } else { 0.0 };
                let dist_from_clearing = (dx * dx + dz * dz).sqrt();
                let blend = (dist_from_clearing / 2.0).clamp(0.0, 1.0);

                let nx = cx as f64 * scale;
                let nz = cz as f64 * scale;
                let n = (noise_fn.get([nx, nz]) * 0.5 + 0.5) as f32; // [0, 1]

                // Relieve suave de máximo +1 bloque
                let hill = (blend * n).round() as i32;
                let y = (3 + hill).clamp(3, 4);

                height_map[idx] = y;
                material_map[idx] = palette.grass;
            }
        }
    }

    // Paso 2: Generar columnas rellenas
    // En cada columna, además del cubo superior, agregá cubos hacia abajo SOLO hasta
    // la altura del vecino más bajo (mínimo 1 cubo debajo).
    // Los bordes exteriores del terreno deben bajar hasta y=0 (estilo isla flotante).
    let mut cubes = Vec::with_capacity(total_cols * 3);

    for z in 0..depth {
        for x in 0..width {
            let idx = (z * width + x) as usize;
            let y_top = height_map[idx];
            let is_border = x == 0 || x == width - 1 || z == 0 || z == depth - 1;

            let y_bottom = if is_border {
                0
            } else {
                let n_left  = height_map[(z * width + (x - 1)) as usize];
                let n_right = height_map[(z * width + (x + 1)) as usize];
                let n_up    = height_map[((z - 1) * width + x) as usize];
                let n_down  = height_map[((z + 1) * width + x) as usize];
                let min_neighbor = n_left.min(n_right).min(n_up).min(n_down);
                (y_top - 1).min(min_neighbor).max(0)
            };

            let wx = x as f32 - width as f32 * 0.5;
            let wz = z as f32 - depth as f32 * 0.5;

            // Rellenar la columna desde y_bottom hasta y_top
            for y in y_bottom..=y_top {
                let material = if y == y_top {
                    material_map[idx]
                } else if y == y_top - 1 {
                    if material_map[idx].albedo == palette.stone.albedo {
                        palette.stone
                    } else {
                        palette.dirt
                    }
                } else {
                    palette.stone
                };

                cubes.push(Cube::new(
                    Vector3::new(wx, y as f32, wz),
                    1.0,
                    material,
                ));
            }

            // Capa de agua plana a ras (cubo achatado ~0.1) si es arroyo
            if is_water_map[idx] {
                // Fondo en y=2 (cara superior Y=3.0), agua colocada en Y=3.0 a 3.1
                cubes.push(Cube::new_box(
                    Vector3::new(wx, 3.0, wz),
                    Vector3::new(1.0, 0.1, 1.0),
                    palette.water,
                ));
            }
        }
    }

    cubes
}
