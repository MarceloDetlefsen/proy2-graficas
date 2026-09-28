use raylib::prelude::Vector3;

pub struct Light {
    pub position: Vector3,
    pub color: Vector3,
    pub intensity: f32,
    pub radius: f32,
    pub casts_shadow: bool,
}

impl Light {
    pub fn new(position: Vector3, color: Vector3, intensity: f32, radius: f32, casts_shadow: bool) -> Self {
        Light {
            position,
            color,
            intensity,
            radius,
            casts_shadow,
        }
    }

    /// Luz cálida de la fogata: atenuación suave 1 / (1 + (d/r)^2) con r ~ 4.8,
    /// color (1.0, 0.60, 0.25). Llega con fuerza a ~3 bloques y alcanza los troncos con 25-35%.
    pub fn campfire(position: Vector3) -> Self {
        Light::new(position, Vector3::new(1.0, 0.60, 0.25), 1.0, 4.8, true)
    }

    /// Luz de relleno azul tenue de noche (sin sombras) para no encarecer el render
    pub fn sky_fill(position: Vector3) -> Self {
        Light::new(position, Vector3::new(0.12, 0.20, 0.45), 0.35, 25.0, false)
    }
}
