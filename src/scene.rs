use raylib::prelude::Vector3;
use crate::cube::Cube;
use crate::material::SceneMaterials;
use crate::light::Light;
use crate::skybox::Skybox;
use crate::billboard::Billboard;
use crate::procedural;

pub struct Scene {
    pub cubes: Vec<Cube>,
    pub billboards: Vec<Billboard>,
    pub ground_sprites: Vec<crate::billboard::GroundSprite>,
    pub lights: Vec<Light>,
    pub skybox: Skybox,
    pub textures: crate::texture::TextureManager,
    pub grid: crate::grid::VoxelGrid,
    pub use_normal_maps: bool,
    pub camera_forward: Vector3,
    pub camera_right: Vector3,
    pub camera_up: Vector3,
}

/// Calcula la altura real del suelo en las coordenadas (x, z) buscando el bloque
/// de terreno superior en esa columna.
pub fn terrain_height_at(cubes: &[Cube], x: f32, z: f32) -> f32 {
    let mut max_y = 0.0f32;
    let mut found = false;
    for cube in cubes {
        if cube.min.x <= x && x < cube.max.x && cube.min.z <= z && z < cube.max.z {
            if !found || cube.max.y > max_y {
                max_y = cube.max.y;
                found = true;
            }
        }
    }
    if found { max_y } else { 3.0 }
}

/// Lee las dimensiones de un archivo PNG desde su cabecera IHDR y devuelve su relación de aspecto (width / height).
pub fn get_png_aspect_ratio(path: &str) -> f32 {
    if let Ok(bytes) = std::fs::read(path) {
        if bytes.len() >= 24 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
            let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as f32;
            let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as f32;
            if h > 0.0 {
                return w / h;
            }
        }
    }
    0.5
}

impl Scene {
    /// Arma el diorama: "Campamento nocturno" (16x16 mínimo de terreno).
    pub fn campfire_diorama() -> Self {
        let mats = SceneMaterials::load();
        let mut cubes = Vec::new();

        // --- Terreno procedural (20 pts) ---
        // Reutiliza generate_terrain con columnas rellenas, colinas, camino y arroyo.
        let terrain = procedural::generate_terrain(
            16, 16, 5, 0.18, 1234,
            &procedural::TerrainPalette {
                grass: mats.grass,
                dirt: mats.dirt,
                stone: mats.stone,
                sand: mats.dirt, // sin playa en este bioma; se reusa dirt
                snow: mats.stone, // sin nieve en este bioma; se reusa stone
                water: mats.water,
                straw: mats.straw,
            },
        );
        cubes.extend(terrain);

        // --- Árboles gigantes (5-6 árboles en anillo alrededor de la clarería, radio ~7-9) ---
        let tree_configs = [
            (-6.5, -6.5, 0),
            (6.5, -6.5, 1),
            (-7.5, -0.5, 2),
            (7.5, -0.5, 0),
            (-6.0, 5.5, 1),
            (6.0, 5.5, 2),
        ];
        for &(x, z, variant) in &tree_configs {
            let tree_y = terrain_height_at(&cubes, x, z);
            cubes.extend(Self::giant_tree(Vector3::new(x, tree_y, z), variant, &mats));
        }

        let ground_y = terrain_height_at(&cubes, 0.0, 0.0);
        // --- Círculo de piedras alrededor de la fogata ---
        for &(x, z) in &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            cubes.push(Cube::new(Vector3::new(x, ground_y, z), 0.5, mats.stone));
        }

        // --- Fuego con forma: 3 cubos emisivos apilados y decrecientes (~0.9, 0.6, 0.35) ---
        cubes.push(Cube::new_box(
            Vector3::new(-0.45, ground_y, -0.45),
            Vector3::new(0.90, 0.48, 0.90),
            mats.fire_base,
        ).without_shadow().with_tile_uv(false));
        cubes.push(Cube::new_box(
            Vector3::new(-0.30, ground_y + 0.45, -0.30),
            Vector3::new(0.60, 0.42, 0.60),
            mats.fire_mid,
        ).without_shadow().with_tile_uv(false));
        cubes.push(Cube::new_box(
            Vector3::new(-0.175, ground_y + 0.84, -0.175),
            Vector3::new(0.35, 0.38, 0.35),
            mats.fire_top,
        ).without_shadow().with_tile_uv(false));

        // --- Brasas: 7 cubos diminutos (~0.08) emisivos flotando sobre el fuego ---
        let ember_offsets = [
            (0.10, ground_y + 1.30, -0.08, 0.08),
            (-0.14, ground_y + 1.55, 0.10, 0.07),
            (0.06, ground_y + 1.82, 0.14, 0.08),
            (-0.08, ground_y + 2.10, -0.12, 0.06),
            (0.15, ground_y + 2.38, -0.04, 0.08),
            (-0.10, ground_y + 2.68, 0.11, 0.07),
            (0.04, ground_y + 2.98, -0.07, 0.06),
        ];
        for &(ex, ey, ez, es) in &ember_offsets {
            cubes.push(Cube::new_box(
                Vector3::new(ex - es * 0.5, ey, ez - es * 0.5),
                Vector3::new(es, es, es),
                mats.ember,
            ).without_shadow().with_tile_uv(false));
        }

        // --- Humo: 8 cubos translúcidos subiendo con dispersión y ligera emisión propia ---
        for i in 0..8 {
            let t = i as f32 / 7.0;
            let sy = ground_y + 1.40 + t * 5.2;
            let ss = 0.45 + t * 1.15; // de 0.45 a 1.60

            // Deriva lateral continua suave hacia +X y +Z
            let drift_x = t * 0.55;
            let drift_z = t * 0.45;

            // Desplazamiento determinístico ±0.15-0.25 para romper la columna recta
            let jitter_x = ((i * 17 + 5) % 9) as f32 / 8.0 * 0.40 - 0.20;
            let jitter_z = ((i * 23 + 3) % 9) as f32 / 8.0 * 0.40 - 0.20;

            let sx = drift_x + jitter_x;
            let sz = drift_z + jitter_z;

            let mat = if i <= 2 {
                mats.smoke_base
            } else if i <= 5 {
                mats.smoke
            } else {
                mats.smoke_top
            };

            cubes.push(Cube::new_box(
                Vector3::new(sx - ss * 0.5, sy, sz - ss * 0.5),
                Vector3::new(ss, ss * 0.85, ss),
                mat,
            ).without_shadow().with_tile_uv(false));
        }

        // --- Troncos horizontales para sentarse (~2x0.5x0.5 con textura bark y mapa normal) ---
        // Log 1: Detrás de la fogata frente a Robo y Lucca
        cubes.push(Cube::new_box(
            Vector3::new(-1.10, ground_y, -1.75),
            Vector3::new(2.20, 0.45, 0.50),
            mats.bark,
        ));
        // Log 2: Costado izquierdo junto a Frog
        cubes.push(Cube::new_box(
            Vector3::new(-1.85, ground_y, -0.90),
            Vector3::new(0.50, 0.45, 1.80),
            mats.bark,
        ));
        // Log 3: Costado derecho junto a Ayla
        cubes.push(Cube::new_box(
            Vector3::new(1.35, ground_y, -0.90),
            Vector3::new(0.50, 0.45, 1.80),
            mats.bark,
        ));
        // Log 4: Junto al árbol de Magus
        cubes.push(Cube::new_box(
            Vector3::new(3.00, ground_y, -2.40),
            Vector3::new(1.80, 0.45, 0.50),
            mats.bark,
        ));

        // --- Puente de tablones donde el camino cruza el arroyo (elevado ~0.08, agua visible a ambos lados) ---
        let bridge_y = terrain_height_at(&cubes, 0.5, 4.0);
        cubes.push(Cube::new_box(
            Vector3::new(-0.50, bridge_y + 0.08, 4.0),
            Vector3::new(2.00, 0.22, 2.00),
            mats.planks,
        ));

        // --- Cristales/gemas tirados cerca de la fogata (refracción + reflexión) ---
        for &(x, z) in &[(-2.0, 0.5), (2.2, -0.8), (0.7, 1.7)] {
            cubes.push(Cube::new(Vector3::new(x, ground_y, z), 0.5, mats.gem));
        }

        // --- Luces (máximo 2: fogata con sombras + luz de relleno azul tenue sin sombras) ---
        let lights = vec![
            Light::campfire(Vector3::new(0.0, ground_y + 1.30, 0.0)),
            Light::sky_fill(Vector3::new(0.0, 15.0, 0.0)),
        ];

        // --- Personajes de pie (Billboards: Robo, Lucca, Frog, Ayla, Magus) ---
        let base_height = 1.8f32;
        let mut billboards = Vec::new();

        let standing_party = [
            ("assets/party/Robo.png", -0.75, -2.20, 1.10),
            ("assets/party/Lucca.png", 0.55, -2.20, 0.90),
            ("assets/party/Frog.png", -2.40, -0.10, 0.75),
            ("assets/party/Ayla.png", 2.35, -0.10, 1.00),
            ("assets/party/Magus.png", 4.20, -1.60, 1.05),
            ("assets/party/Marle.png", -1.65, 1.85, 0.95),
            ("assets/party/Chrono.png", 1.65, 1.85, 1.00),
        ];

        for &(tex, x, z, mult) in &standing_party {
            let y = terrain_height_at(&cubes, x, z);
            let height = base_height * mult;
            let aspect = get_png_aspect_ratio(tex);
            let width = height * aspect;
            billboards.push(Billboard::new(Vector3::new(x, y, z), width, height, tex));
        }

        // --- Mechones de pasto en la periferia de la clarería (Billboards con textura grass_tuft) ---
        let tuft_positions = [
            (-2.7, 0.8), (2.8, -0.6), (-1.2, -2.8), (1.1, -2.9),
            (-2.9, -1.5), (3.1, 0.9), (-0.4, -3.2), (0.5, -3.4),
            (-3.2, 0.4), (3.3, -1.8), (-2.1, -2.6), (2.2, -2.7),
            (-2.7, -2.2), (2.9, -2.1), (-1.8, -3.0), (1.9, -3.1),
            (-3.5, 1.0), (3.6, -0.6), (-2.8, 1.9), (2.9, 1.7),
            (-3.4, -1.1), (3.5, 1.2), (-1.9, 2.7), (2.1, 2.6),
            (-3.1, 2.1), (3.2, 2.0),
        ];
        for (idx, &(tx, tz)) in tuft_positions.iter().enumerate() {
            let ty = terrain_height_at(&cubes, tx, tz);
            let h_var = ((idx * 37 + 11) % 10) as f32 / 9.0;
            let tuft_h = 0.35 + h_var * 0.20;
            let tuft_w = 0.50;
            billboards.push(Billboard::new(
                Vector3::new(tx, ty, tz),
                tuft_w,
                tuft_h,
                "assets/grass_tuft.png",
            ));
        }

        let ground_sprites = Vec::new();

        let grid = crate::grid::VoxelGrid::build(&cubes);

        Scene {
            cubes,
            billboards,
            ground_sprites,
            lights,
            skybox: Skybox::night(),
            textures: crate::texture::TextureManager::new(),
            grid,
            use_normal_maps: true,
            camera_forward: Vector3::new(0.0, 0.0, -1.0),
            camera_right: Vector3::new(1.0, 0.0, 0.0),
            camera_up: Vector3::new(0.0, 1.0, 0.0),
        }
    }

    /// Carga todos los assets de texturas y sprites en el TextureManager de la escena
    pub fn load_textures(&mut self, rl: &mut raylib::prelude::RaylibHandle, thread: &raylib::prelude::RaylibThread) {
        let paths = [
            "assets/grass.png",
            "assets/dirt.png",
            "assets/stone.png",
            "assets/stone_normal.png",
            "assets/bark.png",
            "assets/bark_normal.png",
            "assets/leaves.png",
            "assets/fire.png",
            "assets/gem.png",
            "assets/water.png",
            "assets/straw.png",
            "assets/tuft.png",
            "assets/grass_tuft.png",
            "assets/planks.png",
            "assets/party/Chrono.png",
            "assets/party/Marle.png",
            "assets/party/Lucca.png",
            "assets/party/Frog.png",
            "assets/party/Robo.png",
            "assets/party/Ayla.png",
            "assets/party/Magus.png",
        ];
        for path in paths {
            self.textures.load_texture(rl, thread, path);
        }
    }

    /// Genera un árbol gigante con tronco 2x2, raíces en la base y copas oscuras apiladas en capas.
    /// Con UV tiling, los bloques grandes de tronco y follaje repiten la textura cada 1 unidad en vez de estirarla.
    fn giant_tree(base: Vector3, variant: usize, mats: &SceneMaterials) -> Vec<Cube> {
        let mut parts = Vec::new();

        let (trunk_h, leaf_mat) = match variant {
            0 => (6.5f32, mats.leaves),
            1 => (7.0f32, mats.leaves_alt),
            _ => (6.0f32, mats.leaves),
        };

        // Tronco grueso 2x2 de 6 a 7 bloques de alto (1 solo cubo con UV tiling)
        parts.push(Cube::new_box(
            Vector3::new(base.x - 1.0, base.y, base.z - 1.0),
            Vector3::new(2.0, trunk_h, 2.0),
            mats.bark,
        ));

        // Raíces: cubos de ~1x1 que ensanchan la base del tronco
        parts.push(Cube::new_box(
            Vector3::new(base.x - 1.9, base.y, base.z - 0.5),
            Vector3::new(0.9, 1.1, 1.0),
            mats.bark,
        ));
        parts.push(Cube::new_box(
            Vector3::new(base.x + 1.0, base.y, base.z - 0.5),
            Vector3::new(0.9, 0.9, 1.0),
            mats.bark,
        ));
        parts.push(Cube::new_box(
            Vector3::new(base.x - 0.5, base.y, base.z - 1.9),
            Vector3::new(1.0, 1.0, 0.9),
            mats.bark,
        ));
        parts.push(Cube::new_box(
            Vector3::new(base.x - 0.5, base.y, base.z + 1.0),
            Vector3::new(1.0, 0.8, 0.9),
            mats.bark,
        ));

        // Copas oscuras en 2-3 capas apiladas y de distinto ancho
        match variant {
            0 => {
                // Capa inferior ancha 6x2x6
                parts.push(Cube::new_box(
                    Vector3::new(base.x - 3.0, base.y + trunk_h - 1.0, base.z - 3.0),
                    Vector3::new(6.0, 2.0, 6.0),
                    leaf_mat,
                ));
                // Capa superior 4x2x4
                parts.push(Cube::new_box(
                    Vector3::new(base.x - 2.0, base.y + trunk_h + 1.0, base.z - 2.0),
                    Vector3::new(4.0, 2.0, 4.0),
                    leaf_mat,
                ));
            }
            1 => {
                // Capa inferior 5x2x6
                parts.push(Cube::new_box(
                    Vector3::new(base.x - 2.5, base.y + trunk_h - 1.2, base.z - 3.0),
                    Vector3::new(5.0, 2.0, 6.0),
                    leaf_mat,
                ));
                // Capa media 4x1.8x4
                parts.push(Cube::new_box(
                    Vector3::new(base.x - 2.0, base.y + trunk_h + 0.8, base.z - 2.0),
                    Vector3::new(4.0, 1.8, 4.0),
                    leaf_mat,
                ));
                // Cúpula superior 2.5x1.5x2.5
                parts.push(Cube::new_box(
                    Vector3::new(base.x - 1.25, base.y + trunk_h + 2.6, base.z - 1.25),
                    Vector3::new(2.5, 1.5, 2.5),
                    leaf_mat,
                ));
            }
            _ => {
                // Capa inferior 5.5x2x5.5
                parts.push(Cube::new_box(
                    Vector3::new(base.x - 2.75, base.y + trunk_h - 0.8, base.z - 2.75),
                    Vector3::new(5.5, 2.0, 5.5),
                    leaf_mat,
                ));
                // Capa superior 3.5x2x3.5
                parts.push(Cube::new_box(
                    Vector3::new(base.x - 1.75, base.y + trunk_h + 1.2, base.z - 1.75),
                    Vector3::new(3.5, 2.0, 3.5),
                    leaf_mat,
                ));
            }
        }

        parts
    }
}
