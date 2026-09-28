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
    pub water: Material,
}

/// Genera un terreno de `width` x `depth` cubos con relieve procedural,
/// columnas rellenas (sin huecos), bordes sólidos hasta y=0 estilo isla flotante,
/// un claro plano para el campamento, un camino de tierra y un arroyo hundido con agua reflectiva.
pub fn generate_terrain(
    width: i32,
    depth: i32,
    max_height: i32,
    scale: f64,
    seed: u32,
    palette: &TerrainPalette,
) -> Vec<Cube> {
    let noise_fn: Fbm<OpenSimplex> = Fbm::new(seed);
    let total_cols = (width * depth) as usize;

    let mut height_map = vec![2i32; total_cols];
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

            // 1. Arroyo: franja de 1-2 bloques de ancho que cruza el mapa por un lado del claro
            // Ubicado en z = 12 y 13 (wz en [4.0, 6.0]), hundido 1 bloque respecto al terreno
            let in_stream = z == 12 || z == 13;

            // 2. Camino de tierra: franja de dirt de ~2 bloques de ancho desde el borde sur (z=15)
            // hasta el claro de la fogata (z <= 10), ligeramente curva.
            let in_path = if z >= 14 {
                x == 8 || x == 9
            } else if z == 11 {
                x == 7 || x == 8
            } else if z == 10 || z == 9 {
                x == 7 || x == 8
            } else {
                false
            };

            if in_stream {
                // El arroyo está hundido 1 bloque respecto al terreno/claro (y=1 vs y=2)
                height_map[idx] = 1;
                // Fondo de dirt con toques de stone
                let bed_mat = if (x + z) % 3 == 0 { palette.stone } else { palette.dirt };
                material_map[idx] = bed_mat;
                is_water_map[idx] = true;
            } else if in_path {
                // Camino de tierra: a nivel del claro (y=2), sin textura de pasto
                height_map[idx] = 2;
                material_map[idx] = palette.dirt;
            } else if dist_center <= 3.5 {
                // Claro aplanado de ~5x5 alrededor de la fogata (y=2, superficie en Y=3.0)
                height_map[idx] = 2;
                material_map[idx] = palette.grass;
            } else {
                // Relieve con colinas suaves de 2-3 bloques de desnivel
                let nx = cx as f64 * scale;
                let nz = cz as f64 * scale;
                let n = noise_fn.get([nx, nz]); // [-1.0, 1.0]

                // Transición suave desde el claro hacia las colinas exteriores
                let blend = ((dist_center - 3.5) / 2.5).clamp(0.0, 1.0);
                // Altura base 2 + colinas de 2-3 bloques adicionales
                let hill = (n * 0.5 + 0.5) * 2.8;
                let y = (2.0 + blend as f64 * hill).round() as i32;
                let y_clamped = y.clamp(2, max_height.max(5));

                height_map[idx] = y_clamped;
                if y_clamped >= 4 {
                    // Picos y crestas de roca
                    material_map[idx] = palette.stone;
                } else {
                    material_map[idx] = palette.grass;
                }
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
                // Altura mínima entre los 4 vecinos ortogonales
                let n_left  = height_map[(z * width + (x - 1)) as usize];
                let n_right = height_map[(z * width + (x + 1)) as usize];
                let n_up    = height_map[((z - 1) * width + x) as usize];
                let n_down  = height_map[((z + 1) * width + x) as usize];
                let min_neighbor = n_left.min(n_right).min(n_up).min(n_down);

                // Mínimo 1 cubo debajo: (y_top - 1).min(min_neighbor), no menor a 0
                (y_top - 1).min(min_neighbor).max(0)
            };

            let wx = x as f32 - width as f32 * 0.5;
            let wz = z as f32 - depth as f32 * 0.5;

            // Rellenar la columna desde y_bottom hasta y_top
            for y in y_bottom..=y_top {
                let material = if y == y_top {
                    material_map[idx]
                } else if y == y_top - 1 {
                    // Capa inmediata subsuperficial: dirt (o stone si la superficie es roca)
                    if material_map[idx].albedo == palette.stone.albedo {
                        palette.stone
                    } else {
                        palette.dirt
                    }
                } else {
                    // Estratos inferiores y acantilados profundos: stone
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
                // El fondo del arroyo está en y=1 (su cara superior en Y=2.0).
                // El agua plana se coloca sobre el fondo (Y=2.0 a 2.1).
                cubes.push(Cube::new_box(
                    Vector3::new(wx, 2.0, wz),
                    Vector3::new(1.0, 0.1, 1.0),
                    palette.water,
                ));
            }
        }
    }

    cubes
}
