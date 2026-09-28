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

        // --- Círculo de piedras + fogata al centro (emisivo) ---
        for &(x, z) in &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            cubes.push(Cube::new(Vector3::new(x, 1.0, z), 0.5, mats.stone));
        }
        cubes.push(Cube::new(Vector3::new(-0.25, 1.0, -0.25), 0.5, mats.campfire));

        // --- Cristales/gemas tirados cerca de la fogata (refracción + reflexión) ---
        for &(x, z) in &[(-2.0, 0.5), (2.2, -0.8), (1.5, 1.8)] {
            cubes.push(Cube::new(Vector3::new(x, 1.0, z), 0.3, mats.gem));
        }

        // --- Charco de agua (reflexión) ---
        cubes.push(Cube::new(Vector3::new(4.0, 0.95, 3.0), 2.0, mats.water));

        // --- Luces ---
        let lights = vec![Light::campfire(Vector3::new(0.0, 1.6, 0.0))];

        // --- Personajes (billboards con sprites pixel-art) ---
        let billboards = vec![
            Billboard::new(Vector3::new(-0.8, 1.5, -0.8), 1.0, 1.4, "assets/party/hero.png"),
            Billboard::new(Vector3::new(0.8, 1.5, -0.8), 1.0, 1.4, "assets/party/mage.png"),
            Billboard::new(Vector3::new(-0.8, 1.5, 0.8), 1.0, 1.4, "assets/party/warrior.png"),
            Billboard::new(Vector3::new(0.8, 1.5, 0.8), 1.0, 1.4, "assets/party/rogue.png"),
        ];

        Scene {
            cubes,
            billboards,
            lights,
            skybox: Skybox::night(),
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
