use raylib::prelude::Vector3;

/// Un material define cómo interactúa la luz con una superficie.
/// Cada material "cuenta" para la rúbrica si tiene su propia textura
/// Y sus propios parámetros de albedo, specular, transparencia y reflectividad.
#[derive(Debug, Clone, Copy)]
pub struct Material {
    /// Color base del material cuando no hay textura, o tinte multiplicado con la textura.
    pub albedo: Vector3,

    /// Qué tan fuerte y concentrado es el brillo especular (highlight).
    /// Valor alto = brillo pequeño y intenso (metal/vidrio pulido).
    /// Valor bajo = brillo amplio y suave (piedra, madera).
    pub specular: f32,

    /// 0.0 = totalmente opaco, 1.0 = totalmente transparente.
    /// Los materiales con transparency > 0 deben implementar refracción.
    pub transparency: f32,

    /// 0.0 = no refleja nada, 1.0 = espejo perfecto.
    pub reflectivity: f32,

    /// Índice de refracción (solo relevante si transparency > 0).
    /// Aire = 1.0, vidrio ~1.5, agua ~1.33, diamante ~2.4.
    pub refractive_index: f32,

    /// Color y fuerza de emisión propia de luz. Vector3::zero() = no emisivo.
    /// Si es distinto de cero, este material actúa como fuente de luz.
    pub emission: Vector3,

    /// Ruta a la textura de color (albedo map), si tiene.
    pub texture: Option<&'static str>,

    /// Ruta al mapa normal, si tiene. Perturba la normal de la superficie
    /// para simular detalle (corteza, rugosidad de piedra) sin más geometría.
    pub normal_map: Option<&'static str>,
}

impl Material {
    pub fn new(albedo: Vector3, specular: f32) -> Self {
        Material {
            albedo,
            specular,
            transparency: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
            emission: Vector3::zero(),
            texture: None,
            normal_map: None,
        }
    }

    pub fn with_texture(mut self, path: &'static str) -> Self {
        self.texture = Some(path);
        self
    }

    pub fn with_normal_map(mut self, path: &'static str) -> Self {
        self.normal_map = Some(path);
        self
    }

    pub fn with_reflectivity(mut self, reflectivity: f32) -> Self {
        self.reflectivity = reflectivity;
        self
    }

    pub fn with_refraction(mut self, transparency: f32, refractive_index: f32) -> Self {
        self.transparency = transparency;
        self.refractive_index = refractive_index;
        self
    }

    pub fn with_emission(mut self, emission: Vector3) -> Self {
        self.emission = emission;
        self
    }

    pub fn is_emissive(&self) -> bool {
        self.emission.x > 0.0 || self.emission.y > 0.0 || self.emission.z > 0.0
    }
}

/// Paleta de materiales de la escena: campamento nocturno en el bosque.
/// Cada uno mapea directo a un punto de la rúbrica.
pub struct SceneMaterials {
    pub grass: Material,
    pub dirt: Material,
    pub stone: Material,
    pub bark: Material,      // tronco de árbol -> mapa normal
    pub leaves: Material,    // hojas oscuras azul-verdosas
    pub leaves_alt: Material,// variación de hojas
    pub campfire: Material,  // fogata -> emisivo
    pub fire_base: Material, // base de fogata naranja intenso
    pub fire_mid: Material,  // centro de fuego dorado
    pub fire_top: Material,  // punta de fuego amarilla
    pub ember: Material,     // brasas incandescentes
    pub smoke: Material,     // humo translúcido
    pub tuft: Material,      // mechón de pasto verde-amarillo
    pub planks: Material,    // puente de tablones de madera
    pub gem: Material,       // cristal -> refracción + reflexión
    pub water: Material,     // charco -> reflexión (+ refracción opcional)
    pub straw: Material,     // círculo de paja alrededor de la fogata
}

impl SceneMaterials {
    pub fn load() -> Self {
        SceneMaterials {
            grass: Material::new(Vector3::new(0.20, 0.50, 0.35), 4.0)
                .with_texture("assets/grass.png"),

            dirt: Material::new(Vector3::new(0.40, 0.27, 0.15), 3.0)
                .with_texture("assets/dirt.png"),

            stone: Material::new(Vector3::new(0.45, 0.45, 0.47), 6.0)
                .with_texture("assets/stone.png")
                .with_normal_map("assets/stone_normal.png"),

            bark: Material::new(Vector3::new(0.33, 0.20, 0.10), 3.0)
                .with_texture("assets/bark.png")
                .with_normal_map("assets/bark_normal.png"),

            leaves: Material::new(Vector3::new(0.08, 0.22, 0.16), 2.0)
                .with_texture("assets/leaves.png"),

            leaves_alt: Material::new(Vector3::new(0.06, 0.18, 0.14), 2.0)
                .with_texture("assets/leaves.png"),

            campfire: Material::new(Vector3::new(1.0, 0.55, 0.1), 10.0)
                .with_texture("assets/fire.png")
                .with_emission(Vector3::new(1.0, 0.5, 0.1)),

            fire_base: Material::new(Vector3::new(1.0, 0.38, 0.06), 8.0)
                .with_texture("assets/fire.png")
                .with_emission(Vector3::new(1.1, 0.42, 0.08)),

            fire_mid: Material::new(Vector3::new(1.0, 0.65, 0.15), 10.0)
                .with_texture("assets/fire.png")
                .with_emission(Vector3::new(1.2, 0.70, 0.18)),

            fire_top: Material::new(Vector3::new(1.0, 0.90, 0.30), 12.0)
                .with_texture("assets/fire.png")
                .with_emission(Vector3::new(1.3, 1.00, 0.35)),

            ember: Material::new(Vector3::new(1.0, 0.75, 0.20), 10.0)
                .with_emission(Vector3::new(1.4, 0.75, 0.15)),

            smoke: Material::new(Vector3::new(0.60, 0.62, 0.68), 1.0)
                .with_refraction(0.60, 1.0),

            tuft: Material::new(Vector3::new(0.50, 0.60, 0.30), 2.0)
                .with_texture("assets/tuft.png"),

            planks: Material::new(Vector3::new(0.55, 0.38, 0.22), 4.0)
                .with_texture("assets/planks.png"),

            gem: Material::new(Vector3::new(0.6, 0.9, 1.0), 60.0)
                .with_texture("assets/gem.png")
                .with_reflectivity(0.15)
                .with_refraction(0.85, 1.55),

            water: Material::new(Vector3::new(0.15, 0.25, 0.35), 40.0)
                .with_texture("assets/water.png")
                .with_reflectivity(0.5)
                .with_refraction(0.3, 1.33),

            straw: Material::new(Vector3::new(0.65, 0.50, 0.25), 3.0)
                .with_texture("assets/straw.png"),
        }
    }
}
