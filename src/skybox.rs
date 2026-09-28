use raylib::prelude::Vector3;

/// Skybox procedural muy barato: un degradado vertical de "cielo nocturno"
/// más un patrón pseudoaleatorio de estrellas, evaluado directo por dirección
/// de rayo (sin necesidad de cargar una imagen equirectangular).
pub struct Skybox {
    pub horizon_color: Vector3,
    pub zenith_color: Vector3,
    pub star_density: f32,
}

impl Skybox {
    pub fn night() -> Self {
        Skybox {
            horizon_color: Vector3::new(0.05, 0.07, 0.15),
            zenith_color: Vector3::new(0.01, 0.01, 0.04),
            star_density: 0.0015,
        }
    }

    /// Devuelve el color del cielo para una dirección de rayo que no golpeó nada.
    pub fn sample(&self, dir: Vector3) -> Vector3 {
        let t = (dir.y * 0.5 + 0.5).clamp(0.0, 1.0);
        let sky = lerp_vec3(self.horizon_color, self.zenith_color, t);

        if self.is_star(dir) {
            Vector3::new(1.0, 1.0, 1.0)
        } else {
            sky
        }
    }

    /// Hash determinístico de la dirección para decidir si cae una estrella ahí.
    /// Determinístico == la posición de las estrellas no "parpadea" al rotar cámara.
    fn is_star(&self, dir: Vector3) -> bool {
        let hash = ((dir.x * 12.9898 + dir.y * 78.233 + dir.z * 37.719).sin() * 43758.5453).fract();
        hash.abs() < self.star_density
    }
}

fn lerp_vec3(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a + (b - a) * t
}
