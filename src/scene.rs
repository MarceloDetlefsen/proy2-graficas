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

        // --- Árboles (4 esquinas, como en la referencia de Chrono Trigger) ---
        for &(x, z) in &[(-6.0, -6.0), (6.0, -6.0), (-6.0, 6.0), (6.0, 6.0)] {
            let tree_y = terrain_height_at(&cubes, x, z);
            cubes.extend(Self::tree(Vector3::new(x, tree_y, z), &mats));
        }

        let ground_y = terrain_height_at(&cubes, 0.0, 0.0);
        // --- Círculo de piedras + fogata al centro (emisivo) ---
        for &(x, z) in &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            cubes.push(Cube::new(Vector3::new(x, ground_y, z), 0.5, mats.stone));
        }
        cubes.push(Cube::new(Vector3::new(-0.25, ground_y, -0.25), 0.5, mats.campfire));

        // --- Cristales/gemas tirados cerca de la fogata (refracción + reflexión) ---
        for &(x, z) in &[(-2.0, 0.5), (2.2, -0.8), (0.7, 1.7)] {
            cubes.push(Cube::new(Vector3::new(x, ground_y, z), 0.5, mats.gem));
        }

        // --- Luces (máximo 2: fogata con sombras + luz de relleno azul tenue sin sombras) ---
        let lights = vec![
            Light::campfire(Vector3::new(0.0, ground_y + 0.8, 0.0)),
            Light::sky_fill(Vector3::new(0.0, 15.0, 0.0)),
        ];

        // --- Personajes (7 miembros de Chrono Trigger en círculo alrededor de la fogata) ---
        let party_defs: [(&str, f32); 7] = [
            ("assets/party/Lucca.png", 0.90),
            ("assets/party/Chrono.png", 1.00),
            ("assets/party/Magus.png", 1.05),
            ("assets/party/Ayla.png", 1.00),
            ("assets/party/Robo.png", 1.10),
            ("assets/party/Frog.png", 0.75),
            ("assets/party/Marle.png", 0.95),
        ];

        let base_height = 1.8f32;
        let radius = 3.0f32;
        let base_angle = 17.1f32.to_radians();
        let angle_step = std::f32::consts::TAU / 7.0;

        let mut billboards = Vec::new();
        for (i, &(tex, mult)) in party_defs.iter().enumerate() {
            let angle = base_angle + (i as f32) * angle_step;
            let x = radius * angle.sin();
            let z = radius * angle.cos();
            let y = terrain_height_at(&cubes, x, z);

            let height = base_height * mult;
            let aspect = get_png_aspect_ratio(tex);
            let width = height * aspect;

            billboards.push(Billboard::new(Vector3::new(x, y, z), width, height, tex));
        }

        let grid = crate::grid::VoxelGrid::build(&cubes);

        Scene {
            cubes,
            billboards,
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

    /// Genera un árbol simple: tronco recto + copa de hojas.
    fn tree(base: Vector3, mats: &SceneMaterials) -> Vec<Cube> {
        let mut parts = Vec::new();
        for i in 0..3 {
            parts.push(Cube::new(base + Vector3::new(0.0, i as f32, 0.0), 1.0, mats.bark));
        }
        for dx in -1..=1 {
            for dz in -1..=1 {
                for dy in 0..2 {
                    parts.push(Cube::new(
                        base + Vector3::new(dx as f32, 3.0 + dy as f32, dz as f32),
                        1.0,
                        mats.leaves,
                    ));
                }
            }
        }
        parts
    }
}
