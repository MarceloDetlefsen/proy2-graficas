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

impl Scene {
    /// Arma el diorama: "Campamento nocturno" (16x16 mínimo de terreno).
    pub fn campfire_diorama() -> Self {
        let mats = SceneMaterials::load();
        let mut cubes = Vec::new();

        // --- Terreno procedural (20 pts) ---
        // Reutiliza generate_terrain ya probado en clase.
        let terrain = procedural::generate_terrain(
            16, 16, 4, 0.15, 1234,
            &procedural::TerrainPalette {
                grass: mats.grass,
                dirt: mats.dirt,
                stone: mats.stone,
                sand: mats.dirt, // sin playa en este bioma; se reusa dirt
                snow: mats.stone, // sin nieve en este bioma; se reusa stone
            },
        );
        cubes.extend(terrain);

        // --- Árboles (4 esquinas, como en la referencia de Chrono Trigger) ---
        for &(x, z) in &[(-6.0, -6.0), (6.0, -6.0), (-6.0, 6.0), (6.0, 6.0)] {
            cubes.extend(Self::tree(Vector3::new(x, 1.0, z), &mats));
        }

        let ground_y = 3.0;
        // --- Círculo de piedras + fogata al centro (emisivo) ---
        for &(x, z) in &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            cubes.push(Cube::new(Vector3::new(x, ground_y, z), 0.5, mats.stone));
        }
        cubes.push(Cube::new(Vector3::new(-0.25, ground_y, -0.25), 0.5, mats.campfire));

        // --- Cristales/gemas tirados cerca de la fogata y el charco (refracción + reflexión) ---
        for &(x, z) in &[(-2.0, 0.5), (2.2, -0.8), (0.7, 1.7)] {
            cubes.push(Cube::new(Vector3::new(x, ground_y, z), 0.5, mats.gem));
        }

        // --- Charco de agua a ras de suelo en depresión (reflexión) ---
        cubes.push(Cube::new_box(
            Vector3::new(3.0, 2.01, 3.0),
            Vector3::new(2.8, 0.09, 2.8),
            mats.water,
        ));

        // --- Luces ---
        let lights = vec![Light::campfire(Vector3::new(0.0, ground_y + 0.8, 0.0))];

        // --- Personajes (billboards con sprites pixel-art en semicírculo sobre el terreno real) ---
        let party_configs = [
            (-1.8, -0.6, "assets/party/hero.png"),
            (-0.9, -1.8, "assets/party/mage.png"),
            (0.9, -1.8, "assets/party/warrior.png"),
            (1.8, -0.6, "assets/party/rogue.png"),
        ];

        let mut billboards = Vec::new();
        for (x, z, tex) in party_configs {
            let y = terrain_height_at(&cubes, x, z);
            billboards.push(Billboard::new(Vector3::new(x, y, z), 1.0, 1.4, tex));
        }

        Scene {
            cubes,
            billboards,
            lights,
            skybox: Skybox::night(),
            textures: crate::texture::TextureManager::new(),
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
            "assets/party/hero.png",
            "assets/party/mage.png",
            "assets/party/warrior.png",
            "assets/party/rogue.png",
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
