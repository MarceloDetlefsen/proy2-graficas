use raylib::prelude::Vector3;
use crate::cube::Cube;
use crate::material::SceneMaterials;
use crate::light::Light;
use crate::skybox::Skybox;
use crate::billboard::Billboard;
use crate::procedural;

#[allow(dead_code)]
pub struct Scene {
    pub cubes: Vec<Cube>,
    pub billboards: Vec<Billboard>, // Personajes de la party (7) + Gate vortex (1) + carpa (1)
    pub party_aabb: (Vector3, Vector3),
    pub party_left: Vec<Billboard>,
    pub party_right: Vec<Billboard>,
    pub party_left_aabb: (Vector3, Vector3),
    pub party_right_aabb: (Vector3, Vector3),
    pub grass_billboards: Vec<Billboard>, // Mechones de pasto (~20)
    pub grass_left: Vec<Billboard>,
    pub grass_right: Vec<Billboard>,
    pub grass_left_aabb: (Vector3, Vector3),
    pub grass_right_aabb: (Vector3, Vector3),
    pub smoke_billboards: Vec<Billboard>, // Puffs de humo (11)
    pub smoke_aabb: (Vector3, Vector3),
    pub undergrowth_billboards: Vec<Billboard>, // Helechos y hongos luminiscentes (13)
    pub undergrowth_left: Vec<Billboard>,
    pub undergrowth_right: Vec<Billboard>,
    pub undergrowth_left_aabb: (Vector3, Vector3),
    pub undergrowth_right_aabb: (Vector3, Vector3),
    pub ground_sprites: Vec<crate::billboard::GroundSprite>,
    pub lights: Vec<Light>,
    pub skybox: Skybox,
    pub textures: crate::texture::TextureManager,
    pub grid: crate::grid::VoxelGrid,
    pub use_normal_maps: bool,
    pub use_reflections: bool,
    pub tree_cutaway_dist: Option<f32>,
    pub camera_forward: Vector3,
    pub camera_right: Vector3,
    pub camera_up: Vector3,
}

/// Calcula el AABB envolvente que engloba un conjunto de billboards con holgura
pub fn compute_billboards_aabb(bbs: &[Billboard]) -> (Vector3, Vector3) {
    if bbs.is_empty() {
        return (Vector3::zero(), Vector3::zero());
    }
    let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

    for bb in bbs {
        let half_w = bb.width * 0.5 + 0.1;
        let bb_min_x = bb.position.x - half_w;
        let bb_max_x = bb.position.x + half_w;
        let bb_min_y = bb.position.y - 0.1;
        let bb_max_y = bb.position.y + bb.height + 0.1;
        let bb_min_z = bb.position.z - half_w;
        let bb_max_z = bb.position.z + half_w;

        min.x = min.x.min(bb_min_x);
        min.y = min.y.min(bb_min_y);
        min.z = min.z.min(bb_min_z);

        max.x = max.x.max(bb_max_x);
        max.y = max.y.max(bb_max_y);
        max.z = max.z.max(bb_max_z);
    }
    (min, max)
}

/// Calcula la altura real del suelo en (x, z) EXCLUYENDO cubos is_tree (raíces, troncos, copas).
pub fn terrain_only_height_at(cubes: &[Cube], x: f32, z: f32) -> f32 {
    let mut max_y = 0.0f32;
    let mut found = false;
    for cube in cubes {
        if !cube.is_tree && cube.max.y <= 5.5 && cube.min.x <= x && x < cube.max.x && cube.min.z <= z && z < cube.max.z {
            if !found || cube.max.y > max_y {
                max_y = cube.max.y;
                found = true;
            }
        }
    }
    if found { max_y } else { 4.0 }
}

/// Alias retrocompatible de terrain_only_height_at
pub fn terrain_height_at(cubes: &[Cube], x: f32, z: f32) -> f32 {
    terrain_only_height_at(cubes, x, z)
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

        // --- Terreno procedural (20 pts, 20x20 con claro plano a y=4.0) ---
        let terrain = procedural::generate_terrain(
            20, 20, 5, 0.18, 1234,
            &procedural::TerrainPalette {
                grass: mats.grass,
                dirt: mats.dirt,
                stone: mats.stone,
                sand: mats.dirt,
                snow: mats.stone,
                water: mats.water,
                straw: mats.straw,
            },
        );
        cubes.extend(terrain);

        // --- Árboles gigantes en capas (6 árboles con soporte completo en suelo y roots) ---
        let tree_configs = [
            (-7.2, -1.0, 0), // Gigante cercano W
            (7.3, -0.6, 1),  // Gigante cercano E
            (-6.6, -4.8, 2), // Gigante cercano NW
            (7.2, -4.2, 0),  // Árbol de Magus NE (justo detrás y a la derecha de Magus)
            (-3.8, -7.4, 1), // Gigante fondo NW
            (3.6, -7.6, 2),  // Gigante fondo NE
        ];
        for (i, &(x, z, variant)) in tree_configs.iter().enumerate() {
            assert!(x >= -8.5 && x <= 8.5 && z >= -8.5 && z <= 8.5, "Árbol {} fuera del límite permitido del terreno!", i + 1);
            let tree_cubes = Self::giant_tree(x, z, variant, &cubes, &mats);
            cubes.extend(tree_cubes);
        }

        // --- Pared perimetral: 14 troncos delgados (1x7-9x1, is_tree) a ~8.5 del centro ---
        let perimeter_trunks = [
            // Norte (z ~ -8.5)
            (-6.2, -8.6, 8.5f32),
            (-2.0, -8.7, 8.0f32),
            (0.2, -8.5, 8.0f32),
            (1.9, -8.5, 8.0f32),
            (6.0, -8.7, 8.8f32),
            // Oeste (x ~ -8.5)
            (-8.6, -6.2, 8.0f32),
            (-8.5, -2.5, 8.5f32),
            (-8.6, 2.2, 7.8f32),
            // Este (x ~ +8.5)
            (8.6, -6.5, 8.2f32),
            (8.5, -2.2, 8.6f32),
            (8.6, 2.0, 8.0f32),
            // Sur (z ~ +8.5, dejando hueco |x| < 4 libre para camino y puente)
            (-7.8, 8.5, 7.5f32),
            (-5.2, 8.6, 8.2f32),
            (5.4, 8.5, 8.0f32),
            (7.8, 8.6, 7.8f32),
        ];
        for (i, &(px, pz, ph)) in perimeter_trunks.iter().enumerate() {
            let py = terrain_only_height_at(&cubes, px, pz);
            cubes.push(Cube::new_box(
                Vector3::new(px - 0.5, py, pz - 0.5),
                Vector3::new(1.0, ph, 1.0),
                mats.bark,
            ).as_tree());

            // Copa para cada árbol perimetral (capas apiladas proporcionadas al tronco 1x1)
            let top_y = py + ph;
            let leaf_mat = if i % 2 == 0 { mats.leaves } else { mats.leaves_alt };
            let (w1, h1, w2, h2) = match i % 3 {
                0 => (2.6f32, 1.6f32, 1.8f32, 1.4f32),
                1 => (2.8f32, 1.8f32, 2.0f32, 1.2f32),
                _ => (2.4f32, 1.6f32, 1.6f32, 1.4f32),
            };

            cubes.push(Cube::new_box(
                Vector3::new(px - w1 * 0.5, top_y - 0.8, pz - w1 * 0.5),
                Vector3::new(w1, h1, w1),
                leaf_mat,
            ).as_tree());

            cubes.push(Cube::new_box(
                Vector3::new(px - w2 * 0.5, top_y - 0.8 + h1, pz - w2 * 0.5),
                Vector3::new(w2, h2, w2),
                leaf_mat,
            ).as_tree());
        }

        // Cúmulos de hojas bajas (8) entre los troncos perimetrales
        let perimeter_foliage = [
            (-4.2, -8.6, 1.6f32, 1.4f32, 1.6f32),
            (3.8, -8.5, 1.5f32, 1.3f32, 1.5f32),
            (-8.5, -4.5, 1.4f32, 1.5f32, 1.4f32),
            (-8.5, 0.0, 1.6f32, 1.4f32, 1.6f32),
            (8.5, -4.5, 1.5f32, 1.5f32, 1.5f32),
            (8.5, 0.0, 1.4f32, 1.3f32, 1.4f32),
            (-6.5, 8.5, 1.5f32, 1.4f32, 1.5f32),
            (6.5, 8.5, 1.5f32, 1.4f32, 1.5f32),
        ];
        for &(fx, fz, fw, fh, fd) in &perimeter_foliage {
            let fy = terrain_only_height_at(&cubes, fx, fz);
            cubes.push(Cube::new_box(
                Vector3::new(fx - fw * 0.5, fy, fz - fd * 0.5),
                Vector3::new(fw, fh, fd),
                mats.leaves,
            ).as_tree());
        }

        // --- Arbustos de primer plano (hojas oscuras, altura <= 1.3) en esquinas inferiores (|x| >= 5.5, z >= 4) ---
        let foreground_shrubs = [
            (-6.0, 4.6, 1.2, 1.1, 1.2),
            (-7.2, 5.8, 1.4, 1.2, 1.4),
            (-6.2, 7.0, 1.3, 1.0, 1.3),
            (6.0, 4.6, 1.2, 1.1, 1.2),
            (7.2, 5.8, 1.4, 1.2, 1.4),
            (6.2, 7.0, 1.3, 1.0, 1.3),
        ];
        for &(sx, sz, sw, sh, sd) in &foreground_shrubs {
            let sy = terrain_only_height_at(&cubes, sx, sz);
            cubes.push(Cube::new_box(
                Vector3::new(sx - sw * 0.5, sy, sz - sd * 0.5),
                Vector3::new(sw, sh, sd),
                mats.leaves_alt,
            ).as_tree());
        }

        // --- 5 Rocas pequeñas de piedra con mapa normal en la periferia ---
        let rock_configs = [
            (-3.8, -1.8, 0.55),
            (2.8, -1.8, 0.60),
            (-2.6, 2.2, 0.48),
            (3.2, 2.0, 0.52),
            (0.8, -3.2, 0.50),
        ];
        for &(rx, rz, rs) in &rock_configs {
            let ry = terrain_height_at(&cubes, rx, rz);
            cubes.push(Cube::new_box(
                Vector3::new(rx - rs * 0.5, ry, rz - rs * 0.5),
                Vector3::new(rs, rs * 0.8, rs),
                mats.stone,
            ));
        }

        let ground_y = terrain_height_at(&cubes, 0.0, 0.0);
        // --- Círculo de piedras alrededor de la fogata ---
        for &(x, z) in &[(-1.0, -1.0), (1.0, -1.0), (-1.25, 1.20), (1.25, 1.20)] {
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

        // --- 8 Luciérnagas emisivas flotando a distintas alturas en el claro ---
        let firefly_offsets = [
            (-1.2, 1.2, -0.8),
            (1.4, 1.5, -1.2),
            (-0.9, 1.8, 1.4),
            (1.1, 1.1, 1.6),
            (-2.2, 1.4, 0.2),
            (2.0, 1.6, 0.5),
            (-0.3, 2.1, -1.8),
            (0.5, 2.3, 0.8),
        ];
        for &(fx, fy_off, fz) in &firefly_offsets {
            let fy = terrain_height_at(&cubes, fx, fz) + fy_off;
            let fs = 0.08;
            cubes.push(Cube::new_box(
                Vector3::new(fx - fs * 0.5, fy, fz - fs * 0.5),
                Vector3::new(fs, fs, fs),
                mats.firefly,
            ).without_shadow().with_tile_uv(false));
        }

        // --- Troncos horizontales para sentarse (bark + mapa normal) ---
        // Tronco A (-0.4, -1.7), largo en X: 2.20
        cubes.push(Cube::new_box(
            Vector3::new(-1.50, ground_y, -1.95),
            Vector3::new(2.20, 0.45, 0.50),
            mats.bark,
        ).with_log_axis('X'));
        // Tronco B (-3.0, -0.4), largo en Z: 2.00
        cubes.push(Cube::new_box(
            Vector3::new(-3.25, ground_y, -1.40),
            Vector3::new(0.50, 0.45, 2.00),
            mats.bark,
        ).with_log_axis('Z'));
        // Tronco C (3.0, -0.4), largo en Z: 2.00
        cubes.push(Cube::new_box(
            Vector3::new(2.75, ground_y, -1.40),
            Vector3::new(0.50, 0.45, 2.00),
            mats.bark,
        ).with_log_axis('Z'));

        // --- Puente de tablones donde el camino cruza el arroyo en z in [4, 6] ---
        let bridge_y = 4.02;
        cubes.push(Cube::new_box(
            Vector3::new(-0.50, bridge_y, 4.0),
            Vector3::new(2.00, 0.22, 2.00),
            mats.planks,
        ));

        // --- 2 Cristales translúcidos en (+-0.6, 0.95) con refracción visible (n=1.50, t=0.60) ---
        cubes.push(Cube::new_box(
            Vector3::new(-0.84, ground_y, 0.71),
            Vector3::new(0.48, 0.52, 0.48),
            mats.gem_cyan,
        ));
        cubes.push(Cube::new_box(
            Vector3::new(0.36, ground_y, 0.71),
            Vector3::new(0.48, 0.52, 0.48),
            mats.gem_magenta,
        ));

        // --- Easter Egg: Colgante real de Marle en el suelo junto a sus pies (-1.95, 2.25) ---
        let pendant_ground = terrain_only_height_at(&cubes, -1.95, 2.25);
        cubes.push(Cube::new_box(
            Vector3::new(-2.02, pendant_ground, 2.18),
            Vector3::new(0.14, 0.03, 0.14),
            mats.gold,
        ).without_shadow().with_tile_uv(false));
        cubes.push(Cube::new_box(
            Vector3::new(-1.99, pendant_ground + 0.03, 2.21),
            Vector3::new(0.08, 0.07, 0.08),
            mats.pendant,
        ).without_shadow().with_tile_uv(false));

        // --- Props del grupo Chrono Trigger (sobre el suelo real) ---
        // Masamune de Frog clavada en el suelo (-4.8, -0.2)
        let masa_y = terrain_only_height_at(&cubes, -4.80, -0.20);
        cubes.push(Cube::new_box(
            Vector3::new(-4.85, masa_y, -0.25),
            Vector3::new(0.10, 0.85, 0.10),
            mats.metal,
        ));
        cubes.push(Cube::new_box(
            Vector3::new(-4.89, masa_y + 0.85, -0.29),
            Vector3::new(0.18, 0.12, 0.18),
            mats.gold,
        ));

        // Casco de Lucca sobre el tronco A (-0.10, -1.70)
        cubes.push(Cube::new_box(
            Vector3::new(-0.22, ground_y + 0.45, -1.82),
            Vector3::new(0.24, 0.18, 0.24),
            mats.gold,
        ));

        // Ballesta de Marle apoyada en el suelo junto a ella (-2.65, 2.25)
        let bow_y = terrain_only_height_at(&cubes, -2.65, 2.25);
        cubes.push(Cube::new_box(
            Vector3::new(-2.70, bow_y, 2.05),
            Vector3::new(0.10, 0.10, 0.40),
            mats.bark,
        ));
        cubes.push(Cube::new_box(
            Vector3::new(-3.05, bow_y + 0.02, 2.38),
            Vector3::new(0.40, 0.08, 0.08),
            mats.metal,
        ));

        // Katana de Chrono en el suelo junto a él (2.85, 2.25)
        let katana_y = terrain_only_height_at(&cubes, 2.85, 2.25);
        cubes.push(Cube::new_box(
            Vector3::new(2.80, katana_y, 2.20),
            Vector3::new(0.08, 0.75, 0.08),
            mats.metal,
        ));
        cubes.push(Cube::new_box(
            Vector3::new(2.77, katana_y + 0.75, 2.17),
            Vector3::new(0.14, 0.10, 0.14),
            mats.gold,
        ));

        // Olla de hierro sobre trípode de 3 palitos finos junto al fuego (-0.80, -0.65)
        let pot_ground = terrain_only_height_at(&cubes, -0.80, -0.65);
        cubes.push(Cube::new_box(
            Vector3::new(-0.95, pot_ground, -0.78),
            Vector3::new(0.04, 0.65, 0.04),
            mats.bark,
        ));
        cubes.push(Cube::new_box(
            Vector3::new(-0.73, pot_ground, -0.78),
            Vector3::new(0.04, 0.65, 0.04),
            mats.bark,
        ));
        cubes.push(Cube::new_box(
            Vector3::new(-0.84, pot_ground, -0.58),
            Vector3::new(0.04, 0.65, 0.04),
            mats.bark,
        ));
        cubes.push(Cube::new_box(
            Vector3::new(-0.94, pot_ground + 0.22, -0.79),
            Vector3::new(0.28, 0.24, 0.28),
            mats.pot_iron,
        ));
        cubes.push(Cube::new_box(
            Vector3::new(-0.955, pot_ground + 0.44, -0.805),
            Vector3::new(0.31, 0.04, 0.31),
            mats.pot_rim,
        ));

        // --- Luces (máximo 2: fogata con sombras + luz de relleno azul tenue sin sombras) ---
        let lights = vec![
            Light::campfire(Vector3::new(0.0, ground_y + 1.30, 0.0)),
            Light::sky_fill(Vector3::new(0.0, 15.0, 0.0)),
        ];

        // --- 1. Humo con 11 billboards: bocanadas procedurales que suben desde la llama ---
        let mut smoke_billboards = Vec::new();
        let smoke_textures = [
            "assets/smoke_puff_0.png",
            "assets/smoke_puff_1.png",
            "assets/smoke_puff_2.png",
        ];
        for i in 0..11 {
            let t = i as f32 / 10.0;
            let sy = ground_y + 1.25 + t * 4.8;
            let size = 0.60 + t * 1.20;

            let drift_x = (t * std::f32::consts::PI * 1.6).sin() * 0.35 + t * 0.30;
            let drift_z = (t * std::f32::consts::PI * 1.2).cos() * 0.20 + t * 0.25;
            let jitter_x = ((i * 17 + 5) % 9) as f32 / 8.0 * 0.24 - 0.12;
            let jitter_z = ((i * 23 + 3) % 9) as f32 / 8.0 * 0.24 - 0.12;

            let sx = drift_x + jitter_x;
            let sz = drift_z + jitter_z;

            let tex = smoke_textures[i % 3];
            let warm_emit = if i < 3 {
                Vector3::new(0.18, 0.10, 0.03) * (1.0 - i as f32 / 3.0)
            } else {
                Vector3::zero()
            };

            smoke_billboards.push(
                Billboard::new(Vector3::new(sx, sy, sz), size, size, tex)
                    .with_smoke(true)
                    .with_emission(warm_emit),
            );
        }

        // --- 2. Personajes de pie (Billboards: Robo, Lucca, Frog, Ayla, Magus, Marle, Chrono) ---
        let mut party_billboards = Vec::new();
        let base_height = 1.8f32;
        let standing_party = [
            ("assets/party/Robo.png", -1.80, -2.80, 1.10),
            ("assets/party/Lucca.png", 0.90, -3.20, 0.90),
            ("assets/party/Frog.png", -4.10, -0.40, 0.75),
            ("assets/party/Ayla.png", 3.90, -0.40, 1.00),
            ("assets/party/Magus.png", 5.60, -2.60, 1.05),
            ("assets/party/Marle.png", -2.20, 2.20, 0.95),
            ("assets/party/Chrono.png", 2.40, 2.40, 1.00),
        ];

        for &(tex, x, z, mult) in &standing_party {
            let y = terrain_only_height_at(&cubes, x, z);
            let height = base_height * mult;
            let aspect = get_png_aspect_ratio(tex);
            let width = height * aspect;
            party_billboards.push(
                Billboard::new(Vector3::new(x, y, z), width, height, tex)
                    .with_blob_shadow(true),
            );
        }

        // Billboard de la carpa triangular de campamento en (3.4, -4.6)
        let tent_x = 3.40f32;
        let tent_z = -4.60f32;
        let tent_y = terrain_only_height_at(&cubes, tent_x, tent_z);
        party_billboards.push(
            Billboard::new(Vector3::new(tent_x, tent_y, tent_z), 2.50, 2.10, "assets/tent.png")
                .with_blob_shadow(true),
        );

        // --- Easter Egg: Portal escondido (Gate Vortex) entre troncos perimetrales al fondo (1.05, -8.65) ---
        let vortex_size = 1.10f32;
        let vortex_x = 1.05f32;
        let vortex_z = -8.65f32;
        let vortex_y = terrain_only_height_at(&cubes, vortex_x, vortex_z);
        party_billboards.push(
            Billboard::new(
                Vector3::new(vortex_x, vortex_y, vortex_z),
                vortex_size,
                vortex_size,
                "assets/gate_vortex.png",
            ).with_emission(Vector3::new(0.03, 0.12, 0.22)),
        );
        // Pared de corteza/roca de respaldo detrás del portal para ocluir vistas traseras de órbita
        cubes.push(Cube::new_box(
            Vector3::new(0.40, vortex_y, -9.15),
            Vector3::new(1.30, 4.0, 0.50),
            mats.bark,
        ));

        // --- 3. Mechones de pasto en la periferia de la clarería (~20 billboards grass_tuft) ---
        let mut grass_billboards = Vec::new();
        let tuft_positions = [
            (-2.7, 0.8), (2.8, -0.6), (-1.2, -2.8), (1.1, -2.9),
            (-2.9, -1.5), (3.1, 0.9), (-0.4, -3.2), (0.5, -3.4),
            (-3.2, 0.4), (3.3, -1.8), (-2.1, -2.6), (2.2, -2.7),
            (-2.7, -2.2), (2.9, -2.1), (-1.8, -3.0), (1.9, -3.1),
            (-3.5, 1.0), (3.6, -0.6), (-2.8, 1.9), (2.9, 1.7),
        ];
        for (idx, &(tx, tz)) in tuft_positions.iter().enumerate() {
            let ty = terrain_height_at(&cubes, tx, tz);
            let h_var = ((idx * 37 + 11) % 10) as f32 / 9.0;
            let tuft_h = 0.35 + h_var * 0.20;
            let tuft_w = 0.50;
            grass_billboards.push(Billboard::new(
                Vector3::new(tx, ty, tz),
                tuft_w,
                tuft_h,
                "assets/grass_tuft.png",
            ));
        }

        // --- 4. Sotobosque: 8 helechos + 5 hongos luminiscentes ---
        let mut undergrowth_billboards = Vec::new();
        let fern_positions = [
            (-2.8, -3.0), (2.8, -3.0), (-3.5, 0.5), (3.2, 0.2),
            (-2.8, 2.5), (2.5, 2.8), (-1.5, -3.5), (1.8, -3.6),
        ];
        for &(fx, fz) in &fern_positions {
            let fy = terrain_height_at(&cubes, fx, fz);
            undergrowth_billboards.push(
                Billboard::new(Vector3::new(fx, fy, fz), 0.55, 0.55, "assets/fern.png")
            );
        }

        let mushroom_positions = [
            (-2.0, -1.8), (-3.2, -1.2), (2.2, -1.8), (-1.8, 2.6), (2.8, 1.8),
        ];
        for &(mx, mz) in &mushroom_positions {
            let my = terrain_height_at(&cubes, mx, mz);
            undergrowth_billboards.push(
                Billboard::new(Vector3::new(mx, my, mz), 0.38, 0.38, "assets/mushroom.png")
                    .with_emission(Vector3::new(0.08, 0.18, 0.25))
            );
        }

        let party_aabb = compute_billboards_aabb(&party_billboards);
        let mut party_left = Vec::new();
        let mut party_right = Vec::new();
        for bb in &party_billboards {
            if bb.position.x < 0.0 { party_left.push(*bb); } else { party_right.push(*bb); }
        }
        let party_left_aabb = compute_billboards_aabb(&party_left);
        let party_right_aabb = compute_billboards_aabb(&party_right);

        let smoke_aabb = compute_billboards_aabb(&smoke_billboards);

        let mut grass_left = Vec::new();
        let mut grass_right = Vec::new();
        for bb in &grass_billboards {
            if bb.position.x < 0.0 { grass_left.push(*bb); } else { grass_right.push(*bb); }
        }
        let grass_left_aabb = compute_billboards_aabb(&grass_left);
        let grass_right_aabb = compute_billboards_aabb(&grass_right);

        let mut undergrowth_left = Vec::new();
        let mut undergrowth_right = Vec::new();
        for bb in &undergrowth_billboards {
            if bb.position.x < 0.0 { undergrowth_left.push(*bb); } else { undergrowth_right.push(*bb); }
        }
        let undergrowth_left_aabb = compute_billboards_aabb(&undergrowth_left);
        let undergrowth_right_aabb = compute_billboards_aabb(&undergrowth_right);

        let ground_sprites = Vec::new();

        let grid = crate::grid::VoxelGrid::build(&cubes);

        Scene {
            cubes,
            billboards: party_billboards,
            party_aabb,
            party_left,
            party_right,
            party_left_aabb,
            party_right_aabb,
            grass_billboards,
            grass_left,
            grass_right,
            grass_left_aabb,
            grass_right_aabb,
            smoke_billboards,
            smoke_aabb,
            undergrowth_billboards,
            undergrowth_left,
            undergrowth_right,
            undergrowth_left_aabb,
            undergrowth_right_aabb,
            ground_sprites,
            lights,
            skybox: Skybox::night(),
            textures: crate::texture::TextureManager::new(),
            grid,
            use_normal_maps: true,
            use_reflections: true,
            tree_cutaway_dist: None,
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
            "assets/gem_cyan.png",
            "assets/gem_magenta.png",
            "assets/water.png",
            "assets/straw.png",
            "assets/tuft.png",
            "assets/grass_tuft.png",
            "assets/smoke_puff_0.png",
            "assets/smoke_puff_1.png",
            "assets/smoke_puff_2.png",
            "assets/fern.png",
            "assets/mushroom.png",
            "assets/cloth.png",
            "assets/tent.png",
            "assets/gate_vortex.png",
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
    /// Se apoya en la altura mínima del terreno bajo su huella para garantizar contacto en todas las celdas.
    fn giant_tree(base_x: f32, base_z: f32, variant: usize, cubes: &[Cube], mats: &SceneMaterials) -> Vec<Cube> {
        let mut parts = Vec::new();

        let (trunk_h, leaf_mat) = match variant {
            0 => (6.5f32, mats.leaves),
            1 => (7.0f32, mats.leaves_alt),
            _ => (6.0f32, mats.leaves),
        };

        // Altura mínima y máxima del terreno bajo la huella 2x2 del tronco
        let samples = [
            (base_x - 0.5, base_z - 0.5),
            (base_x + 0.5, base_z - 0.5),
            (base_x - 0.5, base_z + 0.5),
            (base_x + 0.5, base_z + 0.5),
        ];
        let mut min_h = f32::INFINITY;
        let mut max_h = f32::NEG_INFINITY;
        for &(sx, sz) in &samples {
            let h = terrain_height_at(cubes, sx, sz);
            if h < min_h { min_h = h; }
            if h > max_h { max_h = h; }
        }
        if min_h == f32::INFINITY { min_h = 0.0; max_h = 0.0; }

        // El tronco 2x2 se extiende desde la altura mínima del terreno hacia abajo
        // para que en todas las celdas de la huella toque el suelo sin dejar huecos visibles
        let total_trunk_h = (max_h - min_h) + trunk_h;
        parts.push(Cube::new_box(
            Vector3::new(base_x - 1.0, min_h, base_z - 1.0),
            Vector3::new(2.0, total_trunk_h, 2.0),
            mats.bark,
        ).as_tree());

        // Raíces: 4 cubos de bark en el pie, sobresaliendo del tronco a ras del suelo
        // con margen >= 1 celda del borde del terreno [-10, 10]
        let root_offsets = [
            (base_x - 1.5, base_z - 0.5, 0.5, 0.9, 1.0),
            (base_x + 1.0, base_z - 0.5, 0.6, 0.8, 1.0),
            (base_x - 0.5, base_z - 1.3, 1.0, 0.9, 0.5),
            (base_x - 0.5, base_z + 1.0, 1.0, 0.8, 0.6),
        ];
        for &(rx, rz, rw, rh, rd) in &root_offsets {
            let root_y = terrain_only_height_at(cubes, rx + rw * 0.5, rz + rd * 0.5);
            let root_base = root_y.min(min_h);
            parts.push(Cube::new_box(
                Vector3::new(rx, root_base, rz),
                Vector3::new(rw, (root_y - root_base) + rh, rd),
                mats.bark,
            ).as_tree());
        }

        // Copas oscuras en capas apiladas
        let top_y = min_h + total_trunk_h;
        let mut push_canopy_cube = |pos: Vector3, size: Vector3| {
            parts.push(Cube::new_box(pos, size, leaf_mat).as_tree());
        };

        match variant {
            0 => {
                push_canopy_cube(
                    Vector3::new(base_x - 2.5, top_y - 1.0, base_z - 2.5),
                    Vector3::new(5.0, 2.0, 5.0),
                );
                push_canopy_cube(
                    Vector3::new(base_x - 1.75, top_y + 1.0, base_z - 1.75),
                    Vector3::new(3.5, 2.0, 3.5),
                );
            }
            1 => {
                push_canopy_cube(
                    Vector3::new(base_x - 2.4, top_y - 1.2, base_z - 2.4),
                    Vector3::new(4.8, 2.0, 4.8),
                );
                push_canopy_cube(
                    Vector3::new(base_x - 1.8, top_y + 0.8, base_z - 1.8),
                    Vector3::new(3.6, 1.8, 3.6),
                );
                push_canopy_cube(
                    Vector3::new(base_x - 1.1, top_y + 2.6, base_z - 1.1),
                    Vector3::new(2.2, 1.5, 2.2),
                );
            }
            _ => {
                push_canopy_cube(
                    Vector3::new(base_x - 2.5, top_y - 0.8, base_z - 2.5),
                    Vector3::new(5.0, 2.0, 5.0),
                );
                push_canopy_cube(
                    Vector3::new(base_x - 1.6, top_y + 1.2, base_z - 1.6),
                    Vector3::new(3.2, 2.0, 3.2),
                );
            }
        }

        parts
    }
}
